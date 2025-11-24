use crate::config::AppConfig;
use anyhow::{Context, Result};
use base64::{Engine, prelude::BASE64_STANDARD};
use jwt_simple::prelude::{RS256PublicKey, RSAPublicKeyLike};
#[cfg(feature = "mock")]
use mockall::automock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use trait_variant::make;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenClaims {
    pub roles: Option<Vec<String>>,
    pub tenant_list: Option<Vec<String>>,
    pub fleet_list: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct RealmInfo {
    public_key: String,
}

#[make(Send + Sync)]
#[cfg_attr(feature = "mock", automock)]
pub trait SingleSignOnProvider {
    async fn verify_token(&self, token: &str) -> anyhow::Result<TokenClaims>;
}

#[derive(Clone, Default)]
pub struct KeycloakProvider;

impl KeycloakProvider {
    pub fn create_frontend_config_file() -> Result<()> {
        use anyhow::Context;
        use std::io::Write;

        let mut config_file = std::fs::File::create(&AppConfig::get().paths.app_config_path)
            .context("failed to create frontend config file")?;

        config_file
            .write_all(
                format!(
                    "window.__APP_CONFIG__ = {{KEYCLOAK_URL:\"{}\"}};",
                    AppConfig::get().keycloak.url
                )
                .as_bytes(),
            )
            .context("failed to write frontend config file")
    }

    async fn realm_public_key(&self) -> Result<RS256PublicKey> {
        let client = Client::new();
        let resp = client
            .get(&AppConfig::get().keycloak.url)
            .send()
            .await
            .context("failed to fetch from url")?
            .json::<RealmInfo>()
            .await
            .context("failed to parse realm info")?;

        let decoded = BASE64_STANDARD
            .decode(resp.public_key.as_bytes())
            .context("failed to decode public key from base64")?;

        RS256PublicKey::from_der(&decoded).context("failed to parse public key from DER format")
    }
}

impl SingleSignOnProvider for KeycloakProvider {
    async fn verify_token(&self, token: &str) -> anyhow::Result<TokenClaims> {
        let pub_key = self.realm_public_key().await?;
        let claims = pub_key.verify_token::<TokenClaims>(token, None)?;
        Ok(claims.custom)
    }
}

// Note: Unit tests for KeycloakProvider are not included here because:
// - verify_token() requires mocking the HTTP client (reqwest) which is complex
// - verify_token() is already tested indirectly through AuthorizationService tests
// - verify_token() is tested in integration tests (tests/validate_portal_token.rs)
// - create_frontend_config_file() requires AppConfig setup with environment variables
// - The token verification logic itself is handled by jwt-simple library (well-tested)
//
// If direct unit tests are needed in the future, consider:
// - Using wiremock or similar to mock HTTP responses
// - Making realm_public_key() mockable via dependency injection
// - Creating integration tests with a test Keycloak instance
