use actix_web::{HttpResponse, ResponseError, body::BoxBody, http::StatusCode};
use std::fmt;

/// Application-level errors with proper HTTP status code mapping
#[derive(Debug)]
pub enum OmnectUiError {
    /// Authentication or authorization failed
    Authentication(String),

    /// External service (device service, Keycloak) unavailable
    ServiceUnavailable(String),

    /// Certificate-related errors (generation, validation, reading)
    Certificate(String),

    /// Configuration errors (missing/invalid env vars, paths)
    Configuration(String),

    /// Validation errors for user input
    ValidationError { field: String, reason: String },

    /// Internal server errors (catch-all for unexpected errors)
    Internal(anyhow::Error),
}

impl fmt::Display for OmnectUiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OmnectUiError::Authentication(msg) => write!(f, "Authentication failed: {}", msg),
            OmnectUiError::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
            OmnectUiError::Certificate(msg) => write!(f, "Certificate error: {}", msg),
            OmnectUiError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            OmnectUiError::ValidationError { field, reason } => {
                write!(f, "Validation error in '{}': {}", field, reason)
            }
            OmnectUiError::Internal(err) => write!(f, "Internal error: {:#}", err),
        }
    }
}

impl std::error::Error for OmnectUiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OmnectUiError::Internal(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl ResponseError for OmnectUiError {
    fn status_code(&self) -> StatusCode {
        match self {
            OmnectUiError::Authentication(_) => StatusCode::UNAUTHORIZED,
            OmnectUiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            OmnectUiError::Certificate(_) => StatusCode::BAD_REQUEST,
            OmnectUiError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            OmnectUiError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            OmnectUiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

impl From<anyhow::Error> for OmnectUiError {
    fn from(err: anyhow::Error) -> Self {
        OmnectUiError::Internal(err)
    }
}

/// Extension trait for adding context to errors with specific error types
pub trait ErrorContext<T> {
    /// Add authentication context to error
    fn auth_context(self, msg: impl Into<String>) -> Result<T, OmnectUiError>;

    /// Add service unavailable context to error
    fn service_context(self, msg: impl Into<String>) -> Result<T, OmnectUiError>;

    /// Add certificate context to error
    fn cert_context(self, msg: impl Into<String>) -> Result<T, OmnectUiError>;

    /// Add configuration context to error
    fn config_context(self, msg: impl Into<String>) -> Result<T, OmnectUiError>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn auth_context(self, msg: impl Into<String>) -> Result<T, OmnectUiError> {
        self.map_err(|e| OmnectUiError::Authentication(format!("{}: {}", msg.into(), e)))
    }

    fn service_context(self, msg: impl Into<String>) -> Result<T, OmnectUiError> {
        self.map_err(|e| OmnectUiError::ServiceUnavailable(format!("{}: {}", msg.into(), e)))
    }

    fn cert_context(self, msg: impl Into<String>) -> Result<T, OmnectUiError> {
        self.map_err(|e| OmnectUiError::Certificate(format!("{}: {}", msg.into(), e)))
    }

    fn config_context(self, msg: impl Into<String>) -> Result<T, OmnectUiError> {
        self.map_err(|e| OmnectUiError::Configuration(format!("{}: {}", msg.into(), e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            OmnectUiError::Authentication("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            OmnectUiError::ServiceUnavailable("test".to_string()).status_code(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(
            OmnectUiError::Certificate("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            OmnectUiError::ValidationError {
                field: "test".to_string(),
                reason: "invalid".to_string()
            }
            .status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_error_context_trait() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));

        let err = result.auth_context("login failed").unwrap_err();
        assert!(matches!(err, OmnectUiError::Authentication(_)));
        assert!(err.to_string().contains("login failed"));
        assert!(err.to_string().contains("file not found"));
    }
}
