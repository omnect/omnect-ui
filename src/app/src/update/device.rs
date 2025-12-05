use crux_core::Command;

use crate::auth_post;
use crate::events::Event;
use crate::handle_response;
use crate::model::Model;
use crate::types::{
    DeviceOperationState, FactoryResetRequest, LoadUpdateRequest, NetworkChangeState,
    NetworkConfigRequest, NetworkFormData, NetworkFormState, OverlaySpinnerState, RunUpdateRequest,
    UpdateManifest,
};
use crate::{Effect, HttpCmd};

/// Handle device action events (reboot, factory reset, network, updates)
pub fn handle(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Reboot => {
            model.overlay_spinner = OverlaySpinnerState {
                overlay: true,
                title: "Requesting device reboot...".to_string(),
                text: None,
                timed_out: false,
            };
            auth_post!(model, "/reboot", RebootResponse, "Reboot")
        }

        Event::RebootResponse(result) => handle_reboot_response(result, model),

        Event::FactoryResetRequest { mode, preserve } => {
            let parsed_mode = match mode.parse::<u8>() {
                Ok(m) => m,
                Err(e) => {
                    model.error_message = Some(format!("Invalid factory reset mode: {}", e));
                    return crux_core::render::render();
                }
            };
            let request = FactoryResetRequest {
                mode: parsed_mode,
                preserve,
            };
            model.overlay_spinner = OverlaySpinnerState {
                overlay: true,
                title: "Requesting factory reset...".to_string(),
                text: None,
                timed_out: false,
            };
            auth_post!(model, "/factory-reset", FactoryResetResponse, "Factory reset",
                body_json: &request
            )
        }

        Event::FactoryResetResponse(result) => handle_factory_reset_response(result, model),

        Event::ReloadNetwork => {
            auth_post!(
                model,
                "/reload-network",
                ReloadNetworkResponse,
                "Reload network"
            )
        }

        Event::ReloadNetworkResponse(result) => handle_response!(model, result, {
            success_message: "Network reloaded",
        }),

        Event::SetNetworkConfig { config } => handle_set_network_config(config, model),

        Event::SetNetworkConfigResponse(result) => {
            handle_set_network_config_response(result, model)
        }

        Event::LoadUpdate { file_path } => {
            let request = LoadUpdateRequest { file_path };
            auth_post!(model, "/update/load", LoadUpdateResponse, "Load update",
                body_json: &request,
                expect_json: UpdateManifest
            )
        }

        Event::LoadUpdateResponse(result) => handle_response!(model, result, {
            on_success: |model, manifest| {
                model.update_manifest = Some(manifest);
            },
            success_message: "Update loaded",
        }),

        Event::RunUpdate { validate_iothub_connection } => {
            let request = RunUpdateRequest { validate_iothub_connection };
            model.overlay_spinner = OverlaySpinnerState {
                overlay: true,
                title: "Requesting update...".to_string(),
                text: None,
                timed_out: false,
            };
            auth_post!(model, "/update/run", RunUpdateResponse, "Run update",
                body_json: &request
            )
        }

        Event::RunUpdateResponse(result) => handle_run_update_response(result, model),

        Event::HealthcheckResponse(result) => handle_healthcheck_response(result, model),

        // Device reconnection events (reboot/factory reset)
        // Shell sends these tick events based on watching device_operation_state
        Event::ReconnectionCheckTick => handle_reconnection_check_tick(model),
        Event::ReconnectionTimeout => handle_reconnection_timeout(model),

        // Network IP change events
        // Shell sends these tick events based on watching network_change_state
        Event::NewIpCheckTick => handle_new_ip_check_tick(model),
        Event::NewIpCheckTimeout => handle_new_ip_check_timeout(model),

        // Network form events
        Event::NetworkFormStartEdit { adapter_name } => {
            handle_network_form_start_edit(adapter_name, model)
        }
        Event::NetworkFormUpdate { form_data } => handle_network_form_update(form_data, model),
        Event::NetworkFormReset { adapter_name } => handle_network_form_reset(adapter_name, model),

        _ => unreachable!("Non-device event passed to device handler"),
    }
}

// ============================================================================
// Reboot/Factory Reset Reconnection Logic
// ============================================================================

fn handle_reboot_response(result: Result<(), String>, model: &mut Model) -> Command<Effect, Event> {
    model.is_loading = false;
    match result {
        Ok(()) => {
            model.success_message = Some("Reboot initiated".to_string());
            model.device_operation_state = DeviceOperationState::Rebooting;
            model.reconnection_attempt = 0;

            // Set overlay spinner state for reboot
            model.overlay_spinner = OverlaySpinnerState {
                overlay: true,
                title: "Device is rebooting".to_string(),
                text: None,
                timed_out: false,
            };

            // Shell will see the state change and start polling (5 min timeout for reboot)
            crux_core::render::render()
        }
        Err(e) => {
            // Check if error is likely due to connection loss during reboot
            // "Failed to fetch" is standard in browsers for network errors (like connection refused)
            // "IO error" is what our backend/macros might wrap it in
            let e_lower = e.to_lowercase();
            if e_lower.contains("failed to fetch")
                || e_lower.contains("networkerror")
                || e.contains("IO error")
            {
                // Treat as success - device likely went down immediately
                model.success_message = Some("Reboot initiated (connection lost)".to_string());
                model.device_operation_state = DeviceOperationState::Rebooting;
                model.reconnection_attempt = 0;

                // Keep/Update overlay
                model.overlay_spinner = OverlaySpinnerState {
                    overlay: true,
                    title: "Device is rebooting".to_string(),
                    text: None,
                    timed_out: false,
                };
                crux_core::render::render()
            } else {
                model.error_message = Some(e);
                model.overlay_spinner = OverlaySpinnerState::default();
                crux_core::render::render()
            }
        }
    }
}

fn handle_factory_reset_response(
    result: Result<(), String>,
    model: &mut Model,
) -> Command<Effect, Event> {
    model.is_loading = false;

    // Check if this is a success or a network error (both indicate reset started)
    let is_network_error = result.as_ref().is_err_and(|e| {
        let e_lower = e.to_lowercase();
        e_lower.contains("failed to fetch")
            || e_lower.contains("networkerror")
            || e.contains("IO error")
    });

    if result.is_ok() || is_network_error {
        let connection_lost = is_network_error;
        model.success_message = Some(if connection_lost {
            "Factory reset initiated (connection lost)".to_string()
        } else {
            "Factory reset initiated".to_string()
        });
        model.device_operation_state = DeviceOperationState::FactoryResetting;
        model.reconnection_attempt = 0;
        model.overlay_spinner = OverlaySpinnerState {
            overlay: true,
            title: "The device is resetting".to_string(),
            text: Some(
                "Please wait while the device resets. The app will be temporarily \
                 removed and reinstalled automatically when the device is back online."
                    .to_string(),
            ),
            timed_out: false,
        };
    } else if let Err(e) = result {
        model.error_message = Some(e);
        model.overlay_spinner = OverlaySpinnerState::default();
    }

    crux_core::render::render()
}

fn handle_run_update_response(
    result: Result<(), String>,
    model: &mut Model,
) -> Command<Effect, Event> {
    model.is_loading = false;
    match result {
        Ok(()) => {
            model.success_message = Some("Update started".to_string());
            model.device_operation_state = DeviceOperationState::Updating;
            model.reconnection_attempt = 0;

            // Set overlay spinner state for update
            model.overlay_spinner = OverlaySpinnerState {
                overlay: true,
                title: "Installing update".to_string(),
                text: Some("Please have some patience, the update may take some time.".to_string()),
                timed_out: false,
            };

            crux_core::render::render()
        }
        Err(e) => {
            // Check for network error
            let e_lower = e.to_lowercase();
            if e_lower.contains("failed to fetch")
                || e_lower.contains("networkerror")
                || e.contains("IO error")
            {
                model.success_message = Some("Update started (connection lost)".to_string());
                model.device_operation_state = DeviceOperationState::Updating;
                model.reconnection_attempt = 0;

                model.overlay_spinner = OverlaySpinnerState {
                    overlay: true,
                    title: "Installing update".to_string(),
                    text: Some(
                        "Please have some patience, the update may take some time.".to_string(),
                    ),
                    timed_out: false,
                };
                crux_core::render::render()
            } else {
                model.error_message = Some(e);
                model.overlay_spinner = OverlaySpinnerState::default();
                crux_core::render::render()
            }
        }
    }
}

fn handle_reconnection_check_tick(model: &mut Model) -> Command<Effect, Event> {
    // Only check if we're waiting for reconnection
    if !matches!(
        model.device_operation_state,
        DeviceOperationState::Rebooting
            | DeviceOperationState::FactoryResetting
            | DeviceOperationState::Updating
            | DeviceOperationState::WaitingReconnection { .. }
    ) {
        return crux_core::render::render();
    }

    model.reconnection_attempt += 1;

    // Send healthcheck request
    HttpCmd::get("http://omnect-device/healthcheck")
        .build()
        .then_send(|result| {
            Event::HealthcheckResponse(match result {
                Ok(response) => {
                    if response.status().is_success() {
                        Ok(crate::types::HealthcheckInfo::default())
                    } else {
                        Err(format!("Healthcheck failed: {}", response.status()))
                    }
                }
                Err(e) => Err(e.to_string()),
            })
        })
}

fn handle_reconnection_timeout(model: &mut Model) -> Command<Effect, Event> {
    let operation = match &model.device_operation_state {
        DeviceOperationState::Rebooting => "Reboot".to_string(),
        DeviceOperationState::FactoryResetting => "Factory Reset".to_string(),
        DeviceOperationState::Updating => "Update".to_string(),
        DeviceOperationState::WaitingReconnection { operation, .. } => operation.clone(),
        _ => return crux_core::render::render(),
    };

    let timeout_msg = if matches!(
        model.device_operation_state,
        DeviceOperationState::FactoryResetting
    ) {
        "Device did not come back online after 10 minutes. Please check the device manually."
    } else {
        "Device did not come back online after 5 minutes. Please check the device manually."
    };

    model.device_operation_state = DeviceOperationState::ReconnectionFailed {
        operation: operation.clone(),
        reason: timeout_msg.to_string(),
    };

    // Update overlay spinner to show timeout
    model.overlay_spinner.text = Some(timeout_msg.to_string());
    model.overlay_spinner.timed_out = true;

    crux_core::render::render()
}

fn handle_healthcheck_response(
    result: Result<crate::types::HealthcheckInfo, String>,
    model: &mut Model,
) -> Command<Effect, Event> {
    // Update healthcheck info if success
    if let Ok(info) = &result {
        model.healthcheck = Some(info.clone());
    }

    // Handle reconnection state machine
    match &model.device_operation_state {
        DeviceOperationState::Rebooting
        | DeviceOperationState::FactoryResetting
        | DeviceOperationState::Updating => {
            // First check - if it fails, mark as "waiting"
            let is_updating =
                matches!(model.device_operation_state, DeviceOperationState::Updating);

            // For updates, we also check the status field
            // Consider update done when status is Succeeded, Recovered, or NoUpdate
            // (NoUpdate means there's no pending update, so previous one completed)
            let update_done = if is_updating {
                if let Ok(info) = &result {
                    let status = &info.update_validation_status.status;
                    status == "Succeeded" || status == "Recovered" || status == "NoUpdate"
                } else {
                    false
                }
            } else {
                result.is_ok()
            };

            if update_done || (result.is_ok() && !is_updating) { // <-- Corrected here
                // Success on first check (or update finished)
                let operation = match model.device_operation_state {
                    DeviceOperationState::Rebooting => "Reboot".to_string(),
                    DeviceOperationState::FactoryResetting => "Factory Reset".to_string(),
                    DeviceOperationState::Updating => "Update".to_string(),
                    _ => "Unknown".to_string(),
                };
                model.device_operation_state =
                    DeviceOperationState::ReconnectionSuccessful { operation };
                
                // Invalidate session as backend restart clears tokens
                model.is_authenticated = false;
                model.auth_token = None;

                // Clear overlay spinner
                model.overlay_spinner = OverlaySpinnerState::default();
            } else {
                // Failed or still updating - transition to waiting
                let operation = match model.device_operation_state {
                    DeviceOperationState::Rebooting => "Reboot".to_string(),
                    DeviceOperationState::FactoryResetting => "Factory Reset".to_string(),
                    DeviceOperationState::Updating => "Update".to_string(),
                    _ => "Unknown".to_string(),
                };
                model.device_operation_state = DeviceOperationState::WaitingReconnection {
                    operation,
                    attempt: model.reconnection_attempt,
                };
            }
        }
        DeviceOperationState::WaitingReconnection { operation, .. } => {
            let is_update = operation == "Update";
            // Consider update done when status is Succeeded, Recovered, or NoUpdate
            let update_done = if is_update {
                if let Ok(info) = &result {
                    let status = &info.update_validation_status.status;
                    status == "Succeeded" || status == "Recovered" || status == "NoUpdate"
                } else {
                    false
                }
            } else {
                result.is_ok()
            };

            if update_done || (result.is_ok() && !is_update) {
                // Success! Device is back online (or update finished)
                model.device_operation_state = DeviceOperationState::ReconnectionSuccessful {
                    operation: operation.clone(),
                };
                
                // Invalidate session as backend restart clears tokens
                model.is_authenticated = false;
                model.auth_token = None;

                // Clear overlay spinner
                model.overlay_spinner = OverlaySpinnerState::default();
            } else {
                // Still waiting, update attempt counter
                model.device_operation_state = DeviceOperationState::WaitingReconnection {
                    operation: operation.clone(),
                    attempt: model.reconnection_attempt,
                };
            }
        }
        _ => {} // Do nothing for other states
    }

    // Handle network change state machine for IP change polling
    if let NetworkChangeState::WaitingForNewIp { new_ip, .. } = &model.network_change_state {
        if result.is_ok() {
            // Clone new_ip before reassigning state to avoid borrow conflict
            let new_ip = new_ip.clone();
            // New IP is reachable
            model.network_change_state = NetworkChangeState::NewIpReachable {
                new_ip: new_ip.clone(),
            };
            // Update overlay for redirect
            model.overlay_spinner = OverlaySpinnerState {
                overlay: true,
                title: "Network settings applied".to_string(),
                text: Some(format!("Redirecting to new IP: {new_ip}")),
                timed_out: false,
            };
        }
    }

    crux_core::render::render()
}

// ============================================================================
// Network Configuration & IP Change Logic
// ============================================================================

fn handle_set_network_config(config: String, model: &mut Model) -> Command<Effect, Event> {
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

fn handle_set_network_config_response(
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

fn handle_new_ip_check_tick(model: &mut Model) -> Command<Effect, Event> {
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

fn handle_new_ip_check_timeout(model: &mut Model) -> Command<Effect, Event> {
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

// ============================================================================
// Network Form State Management
// ============================================================================

fn handle_network_form_start_edit(
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

fn handle_network_form_update(form_data_json: String, model: &mut Model) -> Command<Effect, Event> {
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

fn handle_network_form_reset(adapter_name: String, model: &mut Model) -> Command<Effect, Event> {
    // Reset form to match current network status
    handle_network_form_start_edit(adapter_name, model)
}
