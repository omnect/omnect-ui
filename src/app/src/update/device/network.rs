use crux_core::Command;

use crate::auth_post;
use crate::events::{DeviceEvent, Event, UiEvent};
use crate::http_get_silent;
use crate::model::Model;
use crate::types::{
    HealthcheckInfo, NetworkChangeState, NetworkConfigRequest, NetworkFormData, NetworkFormState,
    OverlaySpinnerState,
};
use crate::Effect;

/// Success message for network configuration update
const NETWORK_CONFIG_SUCCESS: &str = "Network configuration updated";

/// Handle network configuration request
pub fn handle_set_network_config(config: String, model: &mut Model) -> Command<Effect, Event> {
    // Parse the JSON config to extract metadata
    let parsed_config: Result<NetworkConfigRequest, _> = serde_json::from_str(&config);

    match parsed_config {
        Ok(config_req) => {
            // Store network change state for later use
            if config_req.is_server_addr && config_req.ip_changed {
                model.network_change_state = NetworkChangeState::ApplyingConfig {
                    is_server_addr: true,
                    ip_changed: true,
                    new_ip: config_req.ip.clone().unwrap_or_default(),
                    old_ip: config_req.previous_ip.clone().unwrap_or_default(),
                };
            }

            // Transition network form to submitting state
            if let Some(submitting) = model.network_form_state.to_submitting() {
                model.network_form_state = submitting;
            }

            // Clear dirty flag when submitting
            model.network_form_dirty = false;

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
            // Check if rollback was enabled and we need to poll for new IP
            if response.rollback_enabled {
                if let NetworkChangeState::ApplyingConfig { new_ip, .. } =
                    &model.network_change_state.clone()
                {
                    model.network_change_state = NetworkChangeState::WaitingForNewIp {
                        new_ip: new_ip.clone(),
                        attempt: 0,
                        rollback_timeout_seconds: response.rollback_timeout_seconds,
                        ui_port: response.ui_port,
                    };
                    model.success_message = Some(NETWORK_CONFIG_SUCCESS.to_string());

                    // Set overlay spinner for IP change with countdown
                    // Shell will build redirect URL from network_change_state
                    model.overlay_spinner = OverlaySpinnerState::new("Applying network settings")
                        .with_text(
                            "Network configuration is being applied. Click the button below to open the new address in a new tab. \
                             You must access the new address to cancel the automatic rollback."
                        )
                        .with_countdown(response.rollback_timeout_seconds as u32);

                    // Reset form state after successful submission
                    model.network_form_state = NetworkFormState::Idle;

                    // Shell will see WaitingForNewIp state and start polling
                    crux_core::render::render()
                } else {
                    model.success_message = Some(NETWORK_CONFIG_SUCCESS.to_string());
                    model.network_form_state = NetworkFormState::Idle;
                    crux_core::render::render()
                }
            } else {
                // No rollback enabled - check if IP changed for current connection
                if let NetworkChangeState::ApplyingConfig { new_ip, .. } =
                    &model.network_change_state.clone()
                {
                    // Show overlay without countdown for manual navigation
                    model.network_change_state = NetworkChangeState::WaitingForNewIp {
                        new_ip: new_ip.clone(),
                        attempt: 0,
                        rollback_timeout_seconds: 0, // No countdown
                        ui_port: response.ui_port,
                    };
                    model.success_message = Some(NETWORK_CONFIG_SUCCESS.to_string());

                    // Set overlay spinner without countdown - just redirect button
                    model.overlay_spinner = OverlaySpinnerState::new("Applying network settings")
                        .with_text(
                            "Network configuration has been applied. Your connection will be interrupted. \
                             Click the button below to navigate to the new address."
                        );

                    // Reset form state after successful submission
                    model.network_form_state = NetworkFormState::Idle;

                    crux_core::render::render()
                } else {
                    // Not changing current connection's IP - just show success message
                    model.success_message = Some(NETWORK_CONFIG_SUCCESS.to_string());
                    model.network_change_state = NetworkChangeState::Idle;
                    model.network_form_state = NetworkFormState::Idle;
                    model.overlay_spinner.clear();
                    crux_core::render::render()
                }
            }
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

/// Handle new IP check tick - polls new IP to see if it's reachable
pub fn handle_new_ip_check_tick(model: &mut Model) -> Command<Effect, Event> {
    if let NetworkChangeState::WaitingForNewIp {
        new_ip,
        attempt,
        rollback_timeout_seconds,
        ui_port,
    } = &model.network_change_state
    {
        let new_ip = new_ip.clone();
        let new_attempt = *attempt + 1;
        let timeout_secs = *rollback_timeout_seconds;
        let port = *ui_port;

        // Update attempt counter
        model.network_change_state = NetworkChangeState::WaitingForNewIp {
            new_ip: new_ip.clone(),
            attempt: new_attempt,
            rollback_timeout_seconds: timeout_secs,
            ui_port: port,
        };

        // Try to reach the new IP (silent GET - no error shown on failure)
        // Use HTTPS since the server only listens on HTTPS
        let url = format!("https://{new_ip}:{port}/healthcheck");
        http_get_silent!(
            url,
            on_success: Event::Device(DeviceEvent::HealthcheckResponse(Ok(HealthcheckInfo::default()))),
            on_error: Event::Ui(UiEvent::ClearSuccess)
        )
    } else {
        crux_core::render::render()
    }
}

/// Handle new IP check timeout - new IP didn't become reachable in time
pub fn handle_new_ip_check_timeout(model: &mut Model) -> Command<Effect, Event> {
    if let NetworkChangeState::WaitingForNewIp {
        new_ip, ui_port, ..
    } = &model.network_change_state
    {
        let new_ip_url = format!("https://{new_ip}:{ui_port}");
        model.network_change_state = NetworkChangeState::NewIpTimeout {
            new_ip: new_ip.clone(),
            ui_port: *ui_port,
        };

        // Update overlay spinner to show timeout with manual link
        model.overlay_spinner.set_text(
            format!(
                "Automatic rollback will occur soon. The network settings were not confirmed at the new address. \
                 Please navigate to: {new_ip_url}"
            )
            .as_str(),
        );
        model.overlay_spinner.set_timed_out();
    }

    crux_core::render::render()
}

/// Handle network form start edit - initialize form with current network adapter data
pub fn handle_network_form_start_edit(
    adapter_name: String,
    model: &mut Model,
) -> Command<Effect, Event> {
    // Find the network adapter and copy its data to form state
    if let Some(network_status) = &model.network_status {
        if let Some(adapter) = network_status
            .network_status
            .iter()
            .find(|n| n.name == adapter_name)
        {
            let form_data = NetworkFormData {
                name: adapter.name.clone(),
                ip_address: adapter
                    .ipv4
                    .addrs
                    .first()
                    .map(|a| a.addr.clone())
                    .unwrap_or_default(),
                dhcp: adapter.ipv4.addrs.first().map(|a| a.dhcp).unwrap_or(false),
                prefix_len: adapter
                    .ipv4
                    .addrs
                    .first()
                    .map(|a| a.prefix_len)
                    .unwrap_or(24),
                dns: adapter.ipv4.dns.clone(),
                gateways: adapter.ipv4.gateways.clone(),
            };

            model.network_form_state = NetworkFormState::Editing {
                adapter_name: adapter_name.clone(),
                form_data,
            };
            // Clear dirty flag when starting a fresh edit
            model.network_form_dirty = false;
        }
    }

    crux_core::render::render()
}

/// Handle network form update - update form data from user input
pub fn handle_network_form_update(
    form_data_json: String,
    model: &mut Model,
) -> Command<Effect, Event> {
    // Parse the JSON form data
    let parsed: Result<NetworkFormData, _> = serde_json::from_str(&form_data_json);

    match parsed {
        Ok(form_data) => {
            if let NetworkFormState::Editing { adapter_name, .. } = &model.network_form_state {
                // Compare form data with original adapter data to determine if dirty
                let is_dirty = if let Some(network_status) = &model.network_status {
                    if let Some(adapter) = network_status
                        .network_status
                        .iter()
                        .find(|n| n.name == *adapter_name)
                    {
                        // Build original form data from current network status
                        let original_data = NetworkFormData {
                            name: adapter.name.clone(),
                            ip_address: adapter
                                .ipv4
                                .addrs
                                .first()
                                .map(|a| a.addr.clone())
                                .unwrap_or_default(),
                            dhcp: adapter.ipv4.addrs.first().map(|a| a.dhcp).unwrap_or(false),
                            prefix_len: adapter
                                .ipv4
                                .addrs
                                .first()
                                .map(|a| a.prefix_len)
                                .unwrap_or(24),
                            dns: adapter.ipv4.dns.clone(),
                            gateways: adapter.ipv4.gateways.clone(),
                        };

                        // Form is dirty if current data differs from original
                        form_data != original_data
                    } else {
                        // If we can't find the adapter, assume dirty
                        true
                    }
                } else {
                    // If we don't have network status, assume dirty
                    true
                };

                model.network_form_state = NetworkFormState::Editing {
                    adapter_name: adapter_name.clone(),
                    form_data,
                };
                model.network_form_dirty = is_dirty;
            }
            crux_core::render::render()
        }
        Err(e) => model.set_error_and_render(format!("Invalid form data: {e}")),
    }
}

/// Handle acknowledge network rollback - clear the rollback occurred flag
pub fn handle_ack_rollback(model: &mut Model) -> Command<Effect, Event> {
    // Clear the rollback status in the model
    if let Some(healthcheck) = &mut model.healthcheck {
        healthcheck.network_rollback_occurred = false;
    }

    // Send POST request to backend to clear the marker file
    auth_post!(
        Device,
        DeviceEvent,
        model,
        "/ack-rollback",
        AckRollbackResponse,
        "Acknowledge rollback"
    )
}
