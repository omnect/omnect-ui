use crux_core::Command;

use crate::events::Event;
use crate::model::Model;
use crate::update_field;
use crate::{CentrifugoCmd, Effect};

/// Handle WebSocket and Centrifugo-related events
pub fn handle(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::SubscribeToChannels => {
            // Issue Centrifugo effect (shell sends WebSocket data as events directly)
            CentrifugoCmd::subscribe_all()
                .build()
                .then_send(|_| Event::Connected)
        }

        Event::UnsubscribeFromChannels => {
            // Issue Centrifugo effect
            CentrifugoCmd::unsubscribe_all()
                .build()
                .then_send(|_| Event::Disconnected)
        }

        Event::SystemInfoUpdated(info) => update_field!(model.system_info, Some(info)),
        Event::NetworkStatusUpdated(status) => update_field!(model.network_status, Some(status)),
        Event::OnlineStatusUpdated(status) => update_field!(model.online_status, Some(status)),
        Event::FactoryResetUpdated(reset) => update_field!(model.factory_reset, Some(reset)),
        Event::UpdateValidationStatusUpdated(status) => {
            update_field!(model.update_validation_status, Some(status))
        }
        Event::TimeoutsUpdated(timeouts) => update_field!(model.timeouts, Some(timeouts)),
        Event::Connected => update_field!(model.is_connected, true),
        Event::Disconnected => update_field!(model.is_connected, false),

        _ => unreachable!("Non-websocket event passed to websocket handler"),
    }
}
