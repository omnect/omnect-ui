use actix_web::{App, body::MessageBody, dev::ServiceResponse, test, web};
use omnect_ui::api::Api;

#[mockall_double::double]
use omnect_ui::{
    keycloak_client::SingleSignOnProvider, omnect_device_service_client::DeviceServiceClient,
};

/// Creates a test app with the provided API instance
pub fn create_test_app(
    api: Api<DeviceServiceClient, SingleSignOnProvider>,
) -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new().app_data(web::Data::new(api))
}

/// Helper to create a basic POST request for testing
pub fn create_post_request(uri: &str, payload: &str) -> actix_web::test::TestRequest {
    test::TestRequest::post()
        .uri(uri)
        .insert_header(actix_web::http::header::ContentType::plaintext())
        .set_payload(payload.to_string())
}

/// Helper to create a GET request for testing
pub fn create_get_request(uri: &str) -> actix_web::test::TestRequest {
    test::TestRequest::get().uri(uri)
}

/// Helper to create an authenticated request with Bearer token
pub fn create_authenticated_request(
    uri: &str,
    method: actix_web::http::Method,
    token: &str,
) -> actix_web::test::TestRequest {
    let mut req = test::TestRequest::default().uri(uri).method(method);
    req = req.insert_header((
        actix_web::http::header::AUTHORIZATION,
        format!("Bearer {token}"),
    ));
    req
}

/// Helper to create a request with Basic authentication
pub fn create_basic_auth_request(
    uri: &str,
    method: actix_web::http::Method,
    credentials: &str,
) -> actix_web::test::TestRequest {
    let mut req = test::TestRequest::default().uri(uri).method(method);
    req = req.insert_header((
        actix_web::http::header::AUTHORIZATION,
        format!("Basic {credentials}"),
    ));
    req
}

/// Load fixture file content as string
pub fn load_fixture(path: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{path}"))
        .unwrap_or_else(|_| panic!("Failed to load fixture: {path}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::mocks;
    use omnect_ui::keycloak_client::TokenClaims;

    #[test]
    fn test_create_post_request() {
        let req = create_post_request("/test", "payload");
        assert_eq!(req.to_request().uri(), "/test");
        assert_eq!(req.to_request().method(), actix_web::http::Method::POST);
    }

    #[test]
    fn test_create_get_request() {
        let req = create_get_request("/test");
        assert_eq!(req.to_request().uri(), "/test");
        assert_eq!(req.to_request().method(), actix_web::http::Method::GET);
    }

    #[test]
    fn test_create_authenticated_request() {
        let req = create_authenticated_request("/test", actix_web::http::Method::GET, "token123");
        let req = req.to_request();
        assert_eq!(req.uri(), "/test");
        assert!(req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .is_some());
    }

    #[test]
    fn test_create_basic_auth_request() {
        let req = create_basic_auth_request("/test", actix_web::http::Method::POST, "dXNlcjpwYXNz");
        let req = req.to_request();
        assert_eq!(req.uri(), "/test");
        assert!(req
            .headers()
            .get(actix_web::http::header::AUTHORIZATION)
            .is_some());
    }

    #[test]
    fn test_create_test_app() {
        let claims = TokenClaims {
            roles: Some(vec!["test".to_string()]),
            tenant_list: None,
            fleet_list: None,
        };
        let device_client = mocks::mock_device_service_client_with_fleet_id("test-fleet");
        let sso_provider = mocks::mock_sso_provider_with_claims(claims);

        let api = Api {
            ods_client: device_client,
            sso_provider,
        };

        let _app = create_test_app(api);
    }
}
