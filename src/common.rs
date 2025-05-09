use anyhow::{anyhow, bail, Context, Result};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use jwt_simple::prelude::{RS256PublicKey, RSAPublicKeyLike};
use reqwest::blocking::get;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize)]
pub struct RealmInfo {
    public_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TokenClaims {
    roles: Option<Vec<String>>,
    tenant_list: Option<Vec<String>>,
    fleet_list: Option<Vec<String>>,
}

macro_rules! config_path {
    ($filename:expr) => {{
        static CONFIG_PATH_DEFAULT: &'static str = "/data/config";
        Path::new(&std::env::var("CONFIG_PATH").unwrap_or(CONFIG_PATH_DEFAULT.to_string()))
            .join($filename)
    }};
}
pub(crate) use config_path;

pub fn validate_password(password: &str) -> Result<()> {
    if password.is_empty() {
        bail!("password is empty");
    }

    let password_file = config_path!("password");

    let Ok(password_hash) = std::fs::read_to_string(password_file) else {
        bail!("failed to read password file");
    };

    if password_hash.is_empty() {
        bail!("password hash is empty");
    }

    let Ok(parsed_hash) = PasswordHash::new(&password_hash) else {
        bail!("failed to parse password hash");
    };

    if let Err(e) = Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        bail!("password verification failed: {e}");
    }

    Ok(())
}

pub async fn validate_token_and_claims(
    token: &str,
    keycloak_public_key_url: &str,
    tenant: &String,
) -> Result<()> {
    let Ok(pub_key) = get_keycloak_realm_public_key(keycloak_public_key_url).await else {
        bail!("failed to get public key");
    };

    let Ok(claims) = pub_key.verify_token::<TokenClaims>(token, None) else {
        bail!("failed to verify token");
    };

    let Some(tenant_list) = &claims.custom.tenant_list else {
        bail!("user has no tenant list");
    };

    if !tenant_list.contains(tenant) {
        bail!("user has no permission to set password");
    }

    let Some(roles) = &claims.custom.roles else {
        bail!("user has no roles");
    };

    if roles.contains(&String::from("FleetAdministrator")) {
        return Ok(());
    }

    if roles.contains(&String::from("FleetObserver")) {
        bail!("user has no permission to set password");
    }

    if roles.contains(&String::from("FleetOperator")) {
        let Some(fleet_list) = &claims.custom.fleet_list else {
            bail!("user has no permission on this fleet");
        };

        //TODO: Check if fleet is available and compare with fleet from ods
        if !fleet_list.contains(&String::from("123")) {
            bail!("user has no permission on this fleet");
        } else {
            return Ok(());
        }
    }

    Err(anyhow!("user has no permission to set password"))
}

async fn get_keycloak_realm_public_key(keycloak_public_key_url: &str) -> Result<RS256PublicKey> {
    let resp = get(keycloak_public_key_url)
        .context("failed to fetch from url")?
        .json::<RealmInfo>()
        .context("failed to parse realm info")?;

    let base64_key = &resp.public_key;

    let mut pem = String::from("-----BEGIN PUBLIC KEY-----\n");
    for chunk in base64_key.as_bytes().chunks(64) {
        pem.push_str(&String::from_utf8_lossy(chunk));
        pem.push('\n');
    }
    pem.push_str("-----END PUBLIC KEY-----\n");

    let public_key = RS256PublicKey::from_pem(&pem).context("failed to create pem")?;

    Ok(public_key)
}
