use serde::{Deserialize, Serialize};

/// Validation status for a device update
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateValidationStatus {
    pub status: String,
}

/// Request to load an update onto the device
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoadUpdateRequest {
    pub file_path: String,
}

/// Request to run/apply a loaded update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunUpdateRequest {
    pub validate_iothub: bool,
}
