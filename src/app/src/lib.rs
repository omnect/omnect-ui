pub mod capabilities;
pub mod types;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

use crux_core::{render::render, Command};
use serde::{Deserialize, Serialize};

// Using deprecated Capabilities API for Http (kept for Effect enum generation)
#[allow(deprecated)]
use crux_http::Http;

// Re-export the Centrifugo Operation and Output types from the original capability module
// These types are shared between the deprecated and new Command API
pub use crate::capabilities::centrifugo::{CentrifugoOperation, CentrifugoOutput};
use crate::types::*;

// Re-export crux_http Result type for convenience
pub use crux_http::Result as HttpResult;

// API base URL - empty string means relative URLs (shell will use current origin)
// For absolute URLs, set to something like "http://localhost:8000"
const API_BASE_URL: &str = "http://localhost:8000";

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

    // Centrifugo responses (internal events)
    #[serde(skip)]
    CentrifugoResponse(CentrifugoOutput),

    // UI actions
    ClearError,
    ClearSuccess,
}

// Capabilities - side effects the app can perform
// Note: We keep the old deprecated capabilities in this struct ONLY for Effect enum generation.
// The #[derive(crux_core::macros::Effect)] macro generates the Effect enum with proper
// From<Request<Operation>> implementations based on what's declared here.
// Actual usage goes through the Command-based APIs (HttpCmd, CentrifugoCmd).
#[allow(deprecated)]
#[cfg_attr(feature = "typegen", derive(crux_core::macros::Export))]
#[derive(crux_core::macros::Effect)]
pub struct Capabilities {
    pub render: crux_core::render::Render<Event>,
    pub http: Http<Event>,
    pub centrifugo: crate::capabilities::centrifugo::Centrifugo<Event>,
}

// Type aliases for the Command-based APIs
// Defined after Capabilities to have access to the generated Effect enum
type CentrifugoCmd = crate::capabilities::centrifugo_command::Centrifugo<Effect, Event>;
type HttpCmd = crux_http::command::Http<Effect, Event>;

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
            Event::Login { username, password } => {
                model.is_loading = true;
                model.error_message = None;
                let credentials = LoginCredentials { username, password };
                Command::all([
                    render(),
                    HttpCmd::post(format!("{}/api/token/login", API_BASE_URL))
                        .header("Content-Type", "application/json")
                        .body_json(&credentials)
                        .expect(
                            "Failed to serialize login credentials - this should never happen for valid data",
                        )
                        .expect_json::<AuthToken>()
                        .build()
                        .then_send(|result| match result {
                            Ok(mut response) => match response.take_body() {
                                Some(token) => Event::LoginResponse(Ok(token)),
                                None => {
                                    Event::LoginResponse(Err("Empty response body".to_string()))
                                }
                            },
                            Err(e) => Event::LoginResponse(Err(e.to_string())),
                        }),
                ])
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
                if let Some(token) = &model.auth_token {
                    Command::all([
                        render(),
                        HttpCmd::post(format!("{}/api/token/logout", API_BASE_URL))
                            .header("Authorization", format!("Bearer {token}"))
                            .build()
                            .then_send(|result| match result {
                                Ok(response) => {
                                    if response.status().is_success() {
                                        Event::LogoutResponse(Ok(()))
                                    } else {
                                        Event::LogoutResponse(Err(format!(
                                            "Logout failed: HTTP {}",
                                            response.status()
                                        )))
                                    }
                                }
                                Err(e) => Event::LogoutResponse(Err(e.to_string())),
                            }),
                    ])
                } else {
                    render()
                }
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

            Event::SetPassword { password } => {
                model.is_loading = true;
                #[derive(Serialize)]
                struct SetPasswordRequest {
                    password: String,
                }
                Command::all([
                    render(),
                    HttpCmd::post(format!("{}/api/token/set-password", API_BASE_URL))
                        .header("Content-Type", "application/json")
                        .body_json(&SetPasswordRequest { password })
                        .expect("Failed to serialize password request")
                        .build()
                        .then_send(|result| match result {
                            Ok(response) => {
                                if response.status().is_success() {
                                    Event::SetPasswordResponse(Ok(()))
                                } else {
                                    Event::SetPasswordResponse(Err(format!(
                                        "Set password failed: HTTP {}",
                                        response.status()
                                    )))
                                }
                            }
                            Err(e) => Event::SetPasswordResponse(Err(e.to_string())),
                        }),
                ])
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
                current,
                new_password,
            } => {
                model.is_loading = true;
                #[derive(Serialize)]
                struct UpdatePasswordRequest {
                    current: String,
                    new_password: String,
                }
                if let Some(token) = &model.auth_token {
                    Command::all([
                        render(),
                        HttpCmd::post(format!("{}/api/token/update-password", API_BASE_URL))
                            .header("Authorization", format!("Bearer {token}"))
                            .header("Content-Type", "application/json")
                            .body_json(&UpdatePasswordRequest {
                                current,
                                new_password,
                            })
                            .expect("Failed to serialize password update request")
                            .build()
                            .then_send(|result| match result {
                                Ok(response) => {
                                    if response.status().is_success() {
                                        Event::UpdatePasswordResponse(Ok(()))
                                    } else {
                                        Event::UpdatePasswordResponse(Err(format!(
                                            "Update password failed: HTTP {}",
                                            response.status()
                                        )))
                                    }
                                }
                                Err(e) => Event::UpdatePasswordResponse(Err(e.to_string())),
                            }),
                    ])
                } else {
                    render()
                }
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
                Command::all([
                    render(),
                    HttpCmd::get(format!("{}/api/token/requires-password-set", API_BASE_URL))
                        .expect_json::<bool>()
                        .build()
                        .then_send(|result| match result {
                            Ok(mut response) => match response.take_body() {
                                Some(requires) => {
                                    Event::CheckRequiresPasswordSetResponse(Ok(requires))
                                }
                                None => Event::CheckRequiresPasswordSetResponse(Err(
                                    "Empty response body".to_string(),
                                )),
                            },
                            Err(e) => Event::CheckRequiresPasswordSetResponse(Err(e.to_string())),
                        }),
                ])
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
                if let Some(token) = &model.auth_token {
                    Command::all([
                        render(),
                        HttpCmd::post(format!("{}/api/device/reboot", API_BASE_URL))
                            .header("Authorization", format!("Bearer {token}"))
                            .build()
                            .then_send(|result| match result {
                                Ok(response) => {
                                    if response.status().is_success() {
                                        Event::RebootResponse(Ok(()))
                                    } else {
                                        Event::RebootResponse(Err(format!(
                                            "Reboot failed: HTTP {}",
                                            response.status()
                                        )))
                                    }
                                }
                                Err(e) => Event::RebootResponse(Err(e.to_string())),
                            }),
                    ])
                } else {
                    render()
                }
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

            Event::FactoryResetRequest { mode, preserve } => {
                model.is_loading = true;
                #[derive(Serialize)]
                struct FactoryResetRequest {
                    mode: String,
                    preserve: Vec<String>,
                }
                if let Some(token) = &model.auth_token {
                    Command::all([
                        render(),
                        HttpCmd::post(format!("{}/api/device/factory-reset", API_BASE_URL))
                            .header("Authorization", format!("Bearer {token}"))
                            .header("Content-Type", "application/json")
                            .body_json(&FactoryResetRequest { mode, preserve })
                            .expect("Failed to serialize factory reset request")
                            .build()
                            .then_send(|result| match result {
                                Ok(response) => {
                                    if response.status().is_success() {
                                        Event::FactoryResetResponse(Ok(()))
                                    } else {
                                        Event::FactoryResetResponse(Err(format!(
                                            "Factory reset failed: HTTP {}",
                                            response.status()
                                        )))
                                    }
                                }
                                Err(e) => Event::FactoryResetResponse(Err(e.to_string())),
                            }),
                    ])
                } else {
                    render()
                }
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
                if let Some(token) = &model.auth_token {
                    Command::all([
                        render(),
                        HttpCmd::post(format!("{}/api/device/reload-network", API_BASE_URL))
                            .header("Authorization", format!("Bearer {token}"))
                            .build()
                            .then_send(|result| match result {
                                Ok(response) => {
                                    if response.status().is_success() {
                                        Event::ReloadNetworkResponse(Ok(()))
                                    } else {
                                        Event::ReloadNetworkResponse(Err(format!(
                                            "Reload network failed: HTTP {}",
                                            response.status()
                                        )))
                                    }
                                }
                                Err(e) => Event::ReloadNetworkResponse(Err(e.to_string())),
                            }),
                    ])
                } else {
                    render()
                }
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
            Event::SetNetworkConfig { config } => {
                model.is_loading = true;
                if let Some(token) = &model.auth_token {
                    Command::all([
                        render(),
                        HttpCmd::post(format!("{}/api/device/network", API_BASE_URL))
                            .header("Authorization", format!("Bearer {token}"))
                            .header("Content-Type", "application/json")
                            .body_string(config)
                            .build()
                            .then_send(|result| match result {
                                Ok(response) => {
                                    if response.status().is_success() {
                                        Event::SetNetworkConfigResponse(Ok(()))
                                    } else {
                                        Event::SetNetworkConfigResponse(Err(format!(
                                            "Set network config failed: HTTP {}",
                                            response.status()
                                        )))
                                    }
                                }
                                Err(e) => Event::SetNetworkConfigResponse(Err(e.to_string())),
                            }),
                    ])
                } else {
                    render()
                }
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
            Event::LoadUpdate { file_path } => {
                model.is_loading = true;
                #[derive(Serialize)]
                struct LoadUpdateRequest {
                    file_path: String,
                }
                if let Some(token) = &model.auth_token {
                    Command::all([
                        render(),
                        HttpCmd::post(format!("{}/api/update/load", API_BASE_URL))
                            .header("Authorization", format!("Bearer {token}"))
                            .header("Content-Type", "application/json")
                            .body_json(&LoadUpdateRequest { file_path })
                            .expect("Failed to serialize load update request")
                            .build()
                            .then_send(|result| match result {
                                Ok(response) => {
                                    if response.status().is_success() {
                                        Event::LoadUpdateResponse(Ok(()))
                                    } else {
                                        Event::LoadUpdateResponse(Err(format!(
                                            "Load update failed: HTTP {}",
                                            response.status()
                                        )))
                                    }
                                }
                                Err(e) => Event::LoadUpdateResponse(Err(e.to_string())),
                            }),
                    ])
                } else {
                    render()
                }
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

            Event::RunUpdate { validate_iothub } => {
                model.is_loading = true;
                #[derive(Serialize)]
                struct RunUpdateRequest {
                    validate_iothub: bool,
                }
                if let Some(token) = &model.auth_token {
                    Command::all([
                        render(),
                        HttpCmd::post(format!("{}/api/update/run", API_BASE_URL))
                            .header("Authorization", format!("Bearer {token}"))
                            .header("Content-Type", "application/json")
                            .body_json(&RunUpdateRequest { validate_iothub })
                            .expect("Failed to serialize run update request")
                            .build()
                            .then_send(|result| match result {
                                Ok(response) => {
                                    if response.status().is_success() {
                                        Event::RunUpdateResponse(Ok(()))
                                    } else {
                                        Event::RunUpdateResponse(Err(format!(
                                            "Run update failed: HTTP {}",
                                            response.status()
                                        )))
                                    }
                                }
                                Err(e) => Event::RunUpdateResponse(Err(e.to_string())),
                            }),
                    ])
                } else {
                    render()
                }
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

            // WebSocket subscriptions (using new Command API)
            Event::SubscribeToChannels => {
                // Connect to Centrifugo and subscribe to all channels
                CentrifugoCmd::subscribe_all()
                    .build()
                    .then_send(Event::CentrifugoResponse)
            }

            Event::UnsubscribeFromChannels => {
                // Unsubscribe from all channels
                CentrifugoCmd::unsubscribe_all()
                    .build()
                    .then_send(Event::CentrifugoResponse)
            }

            // Centrifugo response handling
            Event::CentrifugoResponse(output) => match output {
                CentrifugoOutput::Connected => {
                    model.is_connected = true;
                    render()
                }
                CentrifugoOutput::Disconnected => {
                    model.is_connected = false;
                    render()
                }
                CentrifugoOutput::Subscribed { channel: _ } => {
                    // Subscription confirmed, no model change needed
                    render()
                }
                CentrifugoOutput::Unsubscribed { channel: _ } => {
                    // Unsubscription confirmed, no model change needed
                    render()
                }
                CentrifugoOutput::Message { channel, data } => {
                    // Parse the JSON data and dispatch to appropriate update event
                    match channel.as_str() {
                        "SystemInfoV1" => {
                            if let Ok(info) = serde_json::from_str::<SystemInfo>(&data) {
                                model.system_info = Some(info);
                            }
                        }
                        "NetworkStatusV1" => {
                            if let Ok(status) = serde_json::from_str::<NetworkStatus>(&data) {
                                model.network_status = Some(status);
                            }
                        }
                        "OnlineStatusV1" => {
                            if let Ok(status) = serde_json::from_str::<OnlineStatus>(&data) {
                                model.online_status = Some(status);
                            }
                        }
                        "FactoryResetV1" => {
                            if let Ok(reset) = serde_json::from_str::<FactoryReset>(&data) {
                                model.factory_reset = Some(reset);
                            }
                        }
                        "UpdateValidationStatusV1" => {
                            if let Ok(status) =
                                serde_json::from_str::<UpdateValidationStatus>(&data)
                            {
                                model.update_validation_status = Some(status);
                            }
                        }
                        "TimeoutsV1" => {
                            if let Ok(timeouts) = serde_json::from_str::<Timeouts>(&data) {
                                model.timeouts = Some(timeouts);
                            }
                        }
                        _ => {
                            // Unknown channel, ignore
                        }
                    }
                    render()
                }
                CentrifugoOutput::HistoryResult { channel, data } => {
                    // Handle history result similar to Message
                    if let Some(json_data) = data {
                        match channel.as_str() {
                            "SystemInfoV1" => {
                                if let Ok(info) = serde_json::from_str::<SystemInfo>(&json_data) {
                                    model.system_info = Some(info);
                                }
                            }
                            "NetworkStatusV1" => {
                                if let Ok(status) =
                                    serde_json::from_str::<NetworkStatus>(&json_data)
                                {
                                    model.network_status = Some(status);
                                }
                            }
                            "OnlineStatusV1" => {
                                if let Ok(status) = serde_json::from_str::<OnlineStatus>(&json_data)
                                {
                                    model.online_status = Some(status);
                                }
                            }
                            "FactoryResetV1" => {
                                if let Ok(reset) = serde_json::from_str::<FactoryReset>(&json_data)
                                {
                                    model.factory_reset = Some(reset);
                                }
                            }
                            "UpdateValidationStatusV1" => {
                                if let Ok(status) =
                                    serde_json::from_str::<UpdateValidationStatus>(&json_data)
                                {
                                    model.update_validation_status = Some(status);
                                }
                            }
                            "TimeoutsV1" => {
                                if let Ok(timeouts) = serde_json::from_str::<Timeouts>(&json_data) {
                                    model.timeouts = Some(timeouts);
                                }
                            }
                            _ => {
                                // Unknown channel, ignore
                            }
                        }
                    }
                    render()
                }
                CentrifugoOutput::Error { message } => {
                    model.error_message = Some(format!("Centrifugo error: {message}"));
                    render()
                }
            },

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
