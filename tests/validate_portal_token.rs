use actix_web::{App, http::header::ContentType, test, web};
use mockall::predicate::*;
use omnect_ui::api::{Api, KeycloakVerifier, TokenClaims};
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

struct MockOdsClient {
    fleet_id: String,
}

#[async_trait::async_trait]
impl omnect_ui::omnect_device_service_client::DeviceServiceClientTrait for MockOdsClient {
    async fn fleet_id(&self) -> anyhow::Result<String> {
        Ok(self.fleet_id.clone())
    }
    async fn republish(&self) -> anyhow::Result<()> {
        panic!("not implemented")
    }
    async fn version_info(
        &self,
    ) -> anyhow::Result<omnect_ui::omnect_device_service_client::VersionInfo> {
        panic!("not implemented")
    }
    async fn factory_reset(
        &self,
        _factory_reset: omnect_ui::omnect_device_service_client::FactoryReset,
    ) -> anyhow::Result<()> {
        panic!("not implemented")
    }
    async fn reboot(&self) -> anyhow::Result<()> {
        panic!("not implemented")
    }
    async fn reload_network(&self) -> anyhow::Result<()> {
        panic!("not implemented")
    }
    async fn load_update(
        &self,
        _load_update: omnect_ui::omnect_device_service_client::LoadUpdate,
    ) -> anyhow::Result<String> {
        panic!("not implemented")
    }
    async fn run_update(
        &self,
        _run_update: omnect_ui::omnect_device_service_client::RunUpdate,
    ) -> anyhow::Result<()> {
        panic!("not implemented")
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
