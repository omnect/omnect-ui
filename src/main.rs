mod api;
mod certificate;
mod common;
mod keycloak_client;
mod middleware;
mod omnect_device_service_client;
mod socket_client;

use crate::{api::Api, certificate::create_module_certificate};
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
use common::{centrifugo_config, config_path};
use env_logger::{Builder, Env, Target};
use log::{debug, error, info};
use rustls::crypto::{CryptoProvider, ring::default_provider};
use std::{fs, io::Write, sync::Arc};
use tokio::{
    process::{Child, Command},
    signal::unix::{SignalKind, signal},
    sync::{RwLock, broadcast},
    task::AbortHandle,
};

const UPLOAD_LIMIT_BYTES: usize = 250 * 1024 * 1024;
const MEMORY_LIMIT_BYTES: usize = 10 * 1024 * 1024;

// Global server restart signal
static SERVER_RESTART_TX: std::sync::OnceLock<broadcast::Sender<()>> = std::sync::OnceLock::new();

// Global rollback timer handle
static ROLLBACK_TIMER: std::sync::OnceLock<Arc<RwLock<Option<AbortHandle>>>> =
    std::sync::OnceLock::new();

pub fn trigger_server_restart() {
    if let Some(tx) = SERVER_RESTART_TX.get() {
        let _ = tx.send(());
    }
}

pub async fn cancel_rollback_timer() {
    if let Some(timer_handle) = ROLLBACK_TIMER.get() {
        if let Some(handle) = timer_handle.write().await.take() {
            handle.abort();
            info!("Rollback timer cancelled - network change confirmed");
        }
    }
}

pub async fn set_rollback_timer(handle: AbortHandle) {
    if let Some(timer_handle) = ROLLBACK_TIMER.get() {
        *timer_handle.write().await = Some(handle);
    }
}

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

    create_module_certificate(None)
        .await
        .expect("failed to create module certificate");

    // Create restart signal channel
    let (restart_tx, mut restart_rx) = broadcast::channel(1);
    SERVER_RESTART_TX
        .set(restart_tx)
        .expect("Failed to set restart channel");

    // Initialize rollback timer handle
    ROLLBACK_TIMER
        .set(Arc::new(RwLock::new(None)))
        .expect("Failed to set rollback timer");

    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");

    CryptoProvider::install_default(default_provider()).expect("failed to install crypto provider");

    loop {
        let mut centrifugo = run_centrifugo();
        let (server_handle, server_task) = run_server().await;

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                debug!("ctrl-c");
                server_handle.stop(true).await;
                break;
            },
            _ = sigterm.recv() => {
                debug!("SIGTERM received");
                server_handle.stop(true).await;
                break;
            },
            _ = restart_rx.recv() => {
                debug!("Server restart requested");
                server_handle.stop(true).await;
                centrifugo.kill().await.expect("kill centrifugo failed");
                info!("Server stopped, restarting...");
            },
            result = server_task => {
                match result {
                    Ok(_) => debug!("server stopped normally"),
                    Err(e) => debug!("server stopped with error: {e}"),
                }
                centrifugo.kill().await.expect("kill centrifugo failed");
                debug!("centrifugo killed");
                break;
            },
            _ = centrifugo.wait() => {
                debug!("centrifugo stopped");
                server_handle.stop(true).await;
                debug!("server stopped");
                break;
            }
        }
    }

    debug!("good bye");
}

async fn run_server() -> (
    ServerHandle,
    tokio::task::JoinHandle<Result<(), std::io::Error>>,
) {
    let Ok(true) = fs::exists("/data") else {
        panic!("data dir /data is missing");
    };

    if !fs::exists(config_path!()).is_ok_and(|ok| ok) {
        fs::create_dir_all(config_path!()).expect("failed to create config directory");
    };

    common::create_frontend_config_file().expect("failed to create frontend config file");

    let api = Api::new().await.expect("failed to create api");

    if let Err(e) = api.check_and_execute_pending_rollback().await {
        error!("Failed to check pending rollback: {e:#}");
    }

    let mut tls_certs = std::io::BufReader::new(
        std::fs::File::open(certificate::cert_path()).expect("read certs_file"),
    );
    let mut tls_key = std::io::BufReader::new(
        std::fs::File::open(certificate::key_path()).expect("read key_file"),
    );

    let tls_certs = rustls_pemfile::certs(&mut tls_certs)
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to parse cert pem");

    // set up TLS config options
    let tls_config = match rustls_pemfile::read_one(&mut tls_key)
        .expect("cannot read key pem file")
        .expect("nothing found in key pem file")
    {
        rustls_pemfile::Item::Pkcs1Key(key) => rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs1(key))
            .expect("invalid tls config"),
        rustls_pemfile::Item::Pkcs8Key(key) => rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(key))
            .expect("invalid tls config"),
        _ => panic!("unexpected item found in key pem file"),
    };

    let ui_port = std::env::var("UI_PORT")
        .expect("UI_PORT missing")
        .parse::<u64>()
        .expect("UI_PORT format");

    let session_key = Key::generate();

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
            .app_data(Data::new(api.clone()))
            .route("/", web::get().to(Api::index))
            .route("/config.js", web::get().to(Api::config))
            .route(
                "/factory-reset",
                web::post().to(Api::factory_reset).wrap(middleware::AuthMw),
            )
            .route(
                "/reboot",
                web::post().to(Api::reboot).wrap(middleware::AuthMw),
            )
            .route(
                "/reload-network",
                web::post().to(Api::reload_network).wrap(middleware::AuthMw),
            )
            .route(
                "/update/file",
                web::post().to(Api::save_file).wrap(middleware::AuthMw),
            )
            .route(
                "/update/load",
                web::post().to(Api::load_update).wrap(middleware::AuthMw),
            )
            .route(
                "/update/run",
                web::post().to(Api::run_update).wrap(middleware::AuthMw),
            )
            .route(
                "/token/login",
                web::post().to(Api::token).wrap(middleware::AuthMw),
            )
            .route(
                "/token/refresh",
                web::get().to(Api::token).wrap(middleware::AuthMw),
            )
            .route(
                "/token/validate",
                web::post().to(Api::validate_portal_token),
            )
            .route(
                "/require-set-password",
                web::get().to(Api::require_set_password),
            )
            .route("/set-password", web::post().to(Api::set_password))
            .route("/update-password", web::post().to(Api::update_password))
            .route("/version", web::get().to(Api::version))
            .route("/logout", web::post().to(Api::logout))
            .route("/healthcheck", web::get().to(Api::healthcheck))
            .route("/network", web::post().to(Api::set_network))
            .service(Files::new(
                "/static",
                std::fs::canonicalize("static").expect("static folder not found"),
            ))
            .default_service(web::route().to(Api::index))
    })
    .bind_rustls_0_23(format!("0.0.0.0:{ui_port}"), tls_config)
    .expect("bind_rustls")
    .disable_signals()
    .run();

    (server.handle(), tokio::spawn(server))
}

fn run_centrifugo() -> Child {
    let centrifugo =
        Command::new(std::fs::canonicalize("centrifugo").expect("centrifugo not found"))
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
                ("CENTRIFUGO_HTTP_SERVER_PORT", centrifugo_config().port),
                (
                    "CENTRIFUGO_CLIENT_TOKEN_HMAC_SECRET_KEY",
                    centrifugo_config().client_token,
                ),
                ("CENTRIFUGO_HTTP_API_KEY", centrifugo_config().api_key),
            ])
            .spawn()
            .expect("Failed to spawn child process");

    info!(
        "centrifugo pid: {}",
        centrifugo.id().expect("centrifugo pid")
    );

    centrifugo
}
