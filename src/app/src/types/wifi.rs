use serde::{Deserialize, Serialize};

/// WiFi service availability info returned by the backend
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WifiAvailability {
    pub available: bool,
    pub interface_name: Option<String>,
}

/// A WiFi network discovered during scanning
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WifiNetwork {
    pub ssid: String,
    pub mac: String,
    pub channel: u16,
    pub rssi: i16,
}

/// A saved WiFi network from wpa_supplicant
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WifiSavedNetwork {
    pub ssid: String,
    pub flags: String,
}

/// WiFi connection status from the service
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WifiConnectionStatus {
    pub state: WifiConnectionState,
    pub ssid: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WifiScanState {
    #[default]
    Idle,
    Scanning,
    Finished,
    Error(String),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WifiConnectionState {
    #[default]
    Idle,
    Connecting,
    Connected,
    Failed(String),
}

/// Top-level WiFi state machine exposed in the ViewModel
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WifiState {
    #[default]
    Unavailable,
    Ready {
        interface_name: String,
        status: WifiConnectionStatus,
        scan_state: WifiScanState,
        scan_results: Vec<WifiNetwork>,
        saved_networks: Vec<WifiSavedNetwork>,
        scan_poll_attempt: u32,
        connect_poll_attempt: u32,
    },
}
