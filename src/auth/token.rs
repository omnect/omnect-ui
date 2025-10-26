use anyhow::Result;
use jwt_simple::prelude::*;
use std::sync::OnceLock;

const TOKEN_SUBJECT: &str = "omnect-ui";
const TOKEN_EXPIRE_HOURS: u64 = 2;

/// Centralized token management for session tokens
///
/// Handles creation and verification of JWT tokens used for:
/// - Session authentication
/// - Centrifugo WebSocket authentication
pub struct TokenManager {
    key: HS256Key,
    expire_hours: u64,
    subject: String,
}

impl TokenManager {
    /// Create a new TokenManager
    ///
    /// # Arguments
    /// * `secret` - Secret key for HMAC-SHA256 signing
    pub fn new(secret: &str) -> Self {
        let key = HS256Key::from_bytes(secret.as_bytes());
        Self {
            key,
            expire_hours: TOKEN_EXPIRE_HOURS,
            subject: TOKEN_SUBJECT.to_string(),
        }
    }

    /// Create a new token with the configured expiration and subject
    ///
    /// Returns a signed JWT token string
    pub fn create_token(&self) -> Result<String> {
        let claims =
            Claims::create(Duration::from_hours(self.expire_hours)).with_subject(&self.subject);

        self.key
            .authenticate(claims)
            .map_err(|e| anyhow::anyhow!("failed to create token: {}", e))
    }

    /// Verify a token and check if it's valid
    ///
    /// Validates:
    /// - Signature
    /// - Expiration (with 15 minute tolerance)
    /// - Max validity (token age)
    /// - Required subject claim
    ///
    /// Returns true if token is valid, false otherwise
    pub fn verify_token(&self, token: &str) -> bool {
        let options = VerificationOptions {
            accept_future: true,
            time_tolerance: Some(Duration::from_mins(15)),
            max_validity: Some(Duration::from_hours(self.expire_hours)),
            required_subject: Some(self.subject.clone()),
            ..Default::default()
        };

        self.key
            .verify_token::<NoCustomClaims>(token, Some(options))
            .is_ok()
    }
}

/// Get or create the global TokenManager instance
///
/// Uses the centrifugo client token from config
pub fn token_manager() -> &'static TokenManager {
    use crate::common::centrifugo_config;

    static TOKEN_MANAGER: OnceLock<TokenManager> = OnceLock::new();
    TOKEN_MANAGER.get_or_init(|| TokenManager::new(&centrifugo_config().client_token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_token() {
        let manager = TokenManager::new("test-secret");

        let token = manager.create_token().expect("should create token");
        assert!(!token.is_empty());

        assert!(manager.verify_token(&token));
    }

    #[test]
    fn test_verify_invalid_token() {
        let manager = TokenManager::new("test-secret");

        assert!(!manager.verify_token("invalid.token.here"));
        assert!(!manager.verify_token(""));
    }

    #[test]
    fn test_verify_token_wrong_secret() {
        let manager1 = TokenManager::new("secret1");
        let manager2 = TokenManager::new("secret2");

        let token = manager1.create_token().expect("should create token");

        // Token created with secret1 should not verify with secret2
        assert!(!manager2.verify_token(&token));
    }
}
