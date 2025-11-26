use serde::{Deserialize, Serialize};

/// Credentials for user authentication
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginCredentials {
    pub password: String,
}

/// Authentication token response from backend
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthToken {
    pub token: String,
}

/// Request to set initial password (first-time setup)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetPasswordRequest {
    pub password: String,
}

/// Request to update existing password (requires old password)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdatePasswordRequest {
    pub current: String,
    pub new_password: String,
}
