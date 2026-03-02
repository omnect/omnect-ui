use crux_core::Command;

use crate::{
    auth_post, build_url,
    events::{Event, UiEvent},
    handle_response, http_get,
    model::Model,
    types::TimeoutSettings,
    update_field, Effect,
};

/// Handle UI-related events (clear messages, settings, etc.)
pub fn handle(event: UiEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        UiEvent::ClearError => update_field!(model.error_message, None),
        UiEvent::ClearSuccess => update_field!(model.success_message, None),
        UiEvent::SetBrowserHostname(hostname) => {
            model.browser_hostname = Some(hostname);
            model.update_current_connection_adapter();
            crux_core::render::render()
        }
        UiEvent::LoadSettings => {
            http_get!(Ui, UiEvent, build_url("/settings"), LoadSettingsResponse, TimeoutSettings)
        }
        UiEvent::LoadSettingsResponse(result) => handle_response!(model, result, {
            on_success: |m, settings| {
                m.timeout_settings = settings;
            },
            no_loading: true,
        }),
        UiEvent::SaveSettings(settings) => {
            // Optimistic update so the form reflects the new values immediately,
            // regardless of when the POST response arrives.
            model.timeout_settings = settings.clone();
            auth_post!(Ui, UiEvent, model, "/settings", SaveSettingsResponse, "Save settings",
                body_json: &settings
            )
        }
        UiEvent::SaveSettingsResponse(result) => handle_response!(model, result, {
            success_message: "Settings saved",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::UiEvent;
    use crate::types::{DeviceNetwork, InternetProtocol, IpAddress, NetworkStatus};

    #[test]
    fn clear_error_removes_error_message() {
        let mut model = Model {
            error_message: Some("Test error".to_string()),
            ..Default::default()
        };

        let _ = handle(UiEvent::ClearError, &mut model);

        assert_eq!(model.error_message, None);
    }

    #[test]
    fn clear_success_removes_success_message() {
        let mut model = Model {
            success_message: Some("Test success".to_string()),
            ..Default::default()
        };

        let _ = handle(UiEvent::ClearSuccess, &mut model);

        assert_eq!(model.success_message, None);
    }

    #[test]
    fn set_browser_hostname_stores_hostname() {
        let mut model = Model::default();

        let _ = handle(
            UiEvent::SetBrowserHostname("192.168.1.100".to_string()),
            &mut model,
        );

        assert_eq!(model.browser_hostname, Some("192.168.1.100".to_string()));
    }

    #[test]
    fn set_browser_hostname_updates_current_connection_adapter() {
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

        let _ = handle(
            UiEvent::SetBrowserHostname("192.168.1.100".to_string()),
            &mut model,
        );

        assert_eq!(model.current_connection_adapter, Some("eth0".to_string()));
    }

    #[test]
    fn load_settings_response_updates_model() {
        let mut model = Model::default();
        let custom = TimeoutSettings {
            reboot_timeout_secs: 120,
            factory_reset_timeout_secs: 300,
            firmware_update_timeout_secs: 300,
            network_rollback_timeout_secs: 60,
        };

        let _ = handle(
            UiEvent::LoadSettingsResponse(Ok(custom.clone())),
            &mut model,
        );

        assert_eq!(model.timeout_settings, custom);
    }

    #[test]
    fn load_settings_response_error_sets_error_message() {
        let mut model = Model::default();

        let _ = handle(
            UiEvent::LoadSettingsResponse(Err("fetch failed".to_string())),
            &mut model,
        );

        assert_eq!(model.error_message, Some("fetch failed".to_string()));
    }

    #[test]
    fn save_settings_response_sets_success_message() {
        let mut model = Model {
            is_loading: true,
            ..Default::default()
        };

        let _ = handle(UiEvent::SaveSettingsResponse(Ok(())), &mut model);

        assert_eq!(model.success_message, Some("Settings saved".to_string()));
        assert!(!model.is_loading);
    }
}
