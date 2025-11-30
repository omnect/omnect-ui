use serde::{Deserialize, Serialize};
use std::fmt;

use crate::types::*;

/// Events that can happen in the app
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Event {
    // Initialization
    Initialize,

    // Authentication
    Login {
        password: String,
    },
    Logout,
    SetPassword {
        password: String,
    },
    UpdatePassword {
        current_password: String,
        password: String,
    },
    CheckRequiresPasswordSet,

    // Device actions
    Reboot,
    FactoryResetRequest {
        mode: String,
        preserve: Vec<String>,
    },
    ReloadNetwork,

    // Network configuration
    SetNetworkConfig {
        config: String,
    },

    // Update actions
    LoadUpdate {
        file_path: String,
    },
    RunUpdate {
        validate_iothub: bool,
    },

    // WebSocket subscriptions
    SubscribeToChannels,
    UnsubscribeFromChannels,

    // WebSocket updates (from Centrifugo)
    SystemInfoUpdated(SystemInfo),
    NetworkStatusUpdated(NetworkStatus),
    OnlineStatusUpdated(OnlineStatus),
    FactoryResetUpdated(FactoryReset),
    UpdateValidationStatusUpdated(UpdateValidationStatus),
    TimeoutsUpdated(Timeouts),

    // HTTP responses (internal events, skipped from serialization)
    #[serde(skip)]
    LoginResponse(Result<AuthToken, String>),
    #[serde(skip)]
    LogoutResponse(Result<(), String>),
    #[serde(skip)]
    SetPasswordResponse(Result<(), String>),
    #[serde(skip)]
    UpdatePasswordResponse(Result<(), String>),
    #[serde(skip)]
    CheckRequiresPasswordSetResponse(Result<bool, String>),
    #[serde(skip)]
    RebootResponse(Result<(), String>),
    #[serde(skip)]
    FactoryResetResponse(Result<(), String>),
    #[serde(skip)]
    ReloadNetworkResponse(Result<(), String>),
    #[serde(skip)]
    SetNetworkConfigResponse(Result<(), String>),
    #[serde(skip)]
    LoadUpdateResponse(Result<(), String>),
    #[serde(skip)]
    RunUpdateResponse(Result<(), String>),
    #[serde(skip)]
    HealthcheckResponse(Result<HealthcheckInfo, String>),

    // Connection state
    Connected,
    Disconnected,

    // UI actions
    ClearError,
    ClearSuccess,
}

/// Custom Debug implementation to redact sensitive data
impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Redact password fields
            Event::Login { .. } => f
                .debug_struct("Login")
                .field("password", &"<redacted>")
                .finish(),
            Event::SetPassword { .. } => f
                .debug_struct("SetPassword")
                .field("password", &"<redacted>")
                .finish(),
            Event::UpdatePassword { .. } => f
                .debug_struct("UpdatePassword")
                .field("current_password", &"<redacted>")
                .field("password", &"<redacted>")
                .finish(),
            Event::LoginResponse(result) => match result {
                Ok(_) => f
                    .debug_tuple("LoginResponse")
                    .field(&"Ok(<redacted token>)")
                    .finish(),
                Err(e) => f
                    .debug_tuple("LoginResponse")
                    .field(&format!("Err({e})"))
                    .finish(),
            },

            // All other events use default Debug
            Event::Initialize => write!(f, "Initialize"),
            Event::Logout => write!(f, "Logout"),
            Event::CheckRequiresPasswordSet => write!(f, "CheckRequiresPasswordSet"),
            Event::Reboot => write!(f, "Reboot"),
            Event::FactoryResetRequest { mode, preserve } => f
                .debug_struct("FactoryResetRequest")
                .field("mode", mode)
                .field("preserve", preserve)
                .finish(),
            Event::ReloadNetwork => write!(f, "ReloadNetwork"),
            Event::SetNetworkConfig { config } => f
                .debug_struct("SetNetworkConfig")
                .field("config", config)
                .finish(),
            Event::LoadUpdate { file_path } => f
                .debug_struct("LoadUpdate")
                .field("file_path", file_path)
                .finish(),
            Event::RunUpdate { validate_iothub } => f
                .debug_struct("RunUpdate")
                .field("validate_iothub", validate_iothub)
                .finish(),
            Event::SubscribeToChannels => write!(f, "SubscribeToChannels"),
            Event::UnsubscribeFromChannels => write!(f, "UnsubscribeFromChannels"),
            Event::SystemInfoUpdated(data) => {
                f.debug_tuple("SystemInfoUpdated").field(data).finish()
            }
            Event::NetworkStatusUpdated(data) => {
                f.debug_tuple("NetworkStatusUpdated").field(data).finish()
            }
            Event::OnlineStatusUpdated(data) => {
                f.debug_tuple("OnlineStatusUpdated").field(data).finish()
            }
            Event::FactoryResetUpdated(data) => {
                f.debug_tuple("FactoryResetUpdated").field(data).finish()
            }
            Event::UpdateValidationStatusUpdated(data) => f
                .debug_tuple("UpdateValidationStatusUpdated")
                .field(data)
                .finish(),
            Event::TimeoutsUpdated(data) => f.debug_tuple("TimeoutsUpdated").field(data).finish(),
            Event::LogoutResponse(result) => f.debug_tuple("LogoutResponse").field(result).finish(),
            Event::SetPasswordResponse(result) => {
                f.debug_tuple("SetPasswordResponse").field(result).finish()
            }
            Event::UpdatePasswordResponse(result) => f
                .debug_tuple("UpdatePasswordResponse")
                .field(result)
                .finish(),
            Event::CheckRequiresPasswordSetResponse(result) => f
                .debug_tuple("CheckRequiresPasswordSetResponse")
                .field(result)
                .finish(),
            Event::RebootResponse(result) => f.debug_tuple("RebootResponse").field(result).finish(),
            Event::FactoryResetResponse(result) => {
                f.debug_tuple("FactoryResetResponse").field(result).finish()
            }
            Event::ReloadNetworkResponse(result) => f
                .debug_tuple("ReloadNetworkResponse")
                .field(result)
                .finish(),
            Event::SetNetworkConfigResponse(result) => f
                .debug_tuple("SetNetworkConfigResponse")
                .field(result)
                .finish(),
            Event::LoadUpdateResponse(result) => {
                f.debug_tuple("LoadUpdateResponse").field(result).finish()
            }
            Event::RunUpdateResponse(result) => {
                f.debug_tuple("RunUpdateResponse").field(result).finish()
            }
            Event::HealthcheckResponse(result) => {
                f.debug_tuple("HealthcheckResponse").field(result).finish()
            }
            Event::Connected => write!(f, "Connected"),
            Event::Disconnected => write!(f, "Disconnected"),
            Event::ClearError => write!(f, "ClearError"),
            Event::ClearSuccess => write!(f, "ClearSuccess"),
        }
    }
}
