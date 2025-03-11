mod api;
mod middleware;
mod socket_client;

use actix_files::Files;
use actix_multipart::form::MultipartFormConfig;
use actix_session::{
    config::{BrowserSession, CookieContentSecurity},
    storage::CookieSessionStore,
    SessionMiddleware,
};
use actix_web::{
    cookie::{Key, SameSite},
    web, App, HttpServer,
};
use anyhow::Result;
use env_logger::{Builder, Env, Target};
use log::{debug, info};
use std::{fs, io::Write};
use tokio::process::Command;

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

    info!("module version: {}", env!("CARGO_PKG_VERSION"));

    let ui_port = std::env::var("UI_PORT")
        .expect("UI_PORT missing")
        .parse::<u64>()
        .expect("UI_PORT format");

    let device_cert_path = std::env::var("SSL_CERT_PATH").expect("SSL_CERT_PATH missing");
    let device_key_path = std::env::var("SSL_KEY_PATH").expect("SSL_KEY_PATH missing");

    info!("device cert file: {device_cert_path}");
    info!("device key file: {device_key_path}");

    let mut tls_certs =
        std::io::BufReader::new(std::fs::File::open(device_cert_path).expect("read certs_file"));
    let mut tls_key =
        std::io::BufReader::new(std::fs::File::open(device_key_path).expect("read key_file"));

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

    fs::exists("/data").expect("data dir /data is missing");

    fn session_middleware() -> SessionMiddleware<CookieSessionStore> {
        SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
            .cookie_name(String::from("omnect-ui-session"))
            .cookie_secure(true)
            .session_lifecycle(BrowserSession::default())
            .cookie_same_site(SameSite::Strict)
            .cookie_content_security(CookieContentSecurity::Private)
            .cookie_http_only(true)
            .build()
    }

    let server = HttpServer::new(move || {
        App::new()
            .wrap(session_middleware())
            .app_data(
                MultipartFormConfig::default()
                    .total_limit(UPLOAD_LIMIT_BYTES)
                    .memory_limit(MEMORY_LIMIT_BYTES),
            )
            .route("/", web::get().to(api::index))
            .route(
                "/factory-reset",
                web::post().to(api::factory_reset).wrap(middleware::AuthMw),
            )
            .route(
                "/reboot",
                web::post().to(api::reboot).wrap(middleware::AuthMw),
            )
            .route(
                "/reload-network",
                web::post().to(api::reload_network).wrap(middleware::AuthMw),
            )
            .route(
                "/update/file",
                web::post().to(api::save_file).wrap(middleware::AuthMw),
            )
            .route(
                "/update/load",
                web::post().to(api::load_update).wrap(middleware::AuthMw),
            )
            .route(
                "/update/run",
                web::post().to(api::run_update).wrap(middleware::AuthMw),
            )
            .route(
                "/token/login",
                web::post().to(api::token).wrap(middleware::AuthMw),
            )
            .route(
                "/token/refresh",
                web::get().to(api::token).wrap(middleware::AuthMw),
            )
            .route("/version", web::get().to(api::version))
            .route("/logout", web::post().to(api::logout))
            .service(Files::new(
                "/static",
                std::fs::canonicalize("static").expect("static folder not found"),
            ))
            .default_service(web::route().to(api::index))
    })
    .bind_rustls_0_23(format!("0.0.0.0:{ui_port}"), tls_config)
    .expect("bind_rustls")
    .disable_signals()
    .run();

    let server_handle = server.handle();
    let server_task = tokio::spawn(server);

    let mut centrifugo =
        Command::new(std::fs::canonicalize("centrifugo").expect("centrifugo not found"))
            .spawn()
            .expect("Failed to spawn child process");

    debug!("centrifugo pid: {}", centrifugo.id().unwrap());

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            debug!("ctrl-c");
            server_handle.stop(true).await;
        },
        _ = server_task => {
            debug!("server stopped");
            centrifugo.kill().await.expect("kill centrifugo failed");
            debug!("centrifugo killed");
        },
        _ = centrifugo.wait() => {
            debug!("centrifugo stopped");
            server_handle.stop(true).await;
            debug!("server stopped");
        }
    }

    debug!("good bye");
}
