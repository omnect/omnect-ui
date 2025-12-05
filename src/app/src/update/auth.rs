use base64::prelude::*;
use crux_core::Command;

use crate::auth_post;
use crate::events::Event;
use crate::handle_response;
use crate::model::Model;
use crate::types::{AuthToken, SetPasswordRequest, UpdatePasswordRequest};
use crate::unauth_post;
use crate::Effect;

/// Handle authentication-related events
pub fn handle(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Login { password } => {
            model.error_message = None;
            let encoded = BASE64_STANDARD.encode(format!(":{password}"));

            model.is_loading = true;
            // ToDo: replace by macro in future PR
            crux_core::Command::all([
                crux_core::render::render(),
                // Use dummy base URL to satisfy URL validation
                crate::HttpCmd::post("http://omnect-device/token/login")
                    .header("Authorization", format!("Basic {encoded}"))
                    .build()
                    .then_send(|result| match result {
                        Ok(mut response) => {
                            // Check for shell hack
                            let is_hack_error = response.header("x-original-status").is_some();

                            if response.status().is_success() && !is_hack_error {
                                match response.take_body() {
                                    Some(bytes) => match String::from_utf8(bytes) {
                                        Ok(token) => {
                                            let auth = AuthToken { token };
                                            crate::Event::LoginResponse(Ok(auth))
                                        }
                                        Err(_) => crate::Event::LoginResponse(Err(
                                            "Invalid UTF-8 in response".to_string(),
                                        )),
                                    },
                                    None => {
                                        crate::Event::LoginResponse(Err("Empty response body".to_string()))
                                    }
                                }
                            } else {
                                // Authentication failed - extract error message
                                crate::Event::LoginResponse(Err(
                                    crate::macros::extract_error("Login", &mut response)
                                ))
                            }
                        }
                        Err(e) => crate::Event::LoginResponse(Err(e.to_string())),
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

        Event::Logout => auth_post!(model, "/logout", LogoutResponse, "Logout"),

        Event::LogoutResponse(result) => handle_response!(model, result, {
            on_success: |model, _| {
                model.auth_token = None;
                model.is_authenticated = false;
            },
        }),

        Event::SetPassword { password } => {
            let request = SetPasswordRequest { password };
            unauth_post!(model, "/set-password", SetPasswordResponse, "Set password",
                body_json: &request
            )
        }

        Event::SetPasswordResponse(result) => handle_response!(model, result, {
            on_success: |model, _| {
                model.requires_password_set = false;
                model.error_message = None;
            },
            success_message: "Password set successfully",
        }),

        Event::UpdatePassword {
            current_password,
            password,
        } => {
            let request = UpdatePasswordRequest {
                current_password,
                password,
            };
            auth_post!(model, "/update-password", UpdatePasswordResponse, "Update password",
                body_json: &request
            )
        }

        Event::UpdatePasswordResponse(result) => handle_response!(model, result, {
            on_success: |model, _| {
                model.auth_token = None;
                model.is_authenticated = false;
            },
            success_message: "Password updated successfully",
        }),

        Event::CheckRequiresPasswordSet => {
            unauth_post!(model, "/require-set-password", CheckRequiresPasswordSetResponse, "Check password",
                method: get,
                expect_json: bool
            )
        }

        Event::CheckRequiresPasswordSetResponse(result) => handle_response!(model, result, {
            on_success: |model, requires| {
                model.requires_password_set = requires;
            },
        }),

        event => unreachable!("Non-auth event passed to auth handler: {:?}", event),
    }
}
