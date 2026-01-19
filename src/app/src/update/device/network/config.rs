use crux_core::Command;

use crate::auth_post;
use crate::events::Event;
use crate::model::Model;
use crate::types::{NetworkChangeState, NetworkConfigRequest, NetworkFormState};
use crate::Effect;

use super::verification::update_network_state_and_spinner;

/// Success message for network configuration update
const NETWORK_CONFIG_SUCCESS: &str = "Network configuration updated";

/// Handle network configuration request
pub fn handle_set_network_config(config: String, model: &mut Model) -> Command<Effect, Event> {
    // Parse the JSON config to extract metadata
    let parsed_config: Result<NetworkConfigRequest, _> = serde_json::from_str(&config);

    match parsed_config {
        Ok(config_req) => {
            let is_server_addr = model.is_current_adapter(&config_req.name);

            // Store network change state for later use
            // Show modal for: current connection AND (IP changed OR switching to DHCP OR rollback explicitly enabled)
            if is_server_addr
                && (config_req.ip_changed
                    || config_req.switching_to_dhcp
                    || config_req.enable_rollback.unwrap_or(false))
            {
                model.network_change_state = NetworkChangeState::ApplyingConfig {
                    is_server_addr: true,
                    ip_changed: config_req.ip_changed || config_req.switching_to_dhcp,
                    new_ip: config_req.ip.clone().unwrap_or_default(),
                    old_ip: config_req.previous_ip.clone().unwrap_or_default(),
                    switching_to_dhcp: config_req.switching_to_dhcp,
                };
            }

            // Transition network form to submitting state
            if let Some(submitting) = model.network_form_state.to_submitting(&config_req.name) {
                model.network_form_state = submitting;
            }

            // Clear dirty flag when submitting
            model.network_form_dirty = false;

            // Clear any previous messages so that identical subsequent messages
            // (e.g. from multiple network config applies) trigger the UI watcher correctly.
            model.success_message = None;
            model.error_message = None;

            // Send the request to backend
            auth_post!(
                Device,
                DeviceEvent,
                model,
                "/network",
                SetNetworkConfigResponse,
                "Set network config",
                body_string: config,
                expect_json: crate::types::SetNetworkConfigResponse
            )
        }
        Err(e) => model.set_error_and_render(format!("Invalid network config: {e}")),
    }
}

/// Handle network configuration response
pub fn handle_set_network_config_response(
    result: Result<crate::types::SetNetworkConfigResponse, String>,
    model: &mut Model,
) -> Command<Effect, Event> {
    model.stop_loading();

    match result {
        Ok(response) => {
            // Check if we are applying a config that changes IP/DHCP
            if let NetworkChangeState::ApplyingConfig {
                new_ip,
                old_ip,
                switching_to_dhcp,
                ..
            } = &model.network_change_state.clone()
            {
                if response.rollback_enabled {
                    update_network_state_and_spinner(
                        model,
                        new_ip.clone(),
                        old_ip.clone(),
                        response.ui_port,
                        response.rollback_timeout_seconds,
                        *switching_to_dhcp,
                        true,
                    );
                } else {
                    update_network_state_and_spinner(
                        model,
                        new_ip.clone(),
                        old_ip.clone(),
                        response.ui_port,
                        0,
                        *switching_to_dhcp,
                        false,
                    );
                }
            } else {
                // Not changing current connection's IP - just clear state
                model.network_change_state = NetworkChangeState::Idle;
                model.overlay_spinner.clear();
            }

            model.success_message = Some(NETWORK_CONFIG_SUCCESS.to_string());

            // Transition back to editing state with the new data as original
            if let NetworkFormState::Submitting {
                adapter_name,
                form_data,
                ..
            } = &model.network_form_state
            {
                model.network_form_state = NetworkFormState::Editing {
                    adapter_name: adapter_name.clone(),
                    original_data: form_data.clone(),
                    form_data: form_data.clone(),
                };
            } else {
                model.network_form_state = NetworkFormState::Idle;
            }

            // Clear rollback modal flag after config is applied
            model.should_show_rollback_modal = false;
            crux_core::render::render()
        }
        Err(e) => {
            model.set_error(e);
            model.network_change_state = NetworkChangeState::Idle;
            // Reset form state back to editing on failure
            if let Some(editing) = model.network_form_state.to_editing() {
                model.network_form_state = editing;
            }
            crux_core::render::render()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::DeviceEvent;
    use crate::types::{NetworkFormData, SetNetworkConfigResponse};
    use crate::App;
    use crux_core::testing::AppTester;

    #[test]
    fn static_ip_with_rollback_enters_waiting_state() {
        let app = AppTester::<App>::default();
        let mut model = Model {
            network_change_state: NetworkChangeState::ApplyingConfig {
                is_server_addr: true,
                ip_changed: true,
                new_ip: "192.168.1.101".to_string(),
                old_ip: "192.168.1.100".to_string(),
                switching_to_dhcp: false,
            },
            is_loading: true,
            ..Default::default()
        };

        let response = SetNetworkConfigResponse {
            rollback_timeout_seconds: 60,
            ui_port: 443,
            rollback_enabled: true,
        };

        let _ = app.update(
            Event::Device(DeviceEvent::SetNetworkConfigResponse(Ok(response))),
            &mut model,
        );

        assert!(!model.is_loading);
        assert_eq!(
            model.success_message,
            Some("Network configuration updated".to_string())
        );
        assert!(matches!(
            model.network_change_state,
            NetworkChangeState::WaitingForNewIp { .. }
        ));
        if let NetworkChangeState::WaitingForNewIp {
            new_ip,
            old_ip,
            rollback_timeout_seconds,
            switching_to_dhcp,
            ..
        } = model.network_change_state
        {
            assert_eq!(new_ip, "192.168.1.101");
            assert_eq!(old_ip, "192.168.1.100");
            assert_eq!(rollback_timeout_seconds, 60);
            assert!(!switching_to_dhcp);
        }
        assert_eq!(model.network_form_state, NetworkFormState::Idle);
    }

    #[test]
    fn static_ip_without_rollback_enters_waiting_state() {
        let app = AppTester::<App>::default();
        let mut model = Model {
            network_change_state: NetworkChangeState::ApplyingConfig {
                is_server_addr: true,
                ip_changed: true,
                new_ip: "192.168.1.101".to_string(),
                old_ip: "192.168.1.100".to_string(),
                switching_to_dhcp: false,
            },
            is_loading: true,
            ..Default::default()
        };

        let response = SetNetworkConfigResponse {
            rollback_timeout_seconds: 0,
            ui_port: 443,
            rollback_enabled: false,
        };

        let _ = app.update(
            Event::Device(DeviceEvent::SetNetworkConfigResponse(Ok(response))),
            &mut model,
        );

        assert!(matches!(
            model.network_change_state,
            NetworkChangeState::WaitingForNewIp { .. }
        ));
        if let NetworkChangeState::WaitingForNewIp {
            rollback_timeout_seconds,
            old_ip,
            ..
        } = model.network_change_state
        {
            assert_eq!(old_ip, "192.168.1.100");
            assert_eq!(rollback_timeout_seconds, 0);
        }
    }

    #[test]
    fn dhcp_with_rollback_enters_waiting_state() {
        let app = AppTester::<App>::default();
        let mut model = Model {
            network_change_state: NetworkChangeState::ApplyingConfig {
                is_server_addr: true,
                ip_changed: true,
                new_ip: "".to_string(),
                old_ip: "192.168.1.100".to_string(),
                switching_to_dhcp: true,
            },
            is_loading: true,
            ..Default::default()
        };

        let response = SetNetworkConfigResponse {
            rollback_timeout_seconds: 60,
            ui_port: 443,
            rollback_enabled: true,
        };

        let _ = app.update(
            Event::Device(DeviceEvent::SetNetworkConfigResponse(Ok(response))),
            &mut model,
        );

        assert!(matches!(
            model.network_change_state,
            NetworkChangeState::WaitingForNewIp { .. }
        ));
        if let NetworkChangeState::WaitingForNewIp {
            switching_to_dhcp,
            old_ip,
            ..
        } = model.network_change_state
        {
            assert_eq!(old_ip, "192.168.1.100");
            assert!(switching_to_dhcp);
        }
    }

    #[test]
    fn dhcp_without_rollback_goes_to_idle() {
        let app = AppTester::<App>::default();
        let mut model = Model {
            network_change_state: NetworkChangeState::ApplyingConfig {
                is_server_addr: true,
                ip_changed: true,
                new_ip: "".to_string(),
                old_ip: "192.168.1.100".to_string(),
                switching_to_dhcp: true,
            },
            is_loading: true,
            ..Default::default()
        };

        let response = SetNetworkConfigResponse {
            rollback_timeout_seconds: 0,
            ui_port: 443,
            rollback_enabled: false,
        };

        let _ = app.update(
            Event::Device(DeviceEvent::SetNetworkConfigResponse(Ok(response))),
            &mut model,
        );

        assert_eq!(model.network_change_state, NetworkChangeState::Idle);
        assert!(model.overlay_spinner.is_visible());
        assert!(model.overlay_spinner.countdown_seconds().is_none());
    }

    #[test]
    fn non_server_adapter_returns_to_editing_state() {
        let app = AppTester::<App>::default();
        let form_data = NetworkFormData {
            name: "eth1".to_string(),
            ip_address: "192.168.2.101".to_string(),
            dhcp: false,
            prefix_len: 24,
            dns: vec![],
            gateways: vec![],
        };

        let mut model = Model {
            network_form_state: NetworkFormState::Submitting {
                adapter_name: "eth1".to_string(),
                form_data: form_data.clone(),
                original_data: NetworkFormData {
                    name: "eth1".to_string(),
                    ip_address: "192.168.2.100".to_string(),
                    dhcp: false,
                    prefix_len: 24,
                    dns: vec![],
                    gateways: vec![],
                },
            },
            network_change_state: NetworkChangeState::Idle,
            is_loading: true,
            ..Default::default()
        };

        let response = SetNetworkConfigResponse {
            rollback_timeout_seconds: 0,
            ui_port: 443,
            rollback_enabled: false,
        };

        let _ = app.update(
            Event::Device(DeviceEvent::SetNetworkConfigResponse(Ok(response))),
            &mut model,
        );

        assert_eq!(model.network_change_state, NetworkChangeState::Idle);
        if let NetworkFormState::Editing {
            adapter_name,
            form_data: current_data,
            original_data,
        } = &model.network_form_state
        {
            assert_eq!(adapter_name, "eth1");
            assert_eq!(current_data.ip_address, "192.168.2.101");
            assert_eq!(original_data.ip_address, "192.168.2.101");
        } else {
            panic!(
                "Form state should be Editing, but was {:?}",
                model.network_form_state
            );
        }
    }

    #[test]
    fn non_server_adapter_returns_to_idle() {
        let app = AppTester::<App>::default();
        let mut model = Model {
            network_change_state: NetworkChangeState::Idle,
            is_loading: true,
            ..Default::default()
        };

        let response = SetNetworkConfigResponse {
            rollback_timeout_seconds: 60,
            ui_port: 443,
            rollback_enabled: true,
        };

        let _ = app.update(
            Event::Device(DeviceEvent::SetNetworkConfigResponse(Ok(response))),
            &mut model,
        );

        assert_eq!(model.network_change_state, NetworkChangeState::Idle);
        assert!(!model.overlay_spinner.is_visible());
    }

    #[test]
    fn error_resets_to_editing_state() {
        let app = AppTester::<App>::default();
        let form_data = NetworkFormData {
            name: "eth0".to_string(),
            ip_address: "192.168.1.100".to_string(),
            dhcp: false,
            prefix_len: 24,
            dns: vec![],
            gateways: vec![],
        };

        let mut model = Model {
            network_form_state: NetworkFormState::Submitting {
                adapter_name: "eth0".to_string(),
                form_data: form_data.clone(),
                original_data: form_data.clone(),
            },
            network_change_state: NetworkChangeState::ApplyingConfig {
                is_server_addr: true,
                ip_changed: true,
                new_ip: "192.168.1.101".to_string(),
                old_ip: "192.168.1.100".to_string(),
                switching_to_dhcp: false,
            },
            is_loading: true,
            ..Default::default()
        };

        let _ = app.update(
            Event::Device(DeviceEvent::SetNetworkConfigResponse(Err(
                "Network error".to_string()
            ))),
            &mut model,
        );

        assert!(!model.is_loading);
        assert!(model.error_message.is_some());
        assert_eq!(model.network_change_state, NetworkChangeState::Idle);
        assert!(matches!(
            model.network_form_state,
            NetworkFormState::Editing { .. }
        ));
    }
}
