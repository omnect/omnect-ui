pub mod capabilities;
pub mod events;
pub mod macros;
pub mod model;
pub mod types;
pub mod update;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

use crux_core::Command;

// Using deprecated Capabilities API for Http (kept for Effect enum generation)
#[allow(deprecated)]
use crux_http::Http;

// Re-export core types
pub use crate::capabilities::centrifugo::{CentrifugoOperation, CentrifugoOutput};
pub use crate::events::Event;
pub use crate::model::Model;
pub use crate::types::*;
pub use crux_http::Result as HttpResult;

/// API base URL - empty string means relative URLs (shell will use current origin)
/// For absolute URLs, set to something like "http://localhost:8000"
pub const API_BASE_URL: &str = "http://localhost:8000";

/// Capabilities - side effects the app can perform
///
/// Note: We keep the old deprecated capabilities in this struct ONLY for Effect enum generation.
/// The #[derive(crux_core::macros::Effect)] macro generates the Effect enum with proper
/// From<Request<Operation>> implementations based on what's declared here.
/// Actual usage goes through the Command-based APIs (HttpCmd, CentrifugoCmd).
#[allow(deprecated)]
#[cfg_attr(feature = "typegen", derive(crux_core::macros::Export))]
#[derive(crux_core::macros::Effect)]
pub struct Capabilities {
    pub render: crux_core::render::Render<Event>,
    pub http: Http<Event>,
    pub centrifugo: crate::capabilities::centrifugo::Centrifugo<Event>,
}

/// Type aliases for the Command-based APIs
/// Defined after Capabilities to have access to the generated Effect enum
pub type CentrifugoCmd = crate::capabilities::centrifugo_command::Centrifugo<Effect, Event>;
pub type HttpCmd = crux_http::command::Http<Effect, Event>;

/// The Core application
#[derive(Default)]
pub struct App;

impl crux_core::App for App {
    type Event = Event;
    type Model = Model;
    type ViewModel = Model;
    type Capabilities = Capabilities;
    type Effect = Effect;

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
        _caps: &Self::Capabilities,
    ) -> Command<Effect, Event> {
        update::update(event, model)
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        model.clone()
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
