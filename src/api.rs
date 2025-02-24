use crate::middleware::TOKEN_EXPIRE_HOURS;
use crate::socket_client::*;
use actix_files::NamedFile;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_session::Session;
use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use anyhow::Result;
use jwt_simple::prelude::*;
use log::{debug, error};
use std::{fs, os::unix::fs::PermissionsExt};

#[derive(Deserialize)]
pub struct FactoryResetInput {
    preserve: Vec<String>,
}

#[derive(Serialize)]
pub struct FactoryResetPayload {
    mode: u8,
    preserve: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LoadUpdatePayload {
    update_file_path: String,
}

#[derive(Serialize, Deserialize)]
pub struct RunUpdatePayload {
    validate_iothub_connection: bool,
}

#[derive(MultipartForm)]
pub struct UploadFormSingleFile {
    file: TempFile,
}

pub async fn index() -> actix_web::Result<NamedFile> {
    debug!("index() called");

    match post_with_empty_body("/republish/v1").await {
        Ok(response) => response,
        Err(e) => {
            error!("republish failed: {e:#}");
            return Err(actix_web::error::ErrorInternalServerError(
                "republish failed",
            ));
        }
    };

    Ok(NamedFile::open(
        std::fs::canonicalize("static/index.html").expect("static/index.html not found"),
    )?)
}

pub async fn factory_reset(body: web::Json<FactoryResetInput>) -> impl Responder {
    debug!(
        "factory_reset() called with preserved keys {}",
        body.preserve.join(",")
    );

    let payload = FactoryResetPayload {
        mode: 1,
        preserve: body.preserve.clone(),
    };

    match post_with_json_body("/factory-reset/v1", Some(payload)).await {
        Ok(response) => response,
        Err(e) => {
            error!("factory_reset failed: {e:#}");
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

pub async fn reboot() -> impl Responder {
    debug!("reboot() called");

    match post_with_empty_body("/reboot/v1").await {
        Ok(response) => response,
        Err(e) => {
            error!("reboot failed: {e:#}");
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

pub async fn reload_network() -> impl Responder {
    debug!("reload_network() called");

    match post_with_empty_body("/reload-network/v1").await {
        Ok(response) => response,
        Err(e) => {
            error!("reload-network failed: {e:#}");
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

pub async fn token(session: Session) -> impl Responder {
    if let Ok(key) = std::env::var("CENTRIFUGO_CLIENT_TOKEN_HMAC_SECRET_KEY") {
        let key = HS256Key::from_bytes(key.as_bytes());
        let claims =
            Claims::create(Duration::from_hours(TOKEN_EXPIRE_HOURS)).with_subject("omnect-ui");

        if let Ok(token) = key.authenticate(claims) {
            match session.insert("token", token.clone()) {
                Ok(_) => return HttpResponse::Ok().body(token),
                Err(_) => return HttpResponse::InternalServerError().body("Error."),
            }
        } else {
            error!("token: cannot create token");
        };
    } else {
        error!("token: missing secret key");
    };

    HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()
}

pub async fn logout(session: Session) -> impl Responder {
    debug!("logout() called");
    session.purge();
    return HttpResponse::Ok();
}

pub async fn save_file(MultipartForm(form): MultipartForm<UploadFormSingleFile>) -> impl Responder {
    debug!("save_file() called");

    if !form.file.file_name.is_some() {
        HttpResponse::BadRequest().body("Update file is missing")
    } else {
        let _ = clear_data_folder().await;

        let filename = form.file.file_name.unwrap();
        let tmp_path = format!("/tmp/{filename}");
        let data_path = format!("/data/{filename}");

        match form.file.file.persist(&tmp_path) {
            Ok(_) => match fs::copy(tmp_path, &data_path) {
                Ok(_) => {
                    let metadata = fs::metadata(&data_path);
                    if metadata.is_err() {
                        return HttpResponse::InternalServerError().finish();
                    }

                    let mut perm = metadata.unwrap().permissions();
                    perm.set_mode(0o750);

                    match fs::set_permissions(&data_path, perm) {
                        Ok(_) => return HttpResponse::Ok().finish(),
                        Err(err) => {
                            error!("Store file failed: {:?}", err);
                            HttpResponse::InternalServerError().finish()
                        }
                    }
                }
                Err(err) => {
                    error!("Store file failed: {:?}", err);
                    HttpResponse::InternalServerError().finish()
                }
            },
            Err(err) => {
                error!("Store file failed: {:?}", err);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

pub async fn load_update(mut body: web::Json<LoadUpdatePayload>) -> impl Responder {
    debug!(
        "load_update() called with path {}",
        body.update_file_path.clone()
    );

    let update_os_path = std::env::var("UPDATE_PATH").expect("UPDATE_PATH missing");

    body.update_file_path = format!("{update_os_path}/{}", body.update_file_path);

    match post_with_json_body("/fwupdate/load/v1", Some(body)).await {
        Ok(response) => response,
        Err(e) => {
            error!("factory_reset failed: {e:#}");
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

pub async fn run_update(body: web::Json<RunUpdatePayload>) -> impl Responder {
    debug!(
        "run_update() called with validate_iothub_connection {}",
        body.validate_iothub_connection.clone()
    );

    match post_with_json_body("/fwupdate/run/v1", Some(body)).await {
        Ok(response) => response,
        Err(e) => {
            error!("factory_reset failed: {e:#}");
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

async fn clear_data_folder() -> Result<bool> {
    debug!("clear_data_folder() called");
    for entry in fs::read_dir("/data")? {
        let entry = entry?;
        fs::remove_file(entry.path())?;
    }

    Ok(true)
}
