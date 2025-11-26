use serde::{Deserialize, Serialize};

/// Status of factory reset operation
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

/// Result details for factory reset operation
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryResetResult {
    pub status: FactoryResetStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub error: String,
    pub paths: Vec<String>,
}

/// Complete factory reset state
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryReset {
    pub keys: Vec<String>,
    pub result: FactoryResetResult,
}

/// Request to trigger a factory reset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryResetRequest {
    pub mode: String,
    pub preserve: Vec<String>,
}
