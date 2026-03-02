use serde::{Deserialize, Serialize};

pub const DEFAULT_REBOOT_TIMEOUT_SECS: u32 = 300;
pub const DEFAULT_FACTORY_RESET_TIMEOUT_SECS: u32 = 600;
pub const DEFAULT_FIRMWARE_UPDATE_TIMEOUT_SECS: u32 = 600;
pub const DEFAULT_NETWORK_ROLLBACK_TIMEOUT_SECS: u32 = 90;

/// User-configurable timeout settings for device operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TimeoutSettings {
    pub reboot_timeout_secs: u32,
    pub factory_reset_timeout_secs: u32,
    pub firmware_update_timeout_secs: u32,
    pub network_rollback_timeout_secs: u32,
}

impl Default for TimeoutSettings {
    fn default() -> Self {
        Self {
            reboot_timeout_secs: DEFAULT_REBOOT_TIMEOUT_SECS,
            factory_reset_timeout_secs: DEFAULT_FACTORY_RESET_TIMEOUT_SECS,
            firmware_update_timeout_secs: DEFAULT_FIRMWARE_UPDATE_TIMEOUT_SECS,
            network_rollback_timeout_secs: DEFAULT_NETWORK_ROLLBACK_TIMEOUT_SECS,
        }
    }
}
