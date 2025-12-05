use crux_core::Command;

use crate::auth_post;
use crate::model::Model;
use crate::types::{
    NetworkChangeState, NetworkConfigRequest, NetworkFormData, NetworkFormState, OverlaySpinnerState,
};
use crate::{Effect, Event, HttpCmd};

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
            if let NetworkFormState::Editing {
                adapter_name,
                form_data,
            } = &model.network_form_state
            {
                model.network_form_state = NetworkFormState::Submitting {
                    adapter_name: adapter_name.clone(),
                    form_data: form_data.clone(),
                };
            }

            // Send the request to backend
            auth_post!(
                model,
                "/network",
                SetNetworkConfigResponse,
                "Set network config",
                body_string: config
            )
        }
        Err(e) => {
            model.is_loading = false;
            model.error_message = Some(format!("Invalid network config: {e}"));
            crux_core::render::render()
        }
    }
}

/// Handle network configuration response
pub fn handle_set_network_config_response(
    result: Result<(), String>,
    model: &mut Model,
) -> Command<Effect, Event> {
    model.is_loading = false;

    match result {
        Ok(()) => {
            // Check if we need to poll for new IP
            if let NetworkChangeState::ApplyingConfig { new_ip, .. } =
                &model.network_change_state.clone()
            {
                model.network_change_state = NetworkChangeState::WaitingForNewIp {
                    new_ip: new_ip.clone(),
                    attempt: 0,
                };
                model.success_message = Some("Network configuration updated".to_string());

                // Set overlay spinner for IP change
                model.overlay_spinner = OverlaySpinnerState {
                    overlay: true,
                    title: "Applying network settings".to_string(),
                    text: Some(
                        "The network settings are applied. You will be forwarded to the new IP. \
                         Log in to confirm the settings. If you do not log in within 90 seconds, \
                         the IP will be reset."
                            .to_string(),
                    ),
                    timed_out: false,
                };

                // Reset form state after successful submission
                model.network_form_state = NetworkFormState::Idle;

                // Shell will see WaitingForNewIp state and start polling
                crux_core::render::render()
            } else {
                model.success_message = Some("Network configuration updated".to_string());
                // Reset form state after successful submission
                model.network_form_state = NetworkFormState::Idle;
                crux_core::render::render()
            }
        }
        Err(e) => {
            model.error_message = Some(e);
            model.network_change_state = NetworkChangeState::Idle;
            // Reset form state back to editing on failure
            if let NetworkFormState::Submitting {
                adapter_name,
                form_data,
            } = &model.network_form_state
            {
                model.network_form_state = NetworkFormState::Editing {
                    adapter_name: adapter_name.clone(),
                    form_data: form_data.clone(),
                };
            }
            crux_core::render::render()
        }
    }
}

/// Handle new IP check tick - polls new IP to see if it's reachable
pub fn handle_new_ip_check_tick(model: &mut Model) -> Command<Effect, Event> {
    if let NetworkChangeState::WaitingForNewIp { new_ip, attempt } = &model.network_change_state {
        let new_ip = new_ip.clone();
        let new_attempt = *attempt + 1;

        // Update attempt counter
        model.network_change_state = NetworkChangeState::WaitingForNewIp {
            new_ip: new_ip.clone(),
            attempt: new_attempt,
        };

        // Try to reach the new IP
        let url = format!("http://{new_ip}/healthcheck");
        HttpCmd::get(url)
            .build()
            .then_send(move |result| match result {
                Ok(response) if response.status().is_success() => {
                    // New IP is reachable!
                    Event::HealthcheckResponse(Ok(crate::types::HealthcheckInfo::default()))
                }
                _ => {
                    // Still waiting - don't send error response, just render
                    Event::ClearSuccess
                }
            })
    } else {
        crux_core::render::render()
    }
}

/// Handle new IP check timeout - new IP didn't become reachable in time
pub fn handle_new_ip_check_timeout(model: &mut Model) -> Command<Effect, Event> {
    if let NetworkChangeState::WaitingForNewIp { new_ip, .. } = &model.network_change_state {
        model.network_change_state = NetworkChangeState::NewIpTimeout {
            new_ip: new_ip.clone(),
        };

        // Update overlay spinner to show timeout
        model.overlay_spinner.text = Some(
            "New IP not reachable within 90 seconds. Settings may have been reverted.".to_string(),
        );
        model.overlay_spinner.timed_out = true;
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
        }
    }

    crux_core::render::render()
}

/// Handle network form update - update form data from user input
pub fn handle_network_form_update(form_data_json: String, model: &mut Model) -> Command<Effect, Event> {
    // Parse the JSON form data
    let parsed: Result<NetworkFormData, _> = serde_json::from_str(&form_data_json);

    match parsed {
        Ok(form_data) => {
            if let NetworkFormState::Editing { adapter_name, .. } = &model.network_form_state {
                model.network_form_state = NetworkFormState::Editing {
                    adapter_name: adapter_name.clone(),
                    form_data,
                };
            }
            crux_core::render::render()
        }
        Err(e) => {
            model.error_message = Some(format!("Invalid form data: {e}"));
            crux_core::render::render()
        }
    }
}
