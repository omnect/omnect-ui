use crux_core::{render::render, Command};
use serde::Serialize;

use crate::events::Event;
use crate::model::Model;
use crate::{Effect, HttpCmd, API_BASE_URL};

/// Handle device action events (reboot, factory reset, network, updates)
pub fn handle(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Reboot => {
            model.is_loading = true;
            if let Some(token) = &model.auth_token {
                Command::all([
                    render(),
                    HttpCmd::post(format!("{API_BASE_URL}/api/device/reboot"))
                        .header("Authorization", format!("Bearer {token}"))
                        .build()
                        .then_send(|result| match result {
                            Ok(response) => {
                                if response.status().is_success() {
                                    Event::RebootResponse(Ok(()))
                                } else {
                                    Event::RebootResponse(Err(format!(
                                        "Reboot failed: HTTP {}",
                                        response.status()
                                    )))
                                }
                            }
                            Err(e) => Event::RebootResponse(Err(e.to_string())),
                        }),
                ])
            } else {
                render()
            }
        }

        Event::RebootResponse(result) => {
            model.is_loading = false;
            match result {
                Ok(()) => {
                    model.success_message = Some("Reboot initiated".to_string());
                }
                Err(e) => {
                    model.error_message = Some(e);
                }
            }
            render()
        }

        Event::FactoryResetRequest { mode, preserve } => {
            model.is_loading = true;
            #[derive(Serialize)]
            struct FactoryResetRequest {
                mode: String,
                preserve: Vec<String>,
            }
            if let Some(token) = &model.auth_token {
                Command::all([
                    render(),
                    HttpCmd::post(format!("{API_BASE_URL}/api/device/factory-reset"))
                        .header("Authorization", format!("Bearer {token}"))
                        .header("Content-Type", "application/json")
                        .body_json(&FactoryResetRequest { mode, preserve })
                        .expect("Failed to serialize factory reset request")
                        .build()
                        .then_send(|result| match result {
                            Ok(response) => {
                                if response.status().is_success() {
                                    Event::FactoryResetResponse(Ok(()))
                                } else {
                                    Event::FactoryResetResponse(Err(format!(
                                        "Factory reset failed: HTTP {}",
                                        response.status()
                                    )))
                                }
                            }
                            Err(e) => Event::FactoryResetResponse(Err(e.to_string())),
                        }),
                ])
            } else {
                render()
            }
        }

        Event::FactoryResetResponse(result) => {
            model.is_loading = false;
            match result {
                Ok(()) => {
                    model.success_message = Some("Factory reset initiated".to_string());
                }
                Err(e) => {
                    model.error_message = Some(e);
                }
            }
            render()
        }

        Event::ReloadNetwork => {
            model.is_loading = true;
            if let Some(token) = &model.auth_token {
                Command::all([
                    render(),
                    HttpCmd::post(format!("{API_BASE_URL}/api/device/reload-network"))
                        .header("Authorization", format!("Bearer {token}"))
                        .build()
                        .then_send(|result| match result {
                            Ok(response) => {
                                if response.status().is_success() {
                                    Event::ReloadNetworkResponse(Ok(()))
                                } else {
                                    Event::ReloadNetworkResponse(Err(format!(
                                        "Reload network failed: HTTP {}",
                                        response.status()
                                    )))
                                }
                            }
                            Err(e) => Event::ReloadNetworkResponse(Err(e.to_string())),
                        }),
                ])
            } else {
                render()
            }
        }

        Event::ReloadNetworkResponse(result) => {
            model.is_loading = false;
            match result {
                Ok(()) => {
                    model.success_message = Some("Network reloaded".to_string());
                }
                Err(e) => {
                    model.error_message = Some(e);
                }
            }
            render()
        }

        Event::SetNetworkConfig { config } => {
            model.is_loading = true;
            if let Some(token) = &model.auth_token {
                Command::all([
                    render(),
                    HttpCmd::post(format!("{API_BASE_URL}/api/device/network"))
                        .header("Authorization", format!("Bearer {token}"))
                        .header("Content-Type", "application/json")
                        .body_string(config)
                        .build()
                        .then_send(|result| match result {
                            Ok(response) => {
                                if response.status().is_success() {
                                    Event::SetNetworkConfigResponse(Ok(()))
                                } else {
                                    Event::SetNetworkConfigResponse(Err(format!(
                                        "Set network config failed: HTTP {}",
                                        response.status()
                                    )))
                                }
                            }
                            Err(e) => Event::SetNetworkConfigResponse(Err(e.to_string())),
                        }),
                ])
            } else {
                render()
            }
        }

        Event::SetNetworkConfigResponse(result) => {
            model.is_loading = false;
            match result {
                Ok(()) => {
                    model.success_message = Some("Network configuration updated".to_string());
                }
                Err(e) => {
                    model.error_message = Some(e);
                }
            }
            render()
        }

        Event::LoadUpdate { file_path } => {
            model.is_loading = true;
            #[derive(Serialize)]
            struct LoadUpdateRequest {
                file_path: String,
            }
            if let Some(token) = &model.auth_token {
                Command::all([
                    render(),
                    HttpCmd::post(format!("{API_BASE_URL}/api/update/load"))
                        .header("Authorization", format!("Bearer {token}"))
                        .header("Content-Type", "application/json")
                        .body_json(&LoadUpdateRequest { file_path })
                        .expect("Failed to serialize load update request")
                        .build()
                        .then_send(|result| match result {
                            Ok(response) => {
                                if response.status().is_success() {
                                    Event::LoadUpdateResponse(Ok(()))
                                } else {
                                    Event::LoadUpdateResponse(Err(format!(
                                        "Load update failed: HTTP {}",
                                        response.status()
                                    )))
                                }
                            }
                            Err(e) => Event::LoadUpdateResponse(Err(e.to_string())),
                        }),
                ])
            } else {
                render()
            }
        }

        Event::LoadUpdateResponse(result) => {
            model.is_loading = false;
            match result {
                Ok(()) => {
                    model.success_message = Some("Update loaded".to_string());
                }
                Err(e) => {
                    model.error_message = Some(e);
                }
            }
            render()
        }

        Event::RunUpdate { validate_iothub } => {
            model.is_loading = true;
            #[derive(Serialize)]
            struct RunUpdateRequest {
                validate_iothub: bool,
            }
            if let Some(token) = &model.auth_token {
                Command::all([
                    render(),
                    HttpCmd::post(format!("{API_BASE_URL}/api/update/run"))
                        .header("Authorization", format!("Bearer {token}"))
                        .header("Content-Type", "application/json")
                        .body_json(&RunUpdateRequest { validate_iothub })
                        .expect("Failed to serialize run update request")
                        .build()
                        .then_send(|result| match result {
                            Ok(response) => {
                                if response.status().is_success() {
                                    Event::RunUpdateResponse(Ok(()))
                                } else {
                                    Event::RunUpdateResponse(Err(format!(
                                        "Run update failed: HTTP {}",
                                        response.status()
                                    )))
                                }
                            }
                            Err(e) => Event::RunUpdateResponse(Err(e.to_string())),
                        }),
                ])
            } else {
                render()
            }
        }

        Event::RunUpdateResponse(result) => {
            model.is_loading = false;
            match result {
                Ok(()) => {
                    model.success_message = Some("Update started".to_string());
                }
                Err(e) => {
                    model.error_message = Some(e);
                }
            }
            render()
        }

        Event::HealthcheckResponse(result) => {
            match result {
                Ok(info) => {
                    model.healthcheck = Some(info);
                }
                Err(e) => {
                    model.error_message = Some(e);
                }
            }
            render()
        }

        _ => unreachable!("Non-device event passed to device handler"),
    }
}
