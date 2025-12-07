use serde::{Deserialize, Serialize};
use std::fmt;

use crate::types::*;

/// Authentication events
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum AuthEvent {
    Login { password: String },
    Logout,
    SetPassword { password: String },
    UpdatePassword {
        current_password: String,
        password: String,
    },
    CheckRequiresPasswordSet,
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
}

/// Device operation events
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum DeviceEvent {
    Reboot,
    FactoryResetRequest {
        mode: String,
        preserve: Vec<String>,
    },
    ReloadNetwork,
    SetNetworkConfig {
        config: String,
    },
    NetworkFormStartEdit {
        adapter_name: String,
    },
    NetworkFormUpdate {
        form_data: String,
    },
    NetworkFormReset {
        adapter_name: String,
    },
    LoadUpdate {
        file_path: String,
    },
    RunUpdate {
        validate_iothub_connection: bool,
    },
    ReconnectionCheckTick,
    ReconnectionTimeout,
    NewIpCheckTick,
    NewIpCheckTimeout,
    #[serde(skip)]
    RebootResponse(Result<(), String>),
    #[serde(skip)]
    FactoryResetResponse(Result<(), String>),
    #[serde(skip)]
    ReloadNetworkResponse(Result<(), String>),
    #[serde(skip)]
    SetNetworkConfigResponse(Result<(), String>),
    #[serde(skip)]
    LoadUpdateResponse(Result<UpdateManifest, String>),
    #[serde(skip)]
    RunUpdateResponse(Result<(), String>),
    #[serde(skip)]
    HealthcheckResponse(Result<HealthcheckInfo, String>),
}

/// WebSocket/Centrifugo events
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum WebSocketEvent {
    SubscribeToChannels,
    UnsubscribeFromChannels,
    SystemInfoUpdated(SystemInfo),
    NetworkStatusUpdated(NetworkStatus),
    OnlineStatusUpdated(OnlineStatus),
    FactoryResetUpdated(FactoryReset),
    UpdateValidationStatusUpdated(UpdateValidationStatus),
    TimeoutsUpdated(Timeouts),
    Connected,
    Disconnected,
}

/// UI action events
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum UiEvent {
    ClearError,
    ClearSuccess,
}

/// Main event enum - wraps domain events
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Initialize,
    Auth(AuthEvent),
    Device(DeviceEvent),
    WebSocket(WebSocketEvent),
    Ui(UiEvent),
}

/// Custom Debug implementation for AuthEvent to redact sensitive data
impl fmt::Debug for AuthEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthEvent::Login { .. } => f
                .debug_struct("Login")
                .field("password", &"<redacted>")
                .finish(),
            AuthEvent::SetPassword { .. } => f
                .debug_struct("SetPassword")
                .field("password", &"<redacted>")
                .finish(),
            AuthEvent::UpdatePassword { .. } => f
                .debug_struct("UpdatePassword")
                .field("current_password", &"<redacted>")
                .field("password", &"<redacted>")
                .finish(),
            AuthEvent::LoginResponse(result) => match result {
                Ok(_) => f
                    .debug_tuple("LoginResponse")
                    .field(&"Ok(<redacted token>)")
                    .finish(),
                Err(e) => f
                    .debug_tuple("LoginResponse")
                    .field(&format!("Err({e})"))
                    .finish(),
            },
            AuthEvent::Logout => write!(f, "Logout"),
            AuthEvent::CheckRequiresPasswordSet => write!(f, "CheckRequiresPasswordSet"),
            AuthEvent::LogoutResponse(r) => f.debug_tuple("LogoutResponse").field(r).finish(),
            AuthEvent::SetPasswordResponse(r) => {
                f.debug_tuple("SetPasswordResponse").field(r).finish()
            }
            AuthEvent::UpdatePasswordResponse(r) => f
                .debug_tuple("UpdatePasswordResponse")
                .field(r)
                .finish(),
            AuthEvent::CheckRequiresPasswordSetResponse(r) => f
                .debug_tuple("CheckRequiresPasswordSetResponse")
                .field(r)
                .finish(),
        }
    }
}

/// Custom Debug implementation for DeviceEvent
impl fmt::Debug for DeviceEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceEvent::Reboot => write!(f, "Reboot"),
            DeviceEvent::FactoryResetRequest { mode, preserve } => f
                .debug_struct("FactoryResetRequest")
                .field("mode", mode)
                .field("preserve", preserve)
                .finish(),
            DeviceEvent::ReloadNetwork => write!(f, "ReloadNetwork"),
            DeviceEvent::SetNetworkConfig { config } => f
                .debug_struct("SetNetworkConfig")
                .field("config", config)
                .finish(),
            DeviceEvent::NetworkFormStartEdit { adapter_name } => f
                .debug_struct("NetworkFormStartEdit")
                .field("adapter_name", adapter_name)
                .finish(),
            DeviceEvent::NetworkFormUpdate { form_data } => f
                .debug_struct("NetworkFormUpdate")
                .field("form_data", form_data)
                .finish(),
            DeviceEvent::NetworkFormReset { adapter_name } => f
                .debug_struct("NetworkFormReset")
                .field("adapter_name", adapter_name)
                .finish(),
            DeviceEvent::LoadUpdate { file_path } => f
                .debug_struct("LoadUpdate")
                .field("file_path", file_path)
                .finish(),
            DeviceEvent::RunUpdate {
                validate_iothub_connection,
            } => f
                .debug_struct("RunUpdate")
                .field("validate_iothub_connection", validate_iothub_connection)
                .finish(),
            DeviceEvent::ReconnectionCheckTick => write!(f, "ReconnectionCheckTick"),
            DeviceEvent::ReconnectionTimeout => write!(f, "ReconnectionTimeout"),
            DeviceEvent::NewIpCheckTick => write!(f, "NewIpCheckTick"),
            DeviceEvent::NewIpCheckTimeout => write!(f, "NewIpCheckTimeout"),
            DeviceEvent::RebootResponse(r) => f.debug_tuple("RebootResponse").field(r).finish(),
            DeviceEvent::FactoryResetResponse(r) => {
                f.debug_tuple("FactoryResetResponse").field(r).finish()
            }
            DeviceEvent::ReloadNetworkResponse(r) => {
                f.debug_tuple("ReloadNetworkResponse").field(r).finish()
            }
            DeviceEvent::SetNetworkConfigResponse(r) => f
                .debug_tuple("SetNetworkConfigResponse")
                .field(r)
                .finish(),
            DeviceEvent::LoadUpdateResponse(r) => {
                f.debug_tuple("LoadUpdateResponse").field(r).finish()
            }
            DeviceEvent::RunUpdateResponse(r) => {
                f.debug_tuple("RunUpdateResponse").field(r).finish()
            }
            DeviceEvent::HealthcheckResponse(r) => {
                f.debug_tuple("HealthcheckResponse").field(r).finish()
            }
        }
    }
}

/// Custom Debug implementation for WebSocketEvent
impl fmt::Debug for WebSocketEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebSocketEvent::SubscribeToChannels => write!(f, "SubscribeToChannels"),
            WebSocketEvent::UnsubscribeFromChannels => write!(f, "UnsubscribeFromChannels"),
            WebSocketEvent::SystemInfoUpdated(d) => {
                f.debug_tuple("SystemInfoUpdated").field(d).finish()
            }
            WebSocketEvent::NetworkStatusUpdated(d) => {
                f.debug_tuple("NetworkStatusUpdated").field(d).finish()
            }
            WebSocketEvent::OnlineStatusUpdated(d) => {
                f.debug_tuple("OnlineStatusUpdated").field(d).finish()
            }
            WebSocketEvent::FactoryResetUpdated(d) => {
                f.debug_tuple("FactoryResetUpdated").field(d).finish()
            }
            WebSocketEvent::UpdateValidationStatusUpdated(d) => f
                .debug_tuple("UpdateValidationStatusUpdated")
                .field(d)
                .finish(),
            WebSocketEvent::TimeoutsUpdated(d) => {
                f.debug_tuple("TimeoutsUpdated").field(d).finish()
            }
            WebSocketEvent::Connected => write!(f, "Connected"),
            WebSocketEvent::Disconnected => write!(f, "Disconnected"),
        }
    }
}

/// Custom Debug implementation for UiEvent
impl fmt::Debug for UiEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiEvent::ClearError => write!(f, "ClearError"),
            UiEvent::ClearSuccess => write!(f, "ClearSuccess"),
        }
    }
}

/// Custom Debug implementation for Event
impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::Initialize => write!(f, "Initialize"),
            Event::Auth(e) => write!(f, "Auth({e:?})"),
            Event::Device(e) => write!(f, "Device({e:?})"),
            Event::WebSocket(e) => write!(f, "WebSocket({e:?})"),
            Event::Ui(e) => write!(f, "Ui({e:?})"),
        }
    }
}
