use crux_core::Command;

use crate::http_get;
use crate::model::Model;
use crate::types::{DeviceOperationState, NetworkChangeState, OverlaySpinnerState};
use crate::{Effect, Event};

use super::operations::is_update_complete;

/// Handle reconnection check tick - polls healthcheck endpoint
pub fn handle_reconnection_check_tick(model: &mut Model) -> Command<Effect, Event> {
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
    http_get!(
        "http://omnect-device/healthcheck",
        HealthcheckResponse,
        crate::types::HealthcheckInfo
    )
}

/// Handle reconnection timeout - device didn't come back online
pub fn handle_reconnection_timeout(model: &mut Model) -> Command<Effect, Event> {
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

/// Handle healthcheck response - manages reconnection and network change state machines
pub fn handle_healthcheck_response(
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
                result.as_ref().ok().is_some_and(is_update_complete)
            } else {
                result.is_ok()
            };

            if result.is_err() {
                // Device went offline - mark it
                model.device_went_offline = true;
                // Transition to waiting
                let operation = model.device_operation_state.operation_name();
                model.device_operation_state = DeviceOperationState::WaitingReconnection {
                    operation,
                    attempt: model.reconnection_attempt,
                };
            } else if (update_done || !is_updating) && model.device_went_offline {
                // Success AND device went offline previously - reconnection successful
                let operation = model.device_operation_state.operation_name();
                model.device_operation_state =
                    DeviceOperationState::ReconnectionSuccessful { operation };

                // Invalidate session as backend restart clears tokens
                model.is_authenticated = false;
                model.auth_token = None;

                // Clear overlay spinner
                model.overlay_spinner = OverlaySpinnerState::default();
            }
            // else: healthcheck succeeded but device never went offline - keep checking
        }
        DeviceOperationState::WaitingReconnection { operation, .. } => {
            let is_update = operation == "Update";

            if result.is_err() {
                // Still offline - mark it
                model.device_went_offline = true;
                // Update attempt count
                model.device_operation_state = DeviceOperationState::WaitingReconnection {
                    operation: operation.clone(),
                    attempt: model.reconnection_attempt,
                };
            } else {
                // Consider update done when status is Succeeded, Recovered, or NoUpdate
                let update_done = if is_update {
                    if let Ok(info) = &result {
                        let status = &info.update_validation_status.status;
                        status == "Succeeded" || status == "Recovered" || status == "NoUpdate"
                    } else {
                        false
                    }
                } else {
                    true
                };

                if update_done && model.device_went_offline {
                    // Success! Device is back online (or update finished) AND it went offline
                    model.device_operation_state = DeviceOperationState::ReconnectionSuccessful {
                        operation: operation.clone(),
                    };

                    // Invalidate session as backend restart clears tokens
                    model.is_authenticated = false;
                    model.auth_token = None;

                    // Clear overlay spinner
                    model.overlay_spinner = OverlaySpinnerState::default();
                }
                // else: healthcheck succeeded but device never went offline - keep checking
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
