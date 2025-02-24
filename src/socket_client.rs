use actix_web::{http::StatusCode, HttpResponse};
use anyhow::{Context, Result};
use http_body_util::BodyExt;
use hyper::{client::conn::http1, Request};
use hyper_util::rt::TokioIo;
use log::error;
use serde::Serialize;
use tokio::net::UnixStream;

pub async fn post_with_json_body(path: &str, body: impl Serialize) -> Result<HttpResponse> {
    let json = match serde_json::to_value(body) {
        Ok(r) => r,
        Err(e) => {
            error!("failed to serialize data error: {e:#}");
            return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish());
        }
    };

    let request = Request::builder()
        .uri(path)
        .method("POST")
        .header("Host", "localhost")
        .body(serde_json::to_string(&json).unwrap_or_default())
        .context("build request failed")?;

    post(request).await
}

pub async fn post_with_empty_body(path: &str) -> Result<HttpResponse> {
    let request = Request::builder()
        .uri(path)
        .method("POST")
        .header("Host", "localhost")
        .body(String::new())
        .context("build request failed")?;

    post(request).await
}

async fn post(request: Request<String>) -> Result<HttpResponse> {
    let mut sender = match sender().await {
        Err(e) => {
            error!("error creating request sender: {e}. socket might be broken. exit application");
            std::process::exit(1)
        }
        Ok(sender) => sender,
    };

    let res = sender
        .send_request(request)
        .await
        .context("send request failed")?;

    let status_code =
        StatusCode::from_u16(res.status().as_u16()).context("get status code failed")?;

    let body = res
        .collect()
        .await
        .context("collect response body failed")?;

    let body = String::from_utf8(body.to_bytes().to_vec()).context("get response body failed")?;

    Ok(HttpResponse::build(status_code).body(body))
}

async fn sender() -> Result<http1::SendRequest<String>> {
    let stream = UnixStream::connect(std::env::var("SOCKET_PATH").expect("SOCKET_PATH missing"))
        .await
        .context("cannot create unix stream")?;

    let (mut sender, conn) = http1::handshake(TokioIo::new(stream))
        .await
        .context("unix stream handshake failed")?;

    actix_rt::spawn(async move {
        if let Err(err) = conn.await {
            error!("post connection failed: {:?}", err);
        }
    });

    sender
        .ready()
        .await
        .context("unix stream unexpectedly closed")?;

    Ok(sender)
}
