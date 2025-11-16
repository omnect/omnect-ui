use crux_core::{render::render, Command};

use crate::events::Event;
use crate::model::Model;
use crate::Effect;

/// Handle UI-related events (clear messages, etc.)
pub fn handle(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::ClearError => {
            model.error_message = None;
            render()
        }

        Event::ClearSuccess => {
            model.success_message = None;
            render()
        }

        _ => unreachable!("Non-UI event passed to UI handler"),
    }
}
