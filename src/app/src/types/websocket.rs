use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum WebSocketChannel {
    #[serde(rename = "OnlineStatusV1")]
    OnlineStatus,
    #[serde(rename = "SystemInfoV1")]
    SystemInfo,
    #[serde(rename = "TimeoutsV1")]
    Timeouts,
    #[serde(rename = "NetworkStatusV1")]
    NetworkStatus,
    #[serde(rename = "FactoryResetV1")]
    FactoryReset,
    #[serde(rename = "UpdateValidationStatusV1")]
    UpdateStatus,
}
