pub mod token;

pub use token::TokenManager;

use crate::config::AppConfig;
use anyhow::{Result, bail};
use argon2::{Argon2, PasswordHash, PasswordVerifier};

pub fn validate_password(password: &str) -> Result<()> {
    if password.is_empty() {
        bail!("failed to validate password: empty");
    }

    let Ok(password_hash) =
        std::fs::read_to_string(AppConfig::get().paths.config_dir.join("password"))
    else {
        bail!("failed to read password file");
    };

    if password_hash.is_empty() {
        bail!("failed to validate password: hash is empty");
    }

    let Ok(parsed_hash) = PasswordHash::new(&password_hash) else {
        bail!("failed to parse password hash");
    };

    if let Err(e) = Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        bail!("failed to verify password: {e}");
    }

    Ok(())
}
