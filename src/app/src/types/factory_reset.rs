use serde::{Deserialize, Serialize};

/// Factory reset operation status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FactoryResetStatus {
    Unknown,
    ModeSupported,
    ModeUnsupported,
    BackupRestoreError,
    ConfigurationError,
}

impl Default for FactoryResetStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Result of factory reset operation
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryResetResult {
    pub status: FactoryResetStatus,
    pub context: Option<String>,
    pub error: String,
    pub paths: Vec<String>,
}

/// Factory reset state from WebSocket
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryReset {
    pub keys: Vec<String>,
    #[serde(default)]
    pub result: Option<FactoryResetResult>,
}

/// Request to initiate factory reset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryResetRequest {
    pub mode: u8,
    pub preserve: Vec<String>,
}
