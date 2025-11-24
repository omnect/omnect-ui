//! Authorization service
//!
//! Handles token validation and role-based access control independent of HTTP concerns.

use crate::{
    config::AppConfig, keycloak_client::SingleSignOnProvider,
    omnect_device_service_client::DeviceServiceClient,
};
use anyhow::{Result, bail, ensure};

/// Service for authorization operations
pub struct AuthorizationService;

impl AuthorizationService {
    /// Validate SSO token and check user claims for authorization
    ///
    /// Uses the tenant configuration from AppConfig.
    ///
    /// # Arguments
    /// * `single_sign_on` - Single sign-on provider for token verification
    /// * `service_client` - Device service client for fleet ID lookup
    /// * `token` - The authentication token to validate
    ///
    /// # Returns
    /// Result indicating success or authorization failure
    ///
    /// # Authorization Rules
    /// - User must have tenant in their tenant_list
    /// - FleetAdministrator role grants full access
    /// - FleetOperator role requires fleet_id in fleet_list
    pub async fn validate_token_and_claims<ServiceClient, SingleSignOn>(
        single_sign_on: &SingleSignOn,
        service_client: &ServiceClient,
        token: &str,
    ) -> Result<()>
    where
        ServiceClient: DeviceServiceClient,
        SingleSignOn: SingleSignOnProvider,
    {
        let claims = single_sign_on.verify_token(token).await?;
        let tenant = &AppConfig::get().tenant;

        // Validate tenant authorization
        let Some(tenant_list) = &claims.tenant_list else {
            bail!("failed to authorize user: no tenant list in token");
        };
        ensure!(
            tenant_list.contains(tenant),
            "failed to authorize user: insufficient permissions for tenant"
        );

        // Validate role-based authorization
        let Some(roles) = &claims.roles else {
            bail!("failed to authorize user: no roles in token");
        };

        // FleetAdministrator has full access
        if roles.iter().any(|r| r == "FleetAdministrator") {
            return Ok(());
        }

        // FleetOperator requires fleet validation
        if roles.iter().any(|r| r == "FleetOperator") {
            let Some(fleet_list) = &claims.fleet_list else {
                bail!("failed to authorize user: no fleet list in token");
            };
            let fleet_id = service_client.fleet_id().await?;
            ensure!(
                fleet_list.contains(&fleet_id),
                "failed to authorize user: insufficient permissions for fleet"
            );
            return Ok(());
        }

        bail!("failed to authorize user: insufficient role permissions")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keycloak_client::TokenClaims;

    #[cfg(feature = "mock")]
    use mockall_double::double;

    #[cfg(feature = "mock")]
    #[double]
    use crate::{
        keycloak_client::SingleSignOnProvider, omnect_device_service_client::DeviceServiceClient,
    };

    fn make_claims(role: &str, tenant: &str, fleets: Option<Vec<&str>>) -> TokenClaims {
        TokenClaims {
            roles: Some(vec![role.to_string()]),
            tenant_list: Some(vec![tenant.to_string()]),
            fleet_list: fleets.map(|fs| fs.into_iter().map(|f| f.to_string()).collect()),
        }
    }

    #[tokio::test]
    #[cfg(feature = "mock")]
    async fn test_fleet_administrator_with_valid_tenant() {
        let claims = make_claims("FleetAdministrator", "cp", None);
        let mut sso = SingleSignOnProvider::default();
        sso.expect_verify_token().returning(move |_| {
            let claims = claims.clone();
            Box::pin(async move { Ok(claims) })
        });

        let mut device_client = DeviceServiceClient::default();
        device_client
            .expect_fleet_id()
            .returning(|| Box::pin(async { Ok("fleet1".to_string()) }));

        let result =
            AuthorizationService::validate_token_and_claims(&sso, &device_client, "token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "mock")]
    async fn test_fleet_administrator_with_invalid_tenant() {
        let claims = make_claims("FleetAdministrator", "wrong-tenant", None);
        let mut sso = SingleSignOnProvider::default();
        sso.expect_verify_token()
            .returning(move |_| {
            let claims = claims.clone();
            Box::pin(async move { Ok(claims) })
        });

        let mut device_client = DeviceServiceClient::default();
        device_client
            .expect_fleet_id()
            .returning(|| Box::pin(async { Ok("fleet1".to_string()) }));

        let result =
            AuthorizationService::validate_token_and_claims(&sso, &device_client, "token").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("insufficient permissions for tenant"));
    }

    #[tokio::test]
    #[cfg(feature = "mock")]
    async fn test_fleet_operator_with_valid_fleet() {
        let claims = make_claims("FleetOperator", "cp", Some(vec!["fleet1", "fleet2"]));
        let mut sso = SingleSignOnProvider::default();
        sso.expect_verify_token()
            .returning(move |_| {
            let claims = claims.clone();
            Box::pin(async move { Ok(claims) })
        });

        let mut device_client = DeviceServiceClient::default();
        device_client
            .expect_fleet_id()
            .returning(|| Box::pin(async { Ok("fleet1".to_string()) }));

        let result =
            AuthorizationService::validate_token_and_claims(&sso, &device_client, "token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[cfg(feature = "mock")]
    async fn test_fleet_operator_with_invalid_fleet() {
        let claims = make_claims("FleetOperator", "cp", Some(vec!["fleet2", "fleet3"]));
        let mut sso = SingleSignOnProvider::default();
        sso.expect_verify_token()
            .returning(move |_| {
            let claims = claims.clone();
            Box::pin(async move { Ok(claims) })
        });

        let mut device_client = DeviceServiceClient::default();
        device_client
            .expect_fleet_id()
            .returning(|| Box::pin(async { Ok("fleet1".to_string()) }));

        let result =
            AuthorizationService::validate_token_and_claims(&sso, &device_client, "token").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("insufficient permissions for fleet"));
    }

    #[tokio::test]
    #[cfg(feature = "mock")]
    async fn test_fleet_operator_without_fleet_list() {
        let mut claims = make_claims("FleetOperator", "cp", None);
        claims.fleet_list = None;
        let mut sso = SingleSignOnProvider::default();
        sso.expect_verify_token()
            .returning(move |_| {
            let claims = claims.clone();
            Box::pin(async move { Ok(claims) })
        });

        let mut device_client = DeviceServiceClient::default();
        device_client
            .expect_fleet_id()
            .returning(|| Box::pin(async { Ok("fleet1".to_string()) }));

        let result =
            AuthorizationService::validate_token_and_claims(&sso, &device_client, "token").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no fleet list in token"));
    }

    #[tokio::test]
    #[cfg(feature = "mock")]
    async fn test_invalid_role() {
        let claims = make_claims("FleetObserver", "cp", Some(vec!["fleet1"]));
        let mut sso = SingleSignOnProvider::default();
        sso.expect_verify_token()
            .returning(move |_| {
            let claims = claims.clone();
            Box::pin(async move { Ok(claims) })
        });

        let mut device_client = DeviceServiceClient::default();
        device_client
            .expect_fleet_id()
            .returning(|| Box::pin(async { Ok("fleet1".to_string()) }));

        let result =
            AuthorizationService::validate_token_and_claims(&sso, &device_client, "token").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("insufficient role permissions"));
    }

    #[tokio::test]
    #[cfg(feature = "mock")]
    async fn test_missing_tenant_list() {
        let mut claims = make_claims("FleetAdministrator", "cp", None);
        claims.tenant_list = None;
        let mut sso = SingleSignOnProvider::default();
        sso.expect_verify_token()
            .returning(move |_| {
            let claims = claims.clone();
            Box::pin(async move { Ok(claims) })
        });

        let mut device_client = DeviceServiceClient::default();
        device_client
            .expect_fleet_id()
            .returning(|| Box::pin(async { Ok("fleet1".to_string()) }));

        let result =
            AuthorizationService::validate_token_and_claims(&sso, &device_client, "token").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no tenant list in token"));
    }

    #[tokio::test]
    #[cfg(feature = "mock")]
    async fn test_missing_roles() {
        let mut claims = make_claims("FleetAdministrator", "cp", None);
        claims.roles = None;
        let mut sso = SingleSignOnProvider::default();
        sso.expect_verify_token()
            .returning(move |_| {
            let claims = claims.clone();
            Box::pin(async move { Ok(claims) })
        });

        let mut device_client = DeviceServiceClient::default();
        device_client
            .expect_fleet_id()
            .returning(|| Box::pin(async { Ok("fleet1".to_string()) }));

        let result =
            AuthorizationService::validate_token_and_claims(&sso, &device_client, "token").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no roles in token"));
    }

    #[tokio::test]
    #[cfg(feature = "mock")]
    async fn test_sso_token_verification_error() {
        let mut sso = SingleSignOnProvider::default();
        sso.expect_verify_token()
            .returning(|_| Box::pin(async { Err(anyhow::anyhow!("Token verification failed")) }));

        let mut device_client = DeviceServiceClient::default();
        device_client
            .expect_fleet_id()
            .returning(|| Box::pin(async { Ok("fleet1".to_string()) }));

        let result =
            AuthorizationService::validate_token_and_claims(&sso, &device_client, "token").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Token verification failed"));
    }

    #[tokio::test]
    #[cfg(feature = "mock")]
    async fn test_device_service_fleet_id_error() {
        let claims = make_claims("FleetOperator", "cp", Some(vec!["fleet1"]));
        let mut sso = SingleSignOnProvider::default();
        sso.expect_verify_token()
            .returning(move |_| {
            let claims = claims.clone();
            Box::pin(async move { Ok(claims) })
        });

        let mut device_client = DeviceServiceClient::default();
        device_client.expect_fleet_id().returning(|| {
            Box::pin(async { Err(anyhow::anyhow!("Device service unavailable")) })
        });

        let result =
            AuthorizationService::validate_token_and_claims(&sso, &device_client, "token").await;
        assert!(result.is_err());
    }
}
