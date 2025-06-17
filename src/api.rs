use crate::{
    common::{centrifugo_config, config_path, validate_password},
    middleware::TOKEN_EXPIRE_HOURS,
    omnect_device_service_client::*,
};
use actix_files::NamedFile;
use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use anyhow::{Context, Result, anyhow, bail};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use jwt_simple::prelude::*;
use log::{debug, error};
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::Arc,
};

macro_rules! data_path {
    ($filename:expr) => {
        Path::new("/data/").join($filename)
    };
}

macro_rules! host_data_path {
    ($filename:expr) => {
        Path::new(&format!("/var/lib/{}/", env!("CARGO_PKG_NAME"))).join($filename)
    };
}

macro_rules! tmp_path {
    ($filename:expr) => {
        Path::new("/tmp/").join($filename)
    };
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenClaims {
    roles: Option<Vec<String>>,
    tenant_list: Option<Vec<String>>,
    fleet_list: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPasswordPayload {
    password: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePasswordPayload {
    current_password: String,
    password: String,
}

#[derive(MultipartForm)]
pub struct UploadFormSingleFile {
    file: TempFile,
}

pub trait KeycloakVerifier: Send + Sync {
    fn verify_token(&self, token: &str) -> anyhow::Result<TokenClaims>;
}

pub struct RealKeycloakVerifier;
impl KeycloakVerifier for RealKeycloakVerifier {
    fn verify_token(&self, token: &str) -> anyhow::Result<TokenClaims> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let pub_key = rt.block_on(crate::keycloak_client::realm_public_key())?;
        let claims = pub_key.verify_token::<TokenClaims>(token, None)?;
        Ok(claims.custom)
    }
}

pub trait DeviceServiceClientTrait: Send + Sync {
    fn fleet_id<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + 'a>>;
    fn republish<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;
    fn version_info<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = anyhow::Result<crate::omnect_device_service_client::VersionInfo>,
                > + Send
                + 'a,
        >,
    >;
    fn factory_reset<'a>(
        &'a self,
        factory_reset: crate::omnect_device_service_client::FactoryReset,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;
    fn reboot<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;
    fn reload_network<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;
    fn load_update<'a>(
        &'a self,
        load_update: crate::omnect_device_service_client::LoadUpdate,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + 'a>>;
    fn run_update<'a>(
        &'a self,
        run_update: crate::omnect_device_service_client::RunUpdate,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>;
}

impl DeviceServiceClientTrait for OmnectDeviceServiceClient {
    fn fleet_id<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + 'a>>
    {
        Box::pin(self.fleet_id())
    }
    fn republish<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(self.republish())
    }
    fn version_info<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = anyhow::Result<crate::omnect_device_service_client::VersionInfo>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(self.version_info())
    }
    fn factory_reset<'a>(
        &'a self,
        factory_reset: crate::omnect_device_service_client::FactoryReset,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(self.factory_reset(factory_reset))
    }
    fn reboot<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(self.reboot())
    }
    fn reload_network<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(self.reload_network())
    }
    fn load_update<'a>(
        &'a self,
        load_update: crate::omnect_device_service_client::LoadUpdate,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + 'a>>
    {
        Box::pin(self.load_update(load_update))
    }
    fn run_update<'a>(
        &'a self,
        run_update: crate::omnect_device_service_client::RunUpdate,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(self.run_update(run_update))
    }
}

#[derive(Clone)]
pub struct Api {
    pub ods_client: Arc<dyn DeviceServiceClientTrait>,
    pub keycloak: Arc<dyn KeycloakVerifier>,
    pub index_html: PathBuf,
    pub tenant: String,
}

impl Api {
    const UPDATE_FILE_NAME: &str = "update.tar";
    pub async fn new() -> Result<Self> {
        let index_html =
            std::fs::canonicalize("static/index.html").context("static/index.html not found")?;
        let tenant = std::env::var("TENANT").unwrap_or("cp".to_string());
        let ods_client = Arc::new(OmnectDeviceServiceClient::new(true).await?)
            as Arc<dyn DeviceServiceClientTrait>;
        let keycloak = Arc::new(RealKeycloakVerifier);
        Ok(Api {
            ods_client,
            keycloak,
            index_html,
            tenant,
        })
    }

    pub async fn index(api: web::Data<Api>) -> actix_web::Result<NamedFile> {
        debug!("index() called");

        if let Err(e) = api.ods_client.republish().await {
            error!("republish failed: {e:#}");
            return Err(actix_web::error::ErrorInternalServerError(
                "republish failed",
            ));
        }

        Ok(NamedFile::open(&api.index_html)?)
    }

    pub async fn config() -> actix_web::Result<NamedFile> {
        Ok(NamedFile::open(config_path!("app_config.js"))?)
    }

    pub async fn healthcheck(api: web::Data<Api>) -> impl Responder {
        debug!("healthcheck() called");

        match api.ods_client.version_info().await {
            Ok(info) if info.mismatch => HttpResponse::ServiceUnavailable().json(&info),
            Ok(info) => HttpResponse::Ok().json(&info),
            Err(e) => {
                error!("healthcheck: {e:#}");
                HttpResponse::InternalServerError().body(format!("{e}"))
            }
        }
    }

    pub async fn factory_reset(
        body: web::Json<FactoryReset>,
        api: web::Data<Api>,
        session: Session,
    ) -> impl Responder {
        debug!("factory_reset() called: {body:?}");

        match api.ods_client.factory_reset(body.into_inner()).await {
            Ok(_) => {
                session.purge();
                HttpResponse::Ok().finish()
            }
            Err(e) => {
                error!("factory_reset: {e:#}");
                HttpResponse::InternalServerError().body(format!("{e}"))
            }
        }
    }

    pub async fn reboot(api: web::Data<Api>) -> impl Responder {
        debug!("reboot() called");

        match api.ods_client.reboot().await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => {
                error!("reboot failed: {e:#}");
                HttpResponse::InternalServerError().body(format!("{e}"))
            }
        }
    }

    pub async fn reload_network(api: web::Data<Api>) -> impl Responder {
        debug!("reload_network() called");

        match api.ods_client.reload_network().await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => {
                error!("reload_network failed: {e:#}");
                HttpResponse::InternalServerError().body(format!("{e}"))
            }
        }
    }

    pub async fn token(session: Session) -> impl Responder {
        debug!("token() called");

        Api::session_token(session)
    }

    pub async fn logout(session: Session) -> impl Responder {
        debug!("logout() called");
        session.purge();
        HttpResponse::Ok().finish()
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

        let _ = Api::clear_data_folder();

        if let Err(e) = Api::persist_uploaded_file(
            form.file,
            &tmp_path!(&filename),
            &data_path!(&Api::UPDATE_FILE_NAME),
        ) {
            error!("save_file() failed: {e:#}");
            return HttpResponse::InternalServerError().body(format!("{e}"));
        }

        HttpResponse::Ok().finish()
    }

    pub async fn load_update(api: web::Data<Api>) -> impl Responder {
        debug!("load_update() called with path");

        match api
            .ods_client
            .load_update(LoadUpdate {
                update_file_path: host_data_path!(&Api::UPDATE_FILE_NAME)
                    .display()
                    .to_string(),
            })
            .await
        {
            Ok(data) => HttpResponse::Ok().body(data),
            Err(e) => {
                error!("load_update failed: {e:#}");
                HttpResponse::InternalServerError().body(format!("{e}"))
            }
        }
    }

    pub async fn run_update(body: web::Json<RunUpdate>, api: web::Data<Api>) -> impl Responder {
        debug!("run_update() called with validate_iothub_connection: {body:?}");

        match api.ods_client.run_update(body.into_inner()).await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => {
                error!("run_update failed: {e:#}");
                HttpResponse::InternalServerError().body(format!("{e}"))
            }
        }
    }

    pub async fn set_password(
        body: web::Json<SetPasswordPayload>,
        session: Session,
    ) -> impl Responder {
        debug!("set_password() called");

        if config_path!("password").exists() {
            return HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish();
        }

        if let Err(e) = Api::store_or_update_password(&body.password) {
            error!("set_password() failed: {e:#}");
            return HttpResponse::InternalServerError().body(format!("{:#}", e));
        }

        Api::session_token(session)
    }

    pub async fn update_password(
        body: web::Json<UpdatePasswordPayload>,
        session: Session,
    ) -> impl Responder {
        debug!("update_password() called");

        if let Err(e) = validate_password(&body.current_password) {
            error!("update_password() failed: {e:#}");
            return HttpResponse::BadRequest().body("current password is not correct");
        }

        if let Err(e) = Api::store_or_update_password(&body.password) {
            error!("update_password() failed: {e:#}");
            return HttpResponse::InternalServerError().body(format!("{:#}", e));
        }

        session.purge();
        HttpResponse::Ok().finish()
    }

    pub async fn require_set_password() -> impl Responder {
        debug!("require_set_password() called");

        if !config_path!("password").exists() {
            return HttpResponse::Created()
                .append_header(("Location", "/set-password"))
                .finish();
        }

        HttpResponse::Ok().finish()
    }

    pub async fn validate_portal_token(body: String, api: web::Data<Api>) -> impl Responder {
        debug!("validate_portal_token() called");
        if let Err(e) = api.validate_token_and_claims(&body).await {
            error!("validate_portal_token() failed: {e:#}");
            return HttpResponse::Unauthorized().finish();
        }
        HttpResponse::Ok().finish()
    }

    async fn validate_token_and_claims(&self, token: &str) -> Result<()> {
        let claims = self.keycloak.verify_token(token)?;
        let Some(tenant_list) = &claims.tenant_list else {
            bail!("user has no tenant list");
        };
        if !tenant_list.contains(&self.tenant) {
            bail!("user has no permission to set password");
        }
        let Some(roles) = &claims.roles else {
            bail!("user has no roles");
        };
        if roles.contains(&String::from("FleetAdministrator")) {
            return Ok(());
        }
        if roles.contains(&String::from("FleetOperator")) {
            let Some(fleet_list) = &claims.fleet_list else {
                bail!("user has no permission on this fleet");
            };
            let fleet_id = self.ods_client.fleet_id().await?;
            if !fleet_list.contains(&fleet_id) {
                bail!("user has no permission on this fleet");
            }
            return Ok(());
        }
        bail!("user has no permission to set password")
    }

    fn clear_data_folder() -> Result<()> {
        debug!("clear_data_folder() called");
        for entry in fs::read_dir("/data")? {
            let entry = entry?;
            if entry.path().is_file() {
                fs::remove_file(entry.path())?;
            }
        }

        Ok(())
    }

    fn persist_uploaded_file(tmp_file: TempFile, temp_path: &Path, data_path: &Path) -> Result<()> {
        debug!("persist_uploaded_file() called");

        tmp_file
            .file
            .persist(temp_path)
            .context("failed to persist tmp file")?;

        fs::copy(temp_path, data_path).context("failed to copy file to data dir")?;

        let metadata = fs::metadata(data_path).context("failed to get file metadata")?;
        let mut perm = metadata.permissions();
        perm.set_mode(0o750);
        fs::set_permissions(data_path, perm).context("failed to set file permission")
    }

    fn hash_password(password: &str) -> Result<String> {
        debug!("hash_password() called");

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(hash) => Ok(hash.to_string()),
            Err(e) => Err(anyhow!(e).context("failed to hash password")),
        }
    }

    fn store_or_update_password(password: &str) -> Result<()> {
        debug!("store_or_update_password() called");

        let password_file = config_path!("password");
        let hash = Api::hash_password(password)?;
        let mut file = File::create(&password_file).context("failed to create password file")?;

        file.write_all(hash.as_bytes())
            .context("failed to write password file")
    }

    fn session_token(session: Session) -> HttpResponse {
        let key = HS256Key::from_bytes(centrifugo_config().client_token.as_bytes());
        let claims =
            Claims::create(Duration::from_hours(TOKEN_EXPIRE_HOURS)).with_subject("omnect-ui");

        let Ok(token) = key.authenticate(claims) else {
            error!("failed to create token");
            return HttpResponse::InternalServerError().body("failed to create token");
        };

        if session.insert("token", &token).is_err() {
            error!("failed to insert token into session");
            return HttpResponse::InternalServerError().body("failed to insert token into session");
        }

        HttpResponse::Ok().body(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, http::header::ContentType, test};
    use std::sync::Arc;

    struct MockKeycloakVerifier {
        claims: TokenClaims,
    }
    impl KeycloakVerifier for MockKeycloakVerifier {
        fn verify_token(&self, _token: &str) -> anyhow::Result<TokenClaims> {
            Ok(self.claims.clone())
        }
    }

    #[derive(Clone)]
    struct MockOdsClient {
        fleet_id: String,
    }
    impl DeviceServiceClientTrait for MockOdsClient {
        fn fleet_id<'a>(
            &'a self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + 'a>>
        {
            let value = self.fleet_id.clone();
            Box::pin(async move { Ok(value) })
        }
        fn republish<'a>(
            &'a self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>
        {
            Box::pin(async { Ok(()) })
        }
        fn version_info<'a>(
            &'a self,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = anyhow::Result<crate::omnect_device_service_client::VersionInfo>,
                    > + Send
                    + 'a,
            >,
        > {
            Box::pin(async { Err(anyhow::anyhow!("not implemented")) })
        }
        fn factory_reset<'a>(
            &'a self,
            _factory_reset: crate::omnect_device_service_client::FactoryReset,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>
        {
            Box::pin(async { Ok(()) })
        }
        fn reboot<'a>(
            &'a self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>
        {
            Box::pin(async { Ok(()) })
        }
        fn reload_network<'a>(
            &'a self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>
        {
            Box::pin(async { Ok(()) })
        }
        fn load_update<'a>(
            &'a self,
            _load_update: crate::omnect_device_service_client::LoadUpdate,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + 'a>>
        {
            Box::pin(async { Ok("mocked".to_string()) })
        }
        fn run_update<'a>(
            &'a self,
            _run_update: crate::omnect_device_service_client::RunUpdate,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>>
        {
            Box::pin(async { Ok(()) })
        }
    }

    async fn call_validate(api: Api) -> actix_web::dev::ServiceResponse {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(api))
                .route("/validate", web::post().to(Api::validate_portal_token)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/validate")
            .insert_header(ContentType::plaintext())
            .set_payload("dummy")
            .to_request();
        test::call_service(&app, req).await
    }

    #[tokio::test]
    async fn validate_portal_token_fleet_admin_should_succeed() {
        let claims = TokenClaims {
            roles: Some(vec!["FleetAdministrator".to_string()]),
            tenant_list: Some(vec!["cp".to_string()]),
            fleet_list: None,
        };
        let api = Api {
            ods_client: Arc::new(MockOdsClient {
                fleet_id: "Fleet1".to_string(),
            }),
            keycloak: Arc::new(MockKeycloakVerifier { claims }),
            index_html: PathBuf::from("/dev/null"),
            tenant: "cp".to_string(),
        };
        let resp = call_validate(api).await;
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn validate_portal_token_fleet_admin_invalid_tenant_should_fail() {
        let claims = TokenClaims {
            roles: Some(vec!["FleetAdministrator".to_string()]),
            tenant_list: Some(vec!["othertenant".to_string()]),
            fleet_list: None,
        };
        let api = Api {
            ods_client: Arc::new(MockOdsClient {
                fleet_id: "Fleet1".to_string(),
            }),
            keycloak: Arc::new(MockKeycloakVerifier { claims }),
            index_html: PathBuf::from("/dev/null"),
            tenant: "cp".to_string(),
        };
        let resp = call_validate(api).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn validate_portal_token_fleet_operator_should_succeed() {
        let claims = TokenClaims {
            roles: Some(vec!["FleetOperator".to_string()]),
            tenant_list: Some(vec!["cp".to_string()]),
            fleet_list: Some(vec!["Fleet1".to_string(), "Fleet2".to_string()]),
        };
        let api = Api {
            ods_client: Arc::new(MockOdsClient {
                fleet_id: "Fleet1".to_string(),
            }),
            keycloak: Arc::new(MockKeycloakVerifier { claims }),
            index_html: PathBuf::from("/dev/null"),
            tenant: "cp".to_string(),
        };
        let resp = call_validate(api).await;
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn validate_portal_token_fleet_operator_wrong_fleet_should_fail() {
        let claims = TokenClaims {
            roles: Some(vec!["FleetOperator".to_string()]),
            tenant_list: Some(vec!["cp".to_string()]),
            fleet_list: Some(vec!["Fleet2".to_string()]),
        };
        let api = Api {
            ods_client: Arc::new(MockOdsClient {
                fleet_id: "Fleet1".to_string(),
            }),
            keycloak: Arc::new(MockKeycloakVerifier { claims }),
            index_html: PathBuf::from("/dev/null"),
            tenant: "cp".to_string(),
        };
        let resp = call_validate(api).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn validate_portal_token_fleet_observer_should_fail() {
        let claims = TokenClaims {
            roles: Some(vec!["FleetObserver".to_string()]),
            tenant_list: Some(vec!["cp".to_string()]),
            fleet_list: None,
        };
        let api = Api {
            ods_client: Arc::new(MockOdsClient {
                fleet_id: "Fleet1".to_string(),
            }),
            keycloak: Arc::new(MockKeycloakVerifier { claims }),
            index_html: PathBuf::from("/dev/null"),
            tenant: "cp".to_string(),
        };
        let resp = call_validate(api).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }
}
