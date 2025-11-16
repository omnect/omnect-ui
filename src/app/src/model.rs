use serde::{Deserialize, Serialize};

use crate::types::*;

/// Application Model - the complete state
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Model {
    // Device state
    pub system_info: Option<SystemInfo>,
    pub network_status: Option<NetworkStatus>,
    pub online_status: Option<OnlineStatus>,
    pub factory_reset: Option<FactoryReset>,
    pub update_validation_status: Option<UpdateValidationStatus>,
    pub timeouts: Option<Timeouts>,
    pub healthcheck: Option<HealthcheckInfo>,

    // Authentication state
    pub auth_token: Option<String>,
    pub is_authenticated: bool,
    pub requires_password_set: bool,

    // UI state
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,

    // WebSocket state
    pub is_connected: bool,
}

/// View Model - what the UI needs to render
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ViewModel {
    pub system_info: Option<SystemInfo>,
    pub network_status: Option<NetworkStatus>,
    pub online_status: Option<OnlineStatus>,
    pub factory_reset: Option<FactoryReset>,
    pub update_validation_status: Option<UpdateValidationStatus>,
    pub timeouts: Option<Timeouts>,
    pub healthcheck: Option<HealthcheckInfo>,

    pub is_authenticated: bool,
    pub requires_password_set: bool,

    pub is_loading: bool,
    pub error_message: Option<String>,
    pub success_message: Option<String>,
    pub is_connected: bool,
}
