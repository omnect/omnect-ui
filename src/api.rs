use crate::middleware::TOKEN_EXPIRE_HOURS;
use crate::socket_client::*;
use actix_files::NamedFile;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use anyhow::{Context, Result};
use jwt_simple::prelude::*;
use log::{debug, error};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

macro_rules! data_path {
    ($filename:expr) => {{
        Path::new("/data/").join($filename)
    }};
}

macro_rules! tmp_path {
    ($filename:expr) => {{
        Path::new("/tmp/").join($filename)
    }};
}

#[derive(Deserialize)]
pub struct FactoryResetInput {
    preserve: Vec<String>,
}

#[derive(Serialize)]
pub struct FactoryResetPayload {
    mode: FactoryResetMode,
    preserve: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LoadUpdatePayload {
    update_file_path: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct RunUpdatePayload {
    validate_iothub_connection: bool,
}

#[derive(MultipartForm)]
pub struct UploadFormSingleFile {
    file: TempFile,
}

#[derive(Clone, Debug, Deserialize_repr, PartialEq, Serialize_repr)]
#[repr(u8)]
pub enum FactoryResetMode {
    Mode1 = 1,
    Mode2 = 2,
    Mode3 = 3,
    Mode4 = 4,
}

pub struct Api {
    ods_socket_path: String,
    update_os_path: String,
    pub centrifugo_client_token_hmac_secret_key: String,
    pub username: String,
    pub password: String,
    index_html: PathBuf,
}

impl Api {
    pub fn new(
        ods_socket_path: &str,
        update_os_path: &str,
        centrifugo_client_token_hmac_secret_key: &str,
        username: &str,
        password: &str,
        index_html: &Path,
    ) -> Result<Self> {
        debug!("Api::new() called");

        let _ = fs::exists(ods_socket_path).context(format!(
            "omnect device service socket file {ods_socket_path} does not exist"
        ));

        let _ = fs::exists(update_os_path).context(format!(
            "path {update_os_path} for os update does not exist"
        ));

        Ok(Self {
            ods_socket_path: ods_socket_path.into(),
            update_os_path: update_os_path.into(),
            centrifugo_client_token_hmac_secret_key: centrifugo_client_token_hmac_secret_key.into(),
            username: username.into(),
            password: password.into(),
            index_html: index_html.into(),
        })
    }

    pub async fn index(config: web::Data<Api>) -> actix_web::Result<NamedFile> {
        debug!("index() called");

        if let Err(e) = post_with_empty_body("/republish/v1", &config.ods_socket_path).await {
            error!("republish failed: {e:#}");
            return Err(actix_web::error::ErrorInternalServerError(
                "republish failed",
            ));
        }

        Ok(NamedFile::open(&config.index_html)?)
    }

    pub async fn factory_reset(
        body: web::Json<FactoryResetInput>,
        config: web::Data<Api>,
    ) -> impl Responder {
        debug!(
            "factory_reset() called with preserved keys {}",
            body.preserve.join(",")
        );

        let payload = FactoryResetPayload {
            mode: FactoryResetMode::Mode1,
            preserve: body.preserve.clone(),
        };

        match post_with_json_body("/factory-reset/v1", payload, &config.ods_socket_path).await {
            Ok(response) => response,
            Err(e) => {
                error!("factory_reset failed: {e:#}");
                HttpResponse::InternalServerError().body(format!("{e}"))
            }
        }
    }

    pub async fn reboot(config: web::Data<Api>) -> impl Responder {
        debug!("reboot() called");

        match post_with_empty_body("/reboot/v1", &config.ods_socket_path).await {
            Ok(response) => response,
            Err(e) => {
                error!("reboot failed: {e:#}");
                HttpResponse::InternalServerError().body(format!("{e}"))
            }
        }
    }

    pub async fn reload_network(config: web::Data<Api>) -> impl Responder {
        debug!("reload_network() called");

        match post_with_empty_body("/reload-network/v1", &config.ods_socket_path).await {
            Ok(response) => response,
            Err(e) => {
                error!("reload-network failed: {e:#}");
                HttpResponse::InternalServerError().body(format!("{e}"))
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
                    Err(e) => return HttpResponse::InternalServerError().body(format!("{e}")),
                }
            } else {
                error!("token: cannot create token");
            };
        } else {
            error!("token: missing secret key");
        };

        HttpResponse::InternalServerError().finish()
    }

    pub async fn logout(session: Session) -> impl Responder {
        debug!("logout() called");
        session.purge();
        HttpResponse::Ok()
    }

    pub async fn version() -> impl Responder {
        HttpResponse::Ok().body(env!("CARGO_PKG_VERSION"))
    }

    pub async fn save_file(
        MultipartForm(form): MultipartForm<UploadFormSingleFile>,
    ) -> impl Responder {
        debug!("save_file() called");

        let Some(filename) = form.file.file_name.clone() else {
            return HttpResponse::BadRequest().body("update file is missing");
        };

        let _ = Api::clear_data_folder().await;

        if let Err(e) =
            Api::persist_uploaded_file(form.file, &tmp_path!(&filename), &data_path!(&filename))
                .await
        {
            error!("save_file() failed: {e:#}");
            return HttpResponse::InternalServerError().body(format!("{e}"));
        }

        if let Err(e) = Api::set_file_permission(&data_path!(&filename)).await {
            error!("save_file() failed: {e:#}");
            return HttpResponse::InternalServerError().body(format!("{e}"));
        }

        HttpResponse::Ok().finish()
    }

    pub async fn load_update(
        mut body: web::Json<LoadUpdatePayload>,
        config: web::Data<Api>,
    ) -> impl Responder {
        debug!("load_update() called with path {:?}", body.update_file_path);

        body.update_file_path = Path::new(&config.update_os_path).join(&body.update_file_path);

        match post_with_json_body("/fwupdate/load/v1", body, &config.ods_socket_path).await {
            Ok(response) => response,
            Err(e) => {
                error!("load_update failed: {e:#}");
                HttpResponse::InternalServerError().body(format!("{e}"))
            }
        }
    }

    pub async fn run_update(
        body: web::Json<RunUpdatePayload>,
        config: web::Data<Api>,
    ) -> impl Responder {
        debug!(
            "run_update() called with validate_iothub_connection: {}",
            body.validate_iothub_connection
        );

        match post_with_json_body("/fwupdate/run/v1", body, &config.ods_socket_path).await {
            Ok(response) => response,
            Err(e) => {
                error!("run_update failed: {e:#}");
                HttpResponse::InternalServerError().body(format!("{e}"))
            }
        }
    }

    async fn clear_data_folder() -> Result<()> {
        debug!("clear_data_folder() called");
        for entry in fs::read_dir("/data")? {
            let entry = entry?;
            fs::remove_file(entry.path())?;
        }

        Ok(())
    }

    async fn persist_uploaded_file(
        tmp_file: TempFile,
        temp_path: &Path,
        data_path: &Path,
    ) -> Result<()> {
        debug!("persist_uploaded_file() called");

        tmp_file
            .file
            .persist(temp_path)
            .context("failed to persist tmp file")?;

        fs::copy(temp_path, data_path).context("failed to copy file to data dir")?;

        Ok(())
    }

    async fn set_file_permission(file_path: &Path) -> Result<()> {
        debug!("set_file_permission() called");

        let metadata = fs::metadata(file_path).context("failed to get file metadata")?;
        let mut perm = metadata.permissions();
        perm.set_mode(0o750);
        fs::set_permissions(file_path, perm).context("failed to set file permission")?;

        Ok(())
    }
}
