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
    pub required: String,
    pub current: String,
    pub mismatch: bool,
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
    pub mode: u8,
    pub preserve: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoadUpdateRequest {
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunUpdateRequest {
    pub validate_iothub_connection: bool,
}

// Device Operation States (for reboot/factory reset reconnection)
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

// Network Change States (for IP change after network config)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkChangeState {
    Idle,
    ApplyingConfig {
        is_server_addr: bool,
        ip_changed: bool,
        new_ip: String,
        old_ip: String,
    },
    WaitingForNewIp {
        new_ip: String,
        attempt: u32,
    },
    NewIpReachable {
        new_ip: String,
    },
    NewIpTimeout {
        new_ip: String,
    },
}

impl Default for NetworkChangeState {
    fn default() -> Self {
        Self::Idle
    }
}

// Network Form State (for form editing without WebSocket interference)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkFormData {
    pub name: String,
    pub ip_address: String,
    pub dhcp: bool,
    pub prefix_len: u32,
    pub dns: Vec<String>,
    pub gateways: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkFormState {
    Idle,
    Editing {
        adapter_name: String,
        form_data: NetworkFormData,
    },
    Submitting {
        adapter_name: String,
        form_data: NetworkFormData,
    },
}

impl Default for NetworkFormState {
    fn default() -> Self {
        Self::Idle
    }
}

// Network Configuration Request (parsed from JSON)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConfigRequest {
    pub is_server_addr: bool,
    pub ip_changed: bool,
    pub name: String,
    pub dhcp: bool,
    pub ip: Option<String>,
    pub previous_ip: Option<String>,
    pub netmask: Option<u32>,
    pub gateway: Vec<String>,
    pub dns: Vec<String>,
}

// Overlay Spinner State (moved from Shell to Core for single source of truth)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverlaySpinnerState {
    pub overlay: bool,
    pub title: String,
    pub text: Option<String>,
    pub timed_out: bool,
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
