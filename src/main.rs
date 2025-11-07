mod api;
mod auth;
mod certificate;
mod common;
mod http_client;
mod keycloak_client;
mod middleware;
mod network;
mod omnect_device_service_client;

use crate::{
    api::Api,
    auth::TokenManager,
    certificate::create_module_certificate,
    common::{centrifugo_config, config_path},
    keycloak_client::KeycloakProvider,
    network::NetworkConfigService,
    omnect_device_service_client::{DeviceServiceClient, OmnectDeviceServiceClient},
};
use actix_cors::Cors;
use actix_files::Files;
use actix_multipart::form::MultipartFormConfig;
use actix_server::ServerHandle;
use actix_session::{
    SessionMiddleware,
    config::{BrowserSession, CookieContentSecurity},
    storage::CookieSessionStore,
};
use actix_web::{
    App, HttpServer,
    cookie::{Key, SameSite},
    web::{self, Data},
};
use anyhow::Result;
use env_logger::{Builder, Env, Target};
use log::{debug, error, info};
use rustls::crypto::{CryptoProvider, ring::default_provider};
use std::{fs, io::Write};
use tokio::{
    process::{Child, Command},
    signal::unix::{SignalKind, signal},
    sync::broadcast,
};

const UPLOAD_LIMIT_BYTES: usize = 250 * 1024 * 1024;
const MEMORY_LIMIT_BYTES: usize = 10 * 1024 * 1024;

#[actix_web::main]
async fn main() {
    log_panics::init();

    let mut builder = if cfg!(debug_assertions) {
        Builder::from_env(Env::default().default_filter_or("debug"))
    } else {
        Builder::from_env(Env::default().default_filter_or("info"))
    };

    builder.format(|f, record| match record.level() {
        log::Level::Error => {
            eprintln!("{}", record.args());
            Ok(())
        }
        _ => {
            writeln!(f, "{}", record.args())
        }
    });

    builder.target(Target::Stdout).init();

    info!(
        "module version: {} ({})",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_SHORT_REV")
    );

    // Create restart signal channel
    let (restart_tx, mut restart_rx) = broadcast::channel(1);
    NetworkConfigService::init_server_restart_channel(restart_tx).expect("failed to set restart channel");

    let mut sigterm = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

    CryptoProvider::install_default(default_provider()).expect("failed to install crypto provider");

    while run_until_shutdown(&mut restart_rx, &mut sigterm).await {
        info!("restarting server...");
    }
}

async fn run_until_shutdown(
    restart_rx: &mut broadcast::Receiver<()>,
    sigterm: &mut tokio::signal::unix::Signal,
) -> bool {
    let mut centrifugo = run_centrifugo();
    let (server_handle, server_task, service_client) = run_server().await;

    let should_restart = tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            debug!("ctrl-c received");
            false
        },
        _ = sigterm.recv() => {
            debug!("SIGTERM received");
            false
        },
        _ = restart_rx.recv() => {
            debug!("server restart requested");
            true
        },
        result = server_task => {
            match result {
                Ok(Ok(())) => debug!("server stopped normally"),
                Ok(Err(e)) => debug!("server stopped with error: {e}"),
                Err(e) => debug!("server task panicked: {e}"),
            }
            false
        },
        _ = centrifugo.wait() => {
            debug!("centrifugo stopped unexpectedly");
            false
        }
    };

    // Unified cleanup sequence - ensures consistent shutdown regardless of exit reason
    if !should_restart {
        info!("shutting down...");
    }

    // 1. Shutdown service client first (unregister from omnect-device-service)
    if let Err(e) = service_client.shutdown().await {
        error!("failed to shutdown service client: {e:#}");
    }

    // 2. Stop the server gracefully
    server_handle.stop(true).await;
    if !should_restart {
        info!("server stopped");
    }

    // 3. Kill centrifugo
    if let Err(e) = centrifugo.kill().await {
        error!("failed to kill centrifugo: {e:#}");
    }
    if !should_restart {
        info!("centrifugo stopped");
    }

    should_restart
}

async fn run_server() -> (
    ServerHandle,
    tokio::task::JoinHandle<Result<(), std::io::Error>>,
    OmnectDeviceServiceClient,
) {
    let Ok(true) = fs::exists("/data") else {
        panic!("failed to find required data directory: /data is missing");
    };

    if !fs::exists(config_path!()).is_ok_and(|ok| ok) {
        fs::create_dir_all(config_path!()).expect("failed to create config directory");
    };

    common::create_frontend_config_file().expect("failed to create frontend config file");

    type UiApi = Api<OmnectDeviceServiceClient, KeycloakProvider>;

    let service_client = OmnectDeviceServiceClient::new(true)
        .await
        .expect("failed to create client to device service");

    let api = UiApi::new(service_client.clone(), Default::default())
        .await
        .expect("failed to create api");

    create_module_certificate(&service_client)
        .await
        .expect("failed to create module certificate");

    if let Err(e) = network::NetworkConfigService::process_pending_rollback(&service_client).await
    {
        error!("failed to check pending rollback: {e:#}");
    }

    let mut tls_certs = std::io::BufReader::new(
        std::fs::File::open(certificate::cert_path()).expect("failed to read certificate file"),
    );
    let mut tls_key = std::io::BufReader::new(
        std::fs::File::open(certificate::key_path()).expect("failed to read key file"),
    );

    let tls_certs = rustls_pemfile::certs(&mut tls_certs)
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to parse cert pem");

    // set up TLS config options
    let tls_config = match rustls_pemfile::read_one(&mut tls_key)
        .expect("failed to read key pem file")
        .expect("failed to parse key pem file: no valid key found")
    {
        rustls_pemfile::Item::Pkcs1Key(key) => rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs1(key))
            .expect("failed to create TLS config"),
        rustls_pemfile::Item::Pkcs8Key(key) => rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(key))
            .expect("failed to create TLS config"),
        _ => panic!("failed to parse key pem file: unexpected item type found"),
    };

    let ui_port = std::env::var("UI_PORT")
        .expect("failed to read UI_PORT environment variable")
        .parse::<u64>()
        .expect("failed to parse UI_PORT: invalid format");

    let session_key = Key::generate();

    // Create TokenManager with centrifugo client token
    let token_manager = TokenManager::new(&centrifugo_config().client_token);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allowed_methods(vec!["GET"])
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), session_key.clone())
                    .cookie_name(String::from("omnect-ui-session"))
                    .cookie_secure(true)
                    .session_lifecycle(BrowserSession::default())
                    .cookie_same_site(SameSite::Strict)
                    .cookie_content_security(CookieContentSecurity::Private)
                    .cookie_http_only(true)
                    .build(),
            )
            .app_data(
                MultipartFormConfig::default()
                    .total_limit(UPLOAD_LIMIT_BYTES)
                    .memory_limit(MEMORY_LIMIT_BYTES),
            )
            .app_data(Data::new(token_manager.clone()))
            .app_data(Data::new(api.clone()))
            .route("/", web::get().to(UiApi::index))
            .route("/config.js", web::get().to(UiApi::config))
            .route(
                "/factory-reset",
                web::post()
                    .to(UiApi::factory_reset)
                    .wrap(middleware::AuthMw),
            )
            .route(
                "/reboot",
                web::post().to(UiApi::reboot).wrap(middleware::AuthMw),
            )
            .route(
                "/reload-network",
                web::post()
                    .to(UiApi::reload_network)
                    .wrap(middleware::AuthMw),
            )
            .route(
                "/update/file",
                web::post().to(UiApi::save_file).wrap(middleware::AuthMw),
            )
            .route(
                "/update/load",
                web::post().to(UiApi::load_update).wrap(middleware::AuthMw),
            )
            .route(
                "/update/run",
                web::post().to(UiApi::run_update).wrap(middleware::AuthMw),
            )
            .route(
                "/token/login",
                web::post().to(UiApi::token).wrap(middleware::AuthMw),
            )
            .route(
                "/token/refresh",
                web::get().to(UiApi::token).wrap(middleware::AuthMw),
            )
            .route(
                "/token/validate",
                web::post().to(UiApi::validate_portal_token),
            )
            .route(
                "/require-set-password",
                web::get().to(UiApi::require_set_password),
            )
            .route("/set-password", web::post().to(UiApi::set_password))
            .route("/update-password", web::post().to(UiApi::update_password))
            .route("/version", web::get().to(UiApi::version))
            .route("/logout", web::post().to(UiApi::logout))
            .route("/healthcheck", web::get().to(UiApi::healthcheck))
            .route("/network", web::post().to(UiApi::set_network_config))
            .service(Files::new(
                "/static",
                std::fs::canonicalize("static").expect("failed to find static folder"),
            ))
            .default_service(web::route().to(UiApi::index))
    })
    .bind_rustls_0_23(format!("0.0.0.0:{ui_port}"), tls_config)
    .expect("failed to bind server with TLS")
    .disable_signals()
    .run();

    (server.handle(), tokio::spawn(server), service_client)
}

fn run_centrifugo() -> Child {
    let centrifugo = Command::new(
        std::fs::canonicalize("centrifugo").expect("failed to find centrifugo binary"),
    )
    .arg("-c")
    .arg("/centrifugo_config.json")
    .envs(vec![
        (
            "CENTRIFUGO_HTTP_SERVER_TLS_CERT_PEM",
            certificate::cert_path(),
        ),
        (
            "CENTRIFUGO_HTTP_SERVER_TLS_KEY_PEM",
            certificate::key_path(),
        ),
        (
            "CENTRIFUGO_HTTP_SERVER_PORT",
            centrifugo_config().port.clone(),
        ),
        (
            "CENTRIFUGO_CLIENT_TOKEN_HMAC_SECRET_KEY",
            centrifugo_config().client_token.clone(),
        ),
        (
            "CENTRIFUGO_HTTP_API_KEY",
            centrifugo_config().api_key.clone(),
        ),
    ])
    .spawn()
    .expect("failed to spawn centrifugo process");

    info!(
        "centrifugo pid: {}",
        centrifugo
            .id()
            .expect("failed to get centrifugo process id")
    );

    centrifugo
}
