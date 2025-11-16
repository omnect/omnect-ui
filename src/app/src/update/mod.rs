mod auth;
mod device;
mod ui;
mod websocket;

use crux_core::{render::render, Command};

use crate::events::Event;
use crate::model::{Model, ViewModel};
use crate::Effect;

/// Main update dispatcher - routes events to domain-specific handlers
pub fn update(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        // Initialization
        Event::Initialize => {
            model.is_loading = true;
            render()
        }

        // Authentication domain
        Event::Login { .. }
        | Event::LoginResponse(_)
        | Event::Logout
        | Event::LogoutResponse(_)
        | Event::SetPassword { .. }
        | Event::SetPasswordResponse(_)
        | Event::UpdatePassword { .. }
        | Event::UpdatePasswordResponse(_)
        | Event::CheckRequiresPasswordSet
        | Event::CheckRequiresPasswordSetResponse(_) => auth::handle(event, model),

        // Device actions domain
        Event::Reboot
        | Event::RebootResponse(_)
        | Event::FactoryResetRequest { .. }
        | Event::FactoryResetResponse(_)
        | Event::ReloadNetwork
        | Event::ReloadNetworkResponse(_)
        | Event::SetNetworkConfig { .. }
        | Event::SetNetworkConfigResponse(_)
        | Event::LoadUpdate { .. }
        | Event::LoadUpdateResponse(_)
        | Event::RunUpdate { .. }
        | Event::RunUpdateResponse(_)
        | Event::HealthcheckResponse(_) => device::handle(event, model),

        // WebSocket domain
        Event::SubscribeToChannels
        | Event::UnsubscribeFromChannels
        | Event::CentrifugoResponse(_)
        | Event::SystemInfoUpdated(_)
        | Event::NetworkStatusUpdated(_)
        | Event::OnlineStatusUpdated(_)
        | Event::FactoryResetUpdated(_)
        | Event::UpdateValidationStatusUpdated(_)
        | Event::TimeoutsUpdated(_)
        | Event::Connected
        | Event::Disconnected => websocket::handle(event, model),

        // UI actions domain
        Event::ClearError | Event::ClearSuccess => ui::handle(event, model),
    }
}

/// Convert Model to ViewModel for rendering
pub fn view(model: &Model) -> ViewModel {
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
