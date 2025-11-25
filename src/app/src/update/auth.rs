use crux_core::{render::render, Command};
use serde::Serialize;

use crate::events::Event;
use crate::handle_response;
use crate::model::Model;
use crate::types::{AuthToken, LoginCredentials};
use crate::{Effect, HttpCmd, API_BASE_URL};

/// Handle authentication-related events
pub fn handle(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Login { password } => {
            model.is_loading = true;
            model.error_message = None;
            let credentials = LoginCredentials { password };
            Command::all([
                render(),
                HttpCmd::post(format!("{API_BASE_URL}/api/token/login"))
                    .header("Content-Type", "application/json")
                    .body_json(&credentials)
                    .expect("Failed to serialize login credentials")
                    .expect_json::<AuthToken>()
                    .build()
                    .then_send(|result| match result {
                        Ok(mut response) => match response.take_body() {
                            Some(token) => Event::LoginResponse(Ok(token)),
                            None => Event::LoginResponse(Err("Empty response body".to_string())),
                        },
                        Err(e) => Event::LoginResponse(Err(e.to_string())),
                    }),
            ])
        }

        Event::LoginResponse(result) => handle_response!(model, result, {
            on_success: |model, auth| {
                model.auth_token = Some(auth.token);
                model.is_authenticated = true;
                model.error_message = None;
            },
        }),

        Event::Logout => {
            model.is_loading = true;
            if let Some(token) = &model.auth_token {
                Command::all([
                    render(),
                    HttpCmd::post(format!("{API_BASE_URL}/api/token/logout"))
                        .header("Authorization", format!("Bearer {token}"))
                        .build()
                        .then_send(|result| match result {
                            Ok(response) => {
                                if response.status().is_success() {
                                    Event::LogoutResponse(Ok(()))
                                } else {
                                    Event::LogoutResponse(Err(format!(
                                        "Logout failed: HTTP {}",
                                        response.status()
                                    )))
                                }
                            }
                            Err(e) => Event::LogoutResponse(Err(e.to_string())),
                        }),
                ])
            } else {
                render()
            }
        }

        Event::LogoutResponse(result) => handle_response!(model, result, {
            on_success: |model, _| {
                model.auth_token = None;
                model.is_authenticated = false;
            },
        }),

        Event::SetPassword { password } => {
            model.is_loading = true;
            #[derive(Serialize)]
            struct SetPasswordRequest {
                password: String,
            }
            Command::all([
                render(),
                HttpCmd::post(format!("{API_BASE_URL}/api/token/set-password"))
                    .header("Content-Type", "application/json")
                    .body_json(&SetPasswordRequest { password })
                    .expect("Failed to serialize password request")
                    .build()
                    .then_send(|result| match result {
                        Ok(response) => {
                            if response.status().is_success() {
                                Event::SetPasswordResponse(Ok(()))
                            } else {
                                Event::SetPasswordResponse(Err(format!(
                                    "Set password failed: HTTP {}",
                                    response.status()
                                )))
                            }
                        }
                        Err(e) => Event::SetPasswordResponse(Err(e.to_string())),
                    }),
            ])
        }

        Event::SetPasswordResponse(result) => handle_response!(model, result, {
            on_success: |model, _| {
                model.requires_password_set = false;
            },
            success_message: "Password set successfully",
        }),

        Event::UpdatePassword {
            current,
            new_password,
        } => {
            model.is_loading = true;
            #[derive(Serialize)]
            struct UpdatePasswordRequest {
                current: String,
                new_password: String,
            }
            if let Some(token) = &model.auth_token {
                Command::all([
                    render(),
                    HttpCmd::post(format!("{API_BASE_URL}/api/token/update-password"))
                        .header("Authorization", format!("Bearer {token}"))
                        .header("Content-Type", "application/json")
                        .body_json(&UpdatePasswordRequest {
                            current,
                            new_password,
                        })
                        .expect("Failed to serialize password update request")
                        .build()
                        .then_send(|result| match result {
                            Ok(response) => {
                                if response.status().is_success() {
                                    Event::UpdatePasswordResponse(Ok(()))
                                } else {
                                    Event::UpdatePasswordResponse(Err(format!(
                                        "Update password failed: HTTP {}",
                                        response.status()
                                    )))
                                }
                            }
                            Err(e) => Event::UpdatePasswordResponse(Err(e.to_string())),
                        }),
                ])
            } else {
                render()
            }
        }

        Event::UpdatePasswordResponse(result) => handle_response!(model, result, {
            success_message: "Password updated successfully",
        }),

        Event::CheckRequiresPasswordSet => {
            model.is_loading = true;
            Command::all([
                render(),
                HttpCmd::get(format!("{API_BASE_URL}/api/token/requires-password-set"))
                    .expect_json::<bool>()
                    .build()
                    .then_send(|result| match result {
                        Ok(mut response) => match response.take_body() {
                            Some(requires) => Event::CheckRequiresPasswordSetResponse(Ok(requires)),
                            None => Event::CheckRequiresPasswordSetResponse(Err(
                                "Empty response body".to_string(),
                            )),
                        },
                        Err(e) => Event::CheckRequiresPasswordSetResponse(Err(e.to_string())),
                    }),
            ])
        }

        Event::CheckRequiresPasswordSetResponse(result) => handle_response!(model, result, {
            on_success: |model, requires| {
                model.requires_password_set = requires;
            },
        }),

        _ => unreachable!("Non-auth event passed to auth handler"),
    }
}
