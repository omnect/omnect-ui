use anyhow::{Context, Result};
use base64::{Engine, prelude::BASE64_STANDARD};
use jwt_simple::prelude::{RS256PublicKey, RSAPublicKeyLike};
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};

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

macro_rules! keycloak_url {
    () => {{
        std::env::var("KEYCLOAK_URL")
            .unwrap_or("https://keycloak.omnect.conplement.cloud/realms/cp-prod".to_string())
    }};
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

pub fn config() -> String {
    let keycloak_url = &keycloak_url!();
    format!("window.__APP_CONFIG__ = {{KEYCLOAK_URL:\"{keycloak_url}\"}};")
}

pub async fn realm_public_key() -> Result<RS256PublicKey> {
    let resp = get(keycloak_url!())
        .context("failed to fetch from url")?
        .json::<RealmInfo>()
        .context("failed to parse realm info")?;

    RS256PublicKey::from_der(&BASE64_STANDARD.decode(resp.public_key.as_bytes()).unwrap())
        .context("failed to decode public key")
}
