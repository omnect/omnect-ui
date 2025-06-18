use actix_web::{App, http::header::ContentType, test, web};
use omnect_ui::api::{Api, DeviceServiceClientTrait, KeycloakVerifier, TokenClaims};
use std::path::PathBuf;
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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }
    fn version_info<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = anyhow::Result<omnect_ui::omnect_device_service_client::VersionInfo>,
                > + Send
                + 'a,
        >,
    > {
        Box::pin(async { Err(anyhow::anyhow!("not implemented")) })
    }
    fn factory_reset<'a>(
        &'a self,
        _factory_reset: omnect_ui::omnect_device_service_client::FactoryReset,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }
    fn reboot<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }
    fn reload_network<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }
    fn load_update<'a>(
        &'a self,
        _load_update: omnect_ui::omnect_device_service_client::LoadUpdate,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + 'a>>
    {
        Box::pin(async { Ok("mocked".to_string()) })
    }
    fn run_update<'a>(
        &'a self,
        _run_update: omnect_ui::omnect_device_service_client::RunUpdate,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
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
        tenant_list: Some(vec!["invalid_tenant".to_string()]),
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
async fn validate_portal_token_fleet_operator_invalid_fleet_should_fail() {
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
