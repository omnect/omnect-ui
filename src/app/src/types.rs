use serde::{Deserialize, Serialize};

// System Information
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    pub os: OsInfo,
    pub azure_sdk_version: String,
    pub omnect_device_service_version: String,
    pub boot_time: Option<String>,
}

// Network Status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpAddress {
    pub addr: String,
    pub dhcp: bool,
    pub prefix_len: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct InternetProtocol {
    pub addrs: Vec<IpAddress>,
    pub dns: Vec<String>,
    pub gateways: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceNetwork {
    pub ipv4: InternetProtocol,
    pub mac: String,
    pub name: String,
    pub online: bool,
    #[serde(default)]
    pub file: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkStatus {
    pub network_status: Vec<DeviceNetwork>,
}

// Online Status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OnlineStatus {
    pub iothub: bool,
}

// Factory Reset
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryResetResult {
    pub status: FactoryResetStatus,
    pub context: Option<String>,
    pub error: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryReset {
    pub keys: Vec<String>,
    #[serde(default)]
    pub result: Option<FactoryResetResult>,
}

// Update Validation Status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateValidationStatus {
    pub status: String,
}

// Timeouts
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Duration {
    pub nanos: u32,
    pub secs: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Timeouts {
    pub wait_online_timeout: Duration,
}

// Health Check
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionInfo {
    pub version: String,
    pub git_sha: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthcheckInfo {
    pub version_info: VersionInfo,
    pub update_validation_status: UpdateValidationStatus,
}

// Authentication
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginCredentials {
    pub password: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthToken {
    pub token: String,
}

// Request types for API calls
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetPasswordRequest {
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePasswordRequest {
    pub current_password: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FactoryResetRequest {
    pub mode: String,
    pub preserve: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoadUpdateRequest {
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunUpdateRequest {
    pub validate_iothub: bool,
}

// Update Manifest
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateId {
    pub provider: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Compatibility {
    pub manufacturer: String,
    pub model: String,
    pub compatibilityid: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManifest {
    pub update_id: UpdateId,
    pub is_deployable: bool,
    pub compatibility: Vec<Compatibility>,
    pub created_date_time: String,
    pub manifest_version: String,
}
