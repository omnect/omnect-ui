use actix_files::{Files, NamedFile};
use actix_web::http::StatusCode;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::extractors::{basic::BasicAuth, bearer::BearerAuth};
use env_logger::{Builder, Env, Target};
use http_body_util::{BodyExt, Empty};
use hyper::{
    Request,
    {body::Bytes, client::conn::http1},
};
use hyper_util::rt::TokioIo;
use jwt_simple::prelude::*;
use log::{error, info};
use std::{io::Write, process::Command};
use tokio::net::UnixStream;

const JTW_EXPIRE_HOURES: u64 = 2;

#[macro_export]
macro_rules! socket_path {
    () => {{
        const SOCKET_PATH_DEFAULT: &'static str = "/run/omnect-device-service/api.sock";
        std::env::var("SOCKET_PATH").unwrap_or(SOCKET_PATH_DEFAULT.to_string())
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

    let Ok(server) = HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(index))
            .route("/token/login", web::post().to(login_token))
            .route("/token/refresh", web::get().to(refresh_token))
            .route("/reboot", web::post().to(reboot))
            .route("/reload-network", web::post().to(reload_network))
            .service(Files::new("/static", "static").show_files_listing())
    })
    .bind("localhost:1977") else {
        error!("cannot bind server");
        return;
    };

    let mut centrifugo = Command::new("./centrifugo")
        .envs(vec![
            ("CENTRIFUGO_ALLOWED_ORIGINS", "http://localhost:1977"),
            ("CENTRIFUGO_ALLOW_SUBSCRIBE_FOR_CLIENT", "true"),
            ("CENTRIFUGO_ALLOW_HISTORY_FOR_CLIENT", "true"),
            ("CENTRIFUGO_HISTORY_SIZE", "1"),
            ("CENTRIFUGO_HISTORY_TTL", "720h"),
        ])
        .spawn()
        .expect("Failed to spawn child process");
    let _ = server.run().await;
    let _ = centrifugo.kill();
}

async fn index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./static/index.html")?)
}

async fn login_token(auth: BasicAuth) -> HttpResponse {
    let Ok(user) = std::env::var("LOGIN_USER") else {
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("missing user");
    };
    let Ok(password) = std::env::var("LOGIN_PASSWORD") else {
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("missing password");
    };
    if auth.user_id() != user || auth.password() != Some(password.as_str()) {
        return HttpResponse::build(StatusCode::UNAUTHORIZED).body("wrong credentials");
    }
    let Ok(key) = std::env::var("CENTRIFUGO_TOKEN_HMAC_SECRET_KEY") else {
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("missing secret key");
    };
    let key = HS256Key::from_bytes(key.as_bytes());
    let claims = Claims::create(Duration::from_hours(JTW_EXPIRE_HOURES)).with_subject("omnect-ui");

    let Ok(token) = key.authenticate(claims) else {
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("cannot create token");
    };

    HttpResponse::Ok().body(token)
}

async fn refresh_token(auth: BearerAuth) -> HttpResponse {
    let (status_code, error_msg) = verify(auth);

    if status_code != StatusCode::OK {
        error!("refresh_token verify: {error_msg}");
        return HttpResponse::build(status_code).body(error_msg);
    }

    let Ok(key) = std::env::var("CENTRIFUGO_TOKEN_HMAC_SECRET_KEY") else {
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("missing secret key");
    };
    let key = HS256Key::from_bytes(key.as_bytes());
    let claims = Claims::create(Duration::from_hours(JTW_EXPIRE_HOURES)).with_subject("omnect-ui");

    let Ok(token) = key.authenticate(claims) else {
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).body("cannot create token");
    };

    HttpResponse::Ok().body(token)
}

async fn reboot(auth: BearerAuth) -> HttpResponse {
    let (code, body) = put("/reboot/v1", auth).await;
    if code != StatusCode::OK {
        error!("reboot failed: {body}");
    }
    HttpResponse::build(code).body(body)
}

async fn reload_network(auth: BearerAuth) -> impl Responder {
    let (code, body) = put("/reload-network/v1", auth).await;
    if code != StatusCode::OK {
        error!("reload-network failed: {body}");
    }
    HttpResponse::build(code).body(body)
}

async fn put(path: &str, auth: BearerAuth) -> (StatusCode, String) {
    let (status_code, error_msg) = verify(auth);

    if status_code != StatusCode::OK {
        error!("put {path} verify: {error_msg}");
        return (status_code, error_msg);
    }

    let Ok(stream) = UnixStream::connect(socket_path!()).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "cannot create unix stream".to_string(),
        );
    };
    let Ok((mut sender, conn)) = http1::handshake(TokioIo::new(stream)).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "unix stream handshake failed".to_string(),
        );
    };

    actix_rt::spawn(async move {
        if let Err(err) = conn.await {
            error!("Connection failed1: {:?}", err);
        }
    });

    if sender.ready().await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "unix stream unexpectedly closed".to_string(),
        );
    }

    let Ok(request) = Request::builder()
        .uri(path)
        .method("PUT")
        .header("Host", "localhost")
        .body(Empty::<Bytes>::new())
    else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "build request failed".to_string(),
        );
    };

    let Ok(res) = sender.send_request(request).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "send request failed".to_string(),
        );
    };
    let Ok(status_code) = StatusCode::from_u16(res.status().as_u16()) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "get status code failed".to_string(),
        );
    };
    let Ok(body) = res.collect().await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "collect response body failed".to_string(),
        );
    };
    let Ok(body) = String::from_utf8(body.to_bytes().to_vec()) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "get response body failed".to_string(),
        );
    };

    (status_code, body.to_string())
}

fn verify(auth: BearerAuth) -> (StatusCode, String) {
    let Ok(key) = std::env::var("CENTRIFUGO_TOKEN_HMAC_SECRET_KEY") else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "missing jwt secret".to_string(),
        );
    };

    let key = HS256Key::from_bytes(key.as_bytes());
    let mut options = VerificationOptions::default();

    options.accept_future = true;
    options.time_tolerance = Some(Duration::from_mins(15));
    options.max_validity = Some(Duration::from_hours(JTW_EXPIRE_HOURES));
    options.required_subject = Some("omnect-ui".to_string());

    if let Err(e) = key.verify_token::<NoCustomClaims>(&auth.token(), Some(options)) {
        return (
            StatusCode::UNAUTHORIZED,
            format!("verify jwt token failed: {e}"),
        );
    };

    (StatusCode::OK, "ok".to_string())
}
