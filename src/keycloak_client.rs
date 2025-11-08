use crate::{config::AppConfig, http_client::HttpClientFactory};
use anyhow::{Context, Result};
use base64::{Engine, prelude::BASE64_STANDARD};
use jwt_simple::prelude::{RS256PublicKey, RSAPublicKeyLike};
#[cfg(feature = "mock")]
use mockall::automock;
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

        let mut config_file =
            std::fs::File::create(AppConfig::get().paths.config_dir.join("app_config.js"))
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
        let client = HttpClientFactory::https_client();
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
