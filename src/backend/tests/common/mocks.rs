use omnect_ui::{api::Api, keycloak_client::TokenClaims};

#[mockall_double::double]
use omnect_ui::{
    keycloak_client::SingleSignOnProvider, omnect_device_service_client::DeviceServiceClient,
};

/// Helper to create API instance with mocks
pub fn make_api(
    fleet_id: &'static str,
    claims: TokenClaims,
) -> Api<DeviceServiceClient, SingleSignOnProvider> {
    let device_client = mock_device_service_client_with_fleet_id(fleet_id);
    let sso_provider = mock_sso_provider_with_claims(claims);

    Api {
        ods_client: device_client,
        sso_provider,
    }
}

/// Creates a mock DeviceServiceClient with a fleet_id that returns the provided value
pub fn mock_device_service_client_with_fleet_id(
    fleet_id: &'static str,
) -> DeviceServiceClient {
    let mut mock = DeviceServiceClient::default();
    mock.expect_fleet_id()
        .returning(move || Box::pin(async move { Ok(fleet_id.to_string()) }));
    mock
}

/// Creates a mock DeviceServiceClient that returns an error for fleet_id
pub fn mock_device_service_client_with_error() -> DeviceServiceClient {
    let mut mock = DeviceServiceClient::default();
    mock.expect_fleet_id().returning(move || {
        Box::pin(async move { Err("Device service unavailable".to_string()) })
    });
    mock
}

/// Creates a mock SingleSignOnProvider that verifies tokens successfully with the provided claims
pub fn mock_sso_provider_with_claims(claims: TokenClaims) -> SingleSignOnProvider {
    let mut mock = SingleSignOnProvider::default();
    mock.expect_verify_token().returning(move |_| {
        let claims = claims.clone();
        Box::pin(async move { Ok(claims) })
    });
    mock
}

/// Creates a mock SingleSignOnProvider that returns an error for token verification
pub fn mock_sso_provider_with_error(error_msg: &'static str) -> SingleSignOnProvider {
    let mut mock = SingleSignOnProvider::default();
    mock.expect_verify_token()
        .returning(move |_| Box::pin(async move { Err(error_msg.to_string()) }));
    mock
}

/// Helper to create TokenClaims for testing
pub fn make_token_claims(role: &str, tenant: &str, fleets: Option<Vec<&str>>) -> TokenClaims {
    TokenClaims {
        roles: Some(vec![role.to_string()]),
        tenant_list: Some(vec![tenant.to_string()]),
        fleet_list: fleets.map(|fs| fs.into_iter().map(|f| f.to_string()).collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_device_service_client_with_fleet_id() {
        let mock = mock_device_service_client_with_fleet_id("test-fleet");
        let result = mock.fleet_id().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-fleet");
    }

    #[tokio::test]
    async fn test_mock_device_service_client_with_error() {
        let mock = mock_device_service_client_with_error();
        let result = mock.fleet_id().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_sso_provider_with_claims() {
        let claims = make_token_claims("FleetAdministrator", "test-tenant", None);
        let mock = mock_sso_provider_with_claims(claims.clone());
        let result = mock.verify_token("dummy-token").await;
        assert!(result.is_ok());
        let verified_claims = result.unwrap();
        assert_eq!(verified_claims.roles, claims.roles);
    }

    #[tokio::test]
    async fn test_mock_sso_provider_with_error() {
        let mock = mock_sso_provider_with_error("Invalid token");
        let result = mock.verify_token("dummy-token").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_make_token_claims() {
        let claims = make_token_claims("FleetOperator", "tenant1", Some(vec!["fleet1", "fleet2"]));
        assert_eq!(claims.roles, Some(vec!["FleetOperator".to_string()]));
        assert_eq!(claims.tenant_list, Some(vec!["tenant1".to_string()]));
        assert_eq!(
            claims.fleet_list,
            Some(vec!["fleet1".to_string(), "fleet2".to_string()])
        );
    }
}
