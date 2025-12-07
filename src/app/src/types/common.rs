use serde::{Deserialize, Serialize};

/// Operating system information
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
}

/// System information from WebSocket
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    pub os: OsInfo,
    pub azure_sdk_version: String,
    pub omnect_device_service_version: String,
    pub boot_time: Option<String>,
}

/// Online status from WebSocket
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OnlineStatus {
    pub iothub: bool,
}

/// Duration type for timeouts
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Duration {
    pub nanos: u32,
    pub secs: u64,
}

/// Timeout configurations from WebSocket
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Timeouts {
    pub wait_online_timeout: Duration,
}

/// Overlay spinner state (UI state)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverlaySpinnerState {
    pub overlay: bool,
    pub title: String,
    pub text: Option<String>,
    pub timed_out: bool,
}
