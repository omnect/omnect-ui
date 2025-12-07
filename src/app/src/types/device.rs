use serde::{Deserialize, Serialize};

/// State of long-running device operations (reboot, factory reset, update)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceOperationState {
    Idle,
    Rebooting,
    FactoryResetting,
    Updating,
    WaitingReconnection { operation: String, attempt: u32 },
    ReconnectionFailed { operation: String, reason: String },
    ReconnectionSuccessful { operation: String },
}

impl Default for DeviceOperationState {
    fn default() -> Self {
        Self::Idle
    }
}

impl DeviceOperationState {
    pub fn operation_name(&self) -> String {
        match self {
            Self::Rebooting => "Reboot".to_string(),
            Self::FactoryResetting => "Factory Reset".to_string(),
            Self::Updating => "Update".to_string(),
            Self::WaitingReconnection { operation, .. }
            | Self::ReconnectionFailed { operation, .. }
            | Self::ReconnectionSuccessful { operation } => operation.clone(),
            Self::Idle => "Unknown".to_string(),
        }
    }
}
