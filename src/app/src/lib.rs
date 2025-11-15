pub mod capabilities;
pub mod types;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

use crux_core::{render::render, Command};
use serde::{Deserialize, Serialize};

// Using deprecated Capabilities API for backwards compatibility
// Will migrate to full Command API when shell integration is complete
#[allow(deprecated)]
use crux_http::Http;

use crate::capabilities::centrifugo::Centrifugo;
use crate::types::*;

// Application Model - the complete state
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

// View Model - what the UI needs to render
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

// Events that can happen in the app
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Event {
    // Initialization
    Initialize,

    // Authentication
    Login {
        username: String,
        password: String,
    },
    Logout,
    SetPassword {
        password: String,
    },
    UpdatePassword {
        current: String,
        new_password: String,
    },
    CheckRequiresPasswordSet,

    // Device actions
    Reboot,
    FactoryResetRequest {
        mode: String,
        preserve: Vec<String>,
    },
    ReloadNetwork,

    // Network configuration
    SetNetworkConfig {
        config: String,
    },

    // Update actions
    LoadUpdate {
        file_path: String,
    },
    RunUpdate {
        validate_iothub: bool,
    },

    // WebSocket subscriptions
    SubscribeToChannels,
    UnsubscribeFromChannels,

    // WebSocket updates (from Centrifugo)
    SystemInfoUpdated(SystemInfo),
    NetworkStatusUpdated(NetworkStatus),
    OnlineStatusUpdated(OnlineStatus),
    FactoryResetUpdated(FactoryReset),
    UpdateValidationStatusUpdated(UpdateValidationStatus),
    TimeoutsUpdated(Timeouts),

    // HTTP responses (internal events, skipped from serialization)
    #[serde(skip)]
    LoginResponse(Result<AuthToken, String>),
    #[serde(skip)]
    LogoutResponse(Result<(), String>),
    #[serde(skip)]
    SetPasswordResponse(Result<(), String>),
    #[serde(skip)]
    UpdatePasswordResponse(Result<(), String>),
    #[serde(skip)]
    CheckRequiresPasswordSetResponse(Result<bool, String>),
    #[serde(skip)]
    RebootResponse(Result<(), String>),
    #[serde(skip)]
    FactoryResetResponse(Result<(), String>),
    #[serde(skip)]
    ReloadNetworkResponse(Result<(), String>),
    #[serde(skip)]
    SetNetworkConfigResponse(Result<(), String>),
    #[serde(skip)]
    LoadUpdateResponse(Result<(), String>),
    #[serde(skip)]
    RunUpdateResponse(Result<(), String>),
    #[serde(skip)]
    HealthcheckResponse(Result<HealthcheckInfo, String>),

    // Connection state
    Connected,
    Disconnected,

    // UI actions
    ClearError,
    ClearSuccess,
}

// Capabilities - side effects the app can perform
#[allow(deprecated)]
#[cfg_attr(feature = "typegen", derive(crux_core::macros::Export))]
#[derive(crux_core::macros::Effect)]
pub struct Capabilities {
    pub render: crux_core::render::Render<Event>,
    pub http: Http<Event>,
    pub centrifugo: Centrifugo<Event>,
}

// The Core application
#[derive(Default)]
pub struct App;

impl crux_core::App for App {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Capabilities = Capabilities;
    type Effect = Effect;

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
        _caps: &Self::Capabilities,
    ) -> Command<Effect, Event> {
        match event {
            Event::Initialize => {
                model.is_loading = true;
                render()
            }

            // Authentication events
            Event::Login {
                username: _,
                password: _,
            } => {
                model.is_loading = true;
                model.error_message = None;
                // TODO: HTTP request will be implemented when integrating with shell
                // let _credentials = LoginCredentials { username, password };
                render()
            }

            Event::LoginResponse(result) => {
                model.is_loading = false;
                match result {
                    Ok(auth) => {
                        model.auth_token = Some(auth.token);
                        model.is_authenticated = true;
                        model.error_message = None;
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                        model.is_authenticated = false;
                    }
                }
                render()
            }

            Event::Logout => {
                model.is_loading = true;
                render()
            }

            Event::LogoutResponse(result) => {
                model.is_loading = false;
                match result {
                    Ok(()) => {
                        model.auth_token = None;
                        model.is_authenticated = false;
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                    }
                }
                render()
            }

            Event::SetPassword { password: _ } => {
                model.is_loading = true;
                render()
            }

            Event::SetPasswordResponse(result) => {
                model.is_loading = false;
                match result {
                    Ok(()) => {
                        model.requires_password_set = false;
                        model.success_message = Some("Password set successfully".to_string());
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                    }
                }
                render()
            }

            Event::UpdatePassword {
                current: _,
                new_password: _,
            } => {
                model.is_loading = true;
                render()
            }

            Event::UpdatePasswordResponse(result) => {
                model.is_loading = false;
                match result {
                    Ok(()) => {
                        model.success_message = Some("Password updated successfully".to_string());
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                    }
                }
                render()
            }

            Event::CheckRequiresPasswordSet => {
                model.is_loading = true;
                render()
            }

            Event::CheckRequiresPasswordSetResponse(result) => {
                model.is_loading = false;
                match result {
                    Ok(requires) => {
                        model.requires_password_set = requires;
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                    }
                }
                render()
            }

            // Device actions
            Event::Reboot => {
                model.is_loading = true;
                render()
            }

            Event::RebootResponse(result) => {
                model.is_loading = false;
                match result {
                    Ok(()) => {
                        model.success_message = Some("Reboot initiated".to_string());
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                    }
                }
                render()
            }

            Event::FactoryResetRequest {
                mode: _,
                preserve: _,
            } => {
                model.is_loading = true;
                render()
            }

            Event::FactoryResetResponse(result) => {
                model.is_loading = false;
                match result {
                    Ok(()) => {
                        model.success_message = Some("Factory reset initiated".to_string());
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                    }
                }
                render()
            }

            Event::ReloadNetwork => {
                model.is_loading = true;
                render()
            }

            Event::ReloadNetworkResponse(result) => {
                model.is_loading = false;
                match result {
                    Ok(()) => {
                        model.success_message = Some("Network reloaded".to_string());
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                    }
                }
                render()
            }

            // Network configuration
            Event::SetNetworkConfig { config: _ } => {
                model.is_loading = true;
                render()
            }

            Event::SetNetworkConfigResponse(result) => {
                model.is_loading = false;
                match result {
                    Ok(()) => {
                        model.success_message = Some("Network configuration updated".to_string());
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                    }
                }
                render()
            }

            // Update actions
            Event::LoadUpdate { file_path: _ } => {
                model.is_loading = true;
                render()
            }

            Event::LoadUpdateResponse(result) => {
                model.is_loading = false;
                match result {
                    Ok(()) => {
                        model.success_message = Some("Update loaded".to_string());
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                    }
                }
                render()
            }

            Event::RunUpdate { validate_iothub: _ } => {
                model.is_loading = true;
                render()
            }

            Event::RunUpdateResponse(result) => {
                model.is_loading = false;
                match result {
                    Ok(()) => {
                        model.success_message = Some("Update started".to_string());
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                    }
                }
                render()
            }

            Event::HealthcheckResponse(result) => {
                match result {
                    Ok(info) => {
                        model.healthcheck = Some(info);
                    }
                    Err(e) => {
                        model.error_message = Some(e);
                    }
                }
                render()
            }

            // WebSocket subscriptions
            Event::SubscribeToChannels => {
                // TODO: Centrifugo commands will be implemented
                render()
            }

            Event::UnsubscribeFromChannels => {
                // TODO: Centrifugo commands will be implemented
                render()
            }

            // WebSocket updates
            Event::SystemInfoUpdated(info) => {
                model.system_info = Some(info);
                render()
            }

            Event::NetworkStatusUpdated(status) => {
                model.network_status = Some(status);
                render()
            }

            Event::OnlineStatusUpdated(status) => {
                model.online_status = Some(status);
                render()
            }

            Event::FactoryResetUpdated(reset) => {
                model.factory_reset = Some(reset);
                render()
            }

            Event::UpdateValidationStatusUpdated(status) => {
                model.update_validation_status = Some(status);
                render()
            }

            Event::TimeoutsUpdated(timeouts) => {
                model.timeouts = Some(timeouts);
                render()
            }

            // Connection state
            Event::Connected => {
                model.is_connected = true;
                render()
            }

            Event::Disconnected => {
                model.is_connected = false;
                render()
            }

            // UI actions
            Event::ClearError => {
                model.error_message = None;
                render()
            }

            Event::ClearSuccess => {
                model.success_message = None;
                render()
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            system_info: model.system_info.clone(),
            network_status: model.network_status.clone(),
            online_status: model.online_status.clone(),
            factory_reset: model.factory_reset.clone(),
            update_validation_status: model.update_validation_status.clone(),
            timeouts: model.timeouts.clone(),
            healthcheck: model.healthcheck.clone(),

            is_authenticated: model.is_authenticated,
            requires_password_set: model.requires_password_set,

            is_loading: model.is_loading,
            error_message: model.error_message.clone(),
            success_message: model.success_message.clone(),
            is_connected: model.is_connected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crux_core::testing::AppTester;

    #[test]
    fn test_login_sets_loading() {
        let app = AppTester::<App>::default();
        let mut model = Model::default();

        let _command = app.update(
            Event::Login {
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            &mut model,
        );

        assert!(model.is_loading);
    }

    #[test]
    fn test_system_info_updated() {
        let app = AppTester::<App>::default();
        let mut model = Model::default();

        let info = SystemInfo {
            os: OsInfo {
                name: "Linux".to_string(),
                version: "5.10".to_string(),
            },
            azure_sdk_version: "1.0".to_string(),
            omnect_device_service_version: "2.0".to_string(),
            boot_time: Some("2024-01-01".to_string()),
        };

        let _command = app.update(Event::SystemInfoUpdated(info.clone()), &mut model);

        assert_eq!(model.system_info, Some(info));
    }

    #[test]
    fn test_clear_error() {
        let app = AppTester::<App>::default();
        let mut model = Model {
            error_message: Some("Some error".to_string()),
            ..Default::default()
        };

        let _command = app.update(Event::ClearError, &mut model);

        assert_eq!(model.error_message, None);
    }
}
