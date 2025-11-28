use crux_core::Command;

use crate::auth_post;
use crate::events::Event;
use crate::handle_response;
use crate::model::Model;
use crate::types::{AuthToken, LoginCredentials, SetPasswordRequest, UpdatePasswordRequest};
use crate::unauth_post;
use crate::Effect;

/// Handle authentication-related events
pub fn handle(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Login { password } => {
            model.error_message = None;
            let credentials = LoginCredentials { password };
            unauth_post!(model, "/token/login", LoginResponse, "Login",
                body_json: &credentials,
                expect_json: AuthToken
            )
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

        _ => unreachable!("Non-auth event passed to auth handler"),
    }
}
