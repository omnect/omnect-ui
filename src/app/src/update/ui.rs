use crux_core::Command;

use crate::events::{Event, UiEvent};
use crate::model::Model;
use crate::update_field;
use crate::Effect;

/// Handle UI-related events (clear messages, etc.)
pub fn handle(event: UiEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        UiEvent::ClearError => update_field!(model.error_message, None),
        UiEvent::ClearSuccess => update_field!(model.success_message, None),
        UiEvent::SetBrowserHostname(hostname) => {
            model.browser_hostname = Some(hostname);
            model.update_current_connection_adapter();
            crux_core::render::render()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Event, UiEvent};
    use crate::types::{DeviceNetwork, InternetProtocol, IpAddress, NetworkStatus};
    use crate::App;
    use crux_core::testing::AppTester;

    #[test]
    fn clear_error_removes_error_message() {
        let app = AppTester::<App>::default();
        let mut model = Model {
            error_message: Some("Test error".to_string()),
            ..Default::default()
        };

        let _ = app.update(Event::Ui(UiEvent::ClearError), &mut model);

        assert_eq!(model.error_message, None);
    }

    #[test]
    fn clear_success_removes_success_message() {
        let app = AppTester::<App>::default();
        let mut model = Model {
            success_message: Some("Test success".to_string()),
            ..Default::default()
        };

        let _ = app.update(Event::Ui(UiEvent::ClearSuccess), &mut model);

        assert_eq!(model.success_message, None);
    }

    #[test]
    fn set_browser_hostname_stores_hostname() {
        let app = AppTester::<App>::default();
        let mut model = Model::default();

        let _ = app.update(
            Event::Ui(UiEvent::SetBrowserHostname("192.168.1.100".to_string())),
            &mut model,
        );

        assert_eq!(model.browser_hostname, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn set_browser_hostname_updates_current_connection_adapter() {
        let app = AppTester::<App>::default();
        let mut model = Model {
            network_status: Some(NetworkStatus {
                network_status: vec![DeviceNetwork {
                    name: "eth0".to_string(),
                    mac: "00:11:22:33:44:55".to_string(),
                    online: true,
                    file: Some("/etc/network/interfaces".to_string()),
                    ipv4: InternetProtocol {
                        addrs: vec![IpAddress {
                            addr: "192.168.1.100".to_string(),
                            dhcp: false,
                            prefix_len: 24,
                        }],
                        dns: vec![],
                        gateways: vec![],
                    },
                }],
            }),
            ..Default::default()
        };

        let _ = app.update(
            Event::Ui(UiEvent::SetBrowserHostname("192.168.1.100".to_string())),
            &mut model,
        );

        assert_eq!(
            model.current_connection_adapter,
            Some("eth0".to_string())
        );
    }
}
