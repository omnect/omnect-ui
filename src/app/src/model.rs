use serde::{Deserialize, Serialize};

use crate::types::*;

/// Application Model - the complete state
/// Also serves as the ViewModel when serialized (auth_token is excluded)
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Model {
    // Device state
    pub system_info: Option<SystemInfo>,
    pub network_status: Option<NetworkStatus>,
    pub online_status: Option<OnlineStatus>,
    pub factory_reset: Option<FactoryReset>,
    pub update_validation_status: Option<UpdateValidationStatus>,
    pub update_manifest: Option<UpdateManifest>,
    pub timeouts: Option<Timeouts>,
    pub healthcheck: Option<HealthcheckInfo>,

    // Authentication state
    /// Internal auth token. Currently serialized to the view model as a workaround
    /// for a `shared_types` deserialization misalignment. See TODO in README.md.
    pub auth_token: Option<String>,
    pub is_authenticated: bool,
    pub requires_password_set: bool,

    // UI state
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,

    // WebSocket state
    pub is_connected: bool,

    // Device operation state (reboot/factory reset reconnection)
    pub device_operation_state: DeviceOperationState,
    pub reconnection_attempt: u32,
    pub reconnection_timeout_seconds: u32,
    pub device_went_offline: bool,

    // Network change state (IP change detection and polling)
    pub network_change_state: NetworkChangeState,

    // Network form state (editing without WebSocket interference)
    pub network_form_state: NetworkFormState,

    // Firmware upload state
    pub firmware_upload_state: UploadState,

    // Overlay spinner state (moved from Shell for single source of truth)
    pub overlay_spinner: OverlaySpinnerState,
}

impl Model {
    /// Invalidate the current session (logout)
    pub fn invalidate_session(&mut self) {
        self.is_authenticated = false;
        self.auth_token = None;
    }

    /// Start a loading operation (sets is_loading=true, clears error)
    pub fn start_loading(&mut self) {
        self.is_loading = true;
        self.error_message = None;
    }

    /// Stop loading and clear error
    pub fn stop_loading(&mut self) {
        self.is_loading = false;
        self.error_message = None;
    }

    /// Set an error message and stop loading
    pub fn set_error(&mut self, error: String) {
        self.is_loading = false;
        self.error_message = Some(error);
    }

    /// Clear the error message without affecting the loading state.
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }
}
