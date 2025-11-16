use crux_core::{render::render, Command};

use crate::capabilities::centrifugo::CentrifugoOutput;
use crate::events::Event;
use crate::model::Model;
use crate::types::*;
use crate::{CentrifugoCmd, Effect};

/// Handle WebSocket and Centrifugo-related events
pub fn handle(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::SubscribeToChannels => CentrifugoCmd::subscribe_all()
            .build()
            .then_send(Event::CentrifugoResponse),

        Event::UnsubscribeFromChannels => CentrifugoCmd::unsubscribe_all()
            .build()
            .then_send(Event::CentrifugoResponse),

        Event::CentrifugoResponse(output) => handle_centrifugo_output(output, model),

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

        Event::Connected => {
            model.is_connected = true;
            render()
        }

        Event::Disconnected => {
            model.is_connected = false;
            render()
        }

        _ => unreachable!("Non-websocket event passed to websocket handler"),
    }
}

/// Handle Centrifugo output messages
fn handle_centrifugo_output(output: CentrifugoOutput, model: &mut Model) -> Command<Effect, Event> {
    match output {
        CentrifugoOutput::Connected => {
            model.is_connected = true;
            render()
        }
        CentrifugoOutput::Disconnected => {
            model.is_connected = false;
            render()
        }
        CentrifugoOutput::Subscribed { channel: _ } => render(),
        CentrifugoOutput::Unsubscribed { channel: _ } => render(),
        CentrifugoOutput::Message { channel, data } => {
            parse_channel_message(&channel, &data, model);
            render()
        }
        CentrifugoOutput::HistoryResult { channel, data } => {
            if let Some(json_data) = data {
                parse_channel_message(&channel, &json_data, model);
            }
            render()
        }
        CentrifugoOutput::Error { message } => {
            model.error_message = Some(format!("Centrifugo error: {message}"));
            render()
        }
    }
}

/// Parse JSON data from a channel message and update the model
fn parse_channel_message(channel: &str, data: &str, model: &mut Model) {
    match channel {
        "SystemInfoV1" => {
            if let Ok(info) = serde_json::from_str::<SystemInfo>(data) {
                model.system_info = Some(info);
            }
        }
        "NetworkStatusV1" => {
            if let Ok(status) = serde_json::from_str::<NetworkStatus>(data) {
                model.network_status = Some(status);
            }
        }
        "OnlineStatusV1" => {
            if let Ok(status) = serde_json::from_str::<OnlineStatus>(data) {
                model.online_status = Some(status);
            }
        }
        "FactoryResetV1" => {
            if let Ok(reset) = serde_json::from_str::<FactoryReset>(data) {
                model.factory_reset = Some(reset);
            }
        }
        "UpdateValidationStatusV1" => {
            if let Ok(status) = serde_json::from_str::<UpdateValidationStatus>(data) {
                model.update_validation_status = Some(status);
            }
        }
        "TimeoutsV1" => {
            if let Ok(timeouts) = serde_json::from_str::<Timeouts>(data) {
                model.timeouts = Some(timeouts);
            }
        }
        _ => {
            // Unknown channel, ignore
        }
    }
}
