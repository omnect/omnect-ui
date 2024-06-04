use actix_files::{Files, NamedFile};
use actix_web::{http::StatusCode, web, App, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::extractors::{basic::BasicAuth, bearer::BearerAuth};
use env_logger::{Builder, Env, Target};
use http_body_util::{BodyExt, Empty};
use hyper::{
    Request,
    {body::Bytes, client::conn::http1},
};
use hyper_util::rt::TokioIo;
use jwt_simple::prelude::*;
use log::{debug, error, info};
use std::io::Write;
use tokio::{net::UnixStream, process::Command};

const TOKEN_EXPIRE_HOURES: u64 = 2;

#[macro_export]
macro_rules! socket_path {
    () => {{
        const SOCKET_PATH_DEFAULT: &'static str = "/run/omnect-device-service/api.sock";
        std::env::var("SOCKET_PATH").unwrap_or(SOCKET_PATH_DEFAULT.to_string())
    }};
}

#[macro_export]
macro_rules! ssl_cert_path {
    () => {{
        const SSL_CERT_PATH_DEFAULT: &'static str = "/cert/device_id_cert.pem";
        std::env::var("SSL_CERT_PATH").unwrap_or(SSL_CERT_PATH_DEFAULT.to_string())
    }};
}

#[macro_export]
macro_rules! ssl_key_path {
    () => {{
        const SSL_KEY_PATH_DEFAULT: &'static str = "/cert/device_id_cert_key.pem";
        std::env::var("SSL_KEY_PATH").unwrap_or(SSL_KEY_PATH_DEFAULT.to_string())
    }};
}

#[actix_web::main]
async fn main() {
    let mut builder;
    log_panics::init();

    if cfg!(debug_assertions) {
        builder = Builder::from_env(Env::default().default_filter_or("debug"));
    } else {
        builder = Builder::from_env(Env::default().default_filter_or("info"));
    }

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

    let mut certs_file =
        std::io::BufReader::new(std::fs::File::open(ssl_cert_path!()).expect("read ssl cert"));
    let mut key_file =
        std::io::BufReader::new(std::fs::File::open(ssl_key_path!()).expect("read ssl key"));

    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to parse cert pem");

    let tls_key = rustls_pemfile::rsa_private_keys(&mut key_file)
        .next()
        .expect("no keys found")
        .expect("invalid key found");

    // set up TLS config options
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs1(tls_key))
        .expect("invalid tls config");

    let Ok(server) = HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(index))
            .route("/token/login", web::post().to(login_token))
            .route("/token/refresh", web::get().to(refresh_token))
            .route("/reboot", web::post().to(reboot))
            .route("/reload-network", web::post().to(reload_network))
            .service(
                Files::new(
                    "/static",
                    std::fs::canonicalize("static").expect("static folder not found"),
                )
                .show_files_listing(),
            )
    })
    .bind_rustls_0_22("0.0.0.0:1977", tls_config) else {
        error!("cannot bind server");
        return;
    };

    let mut centrifugo =
        Command::new(std::fs::canonicalize("centrifugo").expect("centrifugo not found"))
            .envs(vec![
                ("CENTRIFUGO_ALLOWED_ORIGINS", "*"),
                ("CENTRIFUGO_ALLOW_SUBSCRIBE_FOR_CLIENT", "true"),
                ("CENTRIFUGO_ALLOW_HISTORY_FOR_CLIENT", "true"),
                ("CENTRIFUGO_HISTORY_SIZE", "1"),
                ("CENTRIFUGO_HISTORY_TTL", "720h"),
            ])
            .spawn()
            .expect("Failed to spawn child process");

    tokio::select! {
        _ = server.run() => {debug!("1");centrifugo.kill().await.expect("kill failed")},
        _ = centrifugo.wait() => {}
    }
}

async fn index() -> actix_web::Result<NamedFile> {
    debug!("index() called");

    // trigger omnect-device-service to republish
    let res = post("/republish/v1", None).await;

    if let Some(e) = res.error() {
        error!("republish failed: {e}");

        return Err(actix_web::error::ErrorInternalServerError(
            "republish failed",
        ));
    }

    Ok(NamedFile::open(
        std::fs::canonicalize("static/index.html").expect("static/index.html not found"),
    )?)
}

async fn login_token(auth: BasicAuth) -> impl Responder {
    debug!("login_token() called");
    let Ok(user) = std::env::var("LOGIN_USER") else {
        error!("login_token: missing user");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };
    let Ok(password) = std::env::var("LOGIN_PASSWORD") else {
        error!("login_token: missing password");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };
    if auth.user_id() != user || auth.password() != Some(password.as_str()) {
        error!("login_token: wrong credentials");
        return HttpResponse::build(StatusCode::UNAUTHORIZED).finish();
    }
    let Ok(key) = std::env::var("CENTRIFUGO_TOKEN_HMAC_SECRET_KEY") else {
        error!("login_token: missing secret key");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };
    let key = HS256Key::from_bytes(key.as_bytes());
    let claims =
        Claims::create(Duration::from_hours(TOKEN_EXPIRE_HOURES)).with_subject("omnect-ui");

    let Ok(token) = key.authenticate(claims) else {
        error!("login_token: cannot create token");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };

    HttpResponse::Ok().body(token)
}

async fn refresh_token(auth: BearerAuth) -> impl Responder {
    debug!("refresh_token() called");

    let (status_code, error_msg) = verify(auth);

    if status_code != StatusCode::OK {
        error!("refresh_token verify: {error_msg}");
        return HttpResponse::build(status_code).finish();
    }

    let Ok(key) = std::env::var("CENTRIFUGO_TOKEN_HMAC_SECRET_KEY") else {
        error!("refresh_token: missing secret key");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };
    let key = HS256Key::from_bytes(key.as_bytes());
    let claims =
        Claims::create(Duration::from_hours(TOKEN_EXPIRE_HOURES)).with_subject("omnect-ui");

    let Ok(token) = key.authenticate(claims) else {
        error!("refresh_token: cannot create token");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };

    HttpResponse::Ok().body(token)
}

async fn reboot(auth: BearerAuth) -> impl Responder {
    debug!("reboot() called");

    post("/reboot/v1", Some(auth)).await
}

async fn reload_network(auth: BearerAuth) -> impl Responder {
    debug!("reload_network() called");

    post("/reload-network/v1", Some(auth)).await
}

async fn post(path: &str, auth: Option<BearerAuth>) -> HttpResponse {
    if let Some(auth) = auth {
        let (status_code, error_msg) = verify(auth);

        if status_code != StatusCode::OK {
            error!("put {path} verify: {error_msg}");
            return HttpResponse::build(status_code).finish();
        }
    }

    let Ok(stream) = UnixStream::connect(socket_path!()).await else {
        error!("cannot create unix stream {}", socket_path!());
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };
    let Ok((mut sender, conn)) = http1::handshake(TokioIo::new(stream)).await else {
        error!("unix stream handshake failed");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };

    actix_rt::spawn(async move {
        if let Err(err) = conn.await {
            error!("Connection failed1: {:?}", err);
        }
    });

    if sender.ready().await.is_err() {
        error!("unix stream unexpectedly closed");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    }

    let Ok(request) = Request::builder()
        .uri(path)
        .method("POST")
        .header("Host", "localhost")
        .body(Empty::<Bytes>::new())
    else {
        error!("build request failed");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };

    let Ok(res) = sender.send_request(request).await else {
        error!("send request failed");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };
    let Ok(status_code) = StatusCode::from_u16(res.status().as_u16()) else {
        error!("get status code failed");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };
    let Ok(body) = res.collect().await else {
        error!("collect response body failed");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };
    let Ok(body) = String::from_utf8(body.to_bytes().to_vec()) else {
        error!("get response body failed");
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish();
    };

    HttpResponse::build(status_code).body(body)
}

fn verify(auth: BearerAuth) -> (StatusCode, String) {
    let Ok(key) = std::env::var("CENTRIFUGO_TOKEN_HMAC_SECRET_KEY") else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "missing jwt secret".to_string(),
        );
    };

    let key = HS256Key::from_bytes(key.as_bytes());
    let options = VerificationOptions {
        accept_future: true,
        time_tolerance: Some(Duration::from_mins(15)),
        max_validity: Some(Duration::from_hours(TOKEN_EXPIRE_HOURES)),
        required_subject: Some("omnect-ui".to_string()),
        ..Default::default()
    };

    if let Err(e) = key.verify_token::<NoCustomClaims>(auth.token(), Some(options)) {
        return (
            StatusCode::UNAUTHORIZED,
            format!("verify jwt token failed: {e}"),
        );
    };

    (StatusCode::OK, "ok".to_string())
}
