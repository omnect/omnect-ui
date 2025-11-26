/// Macro for model field updates with automatic rendering.
/// Supports both single and multiple field updates.
///
/// # Examples
///
/// Single field update:
/// ```ignore
/// update_field!(model.system_info, Some(info))
/// ```
///
/// Multiple field updates:
/// ```ignore
/// update_field!(
///     model.is_connected, true;
///     model.error_message, None
/// )
/// ```
#[macro_export]
macro_rules! update_field {
    // Multiple field updates (must come first to match the pattern)
    ($($model_field:expr, $value:expr);+ $(;)?) => {{
        $(
            $model_field = $value;
        )+
        crux_core::render::render()
    }};

    // Single field update
    ($model_field:expr, $value:expr) => {{
        $model_field = $value;
        crux_core::render::render()
    }};
}

/// Macro for parsing JSON channel messages with error handling.
/// Reduces repetitive JSON parsing in WebSocket message handlers.
///
/// # Example
///
/// ```ignore
/// parse_channel_data! {
///     channel, data, model,
///     "SystemInfoV1" => system_info: SystemInfo,
///     "NetworkStatusV1" => network_status: NetworkStatus,
///     "OnlineStatusV1" => online_status: OnlineStatus,
/// }
/// ```
#[macro_export]
macro_rules! parse_channel_data {
    ($channel:expr, $data:expr, $model:expr, $($channel_name:literal => $field:ident: $type:ty),+ $(,)?) => {
        match $channel {
            $(
                $channel_name => {
                    if let Ok(parsed) = serde_json::from_str::<$type>($data) {
                        $model.$field = Some(parsed);
                    }
                }
            )+
            _ => {
                // Unknown channel, ignore
            }
        }
    };
}

/// Helper function for standardized HTTP error messages
pub fn http_error(action: &str, status: impl std::fmt::Display) -> String {
    format!("{action} failed: HTTP {status}")
}

/// Helper function to build full API URL from endpoint
pub fn build_api_url(endpoint: &str) -> String {
    format!("{}{endpoint}", crate::API_BASE_URL)
}

/// Helper function to handle HTTP response with JSON body extraction
pub fn handle_json_response<T>(
    result: Result<crux_http::Response<T>, crux_http::HttpError>,
) -> Result<T, String> {
    match result {
        Ok(mut response) => match response.take_body() {
            Some(data) => Ok(data),
            None => Err("Empty response body".to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

/// Helper function to handle HTTP response with status check
pub fn handle_status_response(
    result: Result<crux_http::Response<Vec<u8>>, crux_http::HttpError>,
    action: &str,
) -> Result<(), String> {
    match result {
        Ok(response) => {
            if response.status().is_success() {
                Ok(())
            } else {
                Err(http_error(action, response.status()))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Helper macro to set loading state and wrap command with render
#[macro_export]
macro_rules! with_loading {
    ($model:expr, $command:expr) => {{
        $model.is_loading = true;
        crux_core::Command::all([crux_core::render::render(), $command])
    }};
}

/// Helper macro to handle result matching with error handling and render
#[macro_export]
macro_rules! handle_result {
    // With loading state management
    ($model:expr, $result:expr, $value:tt, $success_body:block) => {{
        $model.is_loading = false;
        match $result {
            Ok($value) => $success_body,
            Err(e) => {
                $model.error_message = Some(e);
            }
        }
        crux_core::render::render()
    }};

    // Without loading state management
    ($model:expr, $result:expr, $value:tt, $success_body:block, no_loading) => {{
        match $result {
            Ok($value) => $success_body,
            Err(e) => {
                $model.error_message = Some(e);
            }
        }
        crux_core::render::render()
    }};
}

/// Macro for unauthenticated POST requests with standard error handling.
/// Used for login, password setup, and other pre-authentication endpoints.
///
/// # Patterns
///
/// Pattern 1: POST with JSON body expecting JSON response
/// ```ignore
/// unauth_post!(model, "/api/token/login", LoginResponse, "Login",
///     body_json: &credentials,
///     expect_json: AuthToken
/// )
/// ```
///
/// Pattern 2: POST with JSON body expecting status only
/// ```ignore
/// unauth_post!(model, "/api/token/set-password", SetPasswordResponse, "Set password",
///     body_json: &request
/// )
/// ```
///
/// Pattern 3: GET expecting JSON response
/// ```ignore
/// unauth_post!(model, "/api/token/requires-password-set", CheckRequiresPasswordSetResponse, "Check password",
///     method: get,
///     expect_json: bool
/// )
/// ```
#[macro_export]
macro_rules! unauth_post {
    // Pattern 1: POST with JSON body expecting JSON response
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr, body_json: $body:expr, expect_json: $response_type:ty) => {
        $crate::with_loading!(
            $model,
            $crate::HttpCmd::post($crate::macros::build_api_url($endpoint))
                .header("Content-Type", "application/json")
                .body_json($body)
                .expect(&format!("Failed to serialize {} request", $action))
                .expect_json::<$response_type>()
                .build()
                .then_send(|result| {
                    $crate::Event::$response_event($crate::macros::handle_json_response(result))
                })
        )
    };

    // Pattern 2: POST with JSON body expecting status only
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr, body_json: $body:expr) => {
        $crate::with_loading!(
            $model,
            $crate::HttpCmd::post($crate::macros::build_api_url($endpoint))
                .header("Content-Type", "application/json")
                .body_json($body)
                .expect(&format!("Failed to serialize {} request", $action))
                .build()
                .then_send(|result| {
                    $crate::Event::$response_event($crate::macros::handle_status_response(
                        result, $action,
                    ))
                })
        )
    };

    // Pattern 3: GET expecting JSON response
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr, method: get, expect_json: $response_type:ty) => {
        $crate::with_loading!(
            $model,
            $crate::HttpCmd::get($crate::macros::build_api_url($endpoint))
                .expect_json::<$response_type>()
                .build()
                .then_send(|result| {
                    $crate::Event::$response_event($crate::macros::handle_json_response(result))
                })
        )
    };
}

/// Macro for authenticated POST requests with standard error handling.
/// Reduces boilerplate for POST requests that require authentication.
///
/// # Patterns
///
/// Pattern 1: Simple POST without body
/// ```ignore
/// auth_post!(model, "/api/device/reboot", RebootResponse, "Reboot")
/// ```
///
/// Pattern 2: POST with JSON body
/// ```ignore
/// auth_post!(model, "/api/device/factory-reset", FactoryResetResponse, "Factory reset",
///     body_json: &FactoryResetRequest { mode, preserve }
/// )
/// ```
///
/// Pattern 3: POST with string body
/// ```ignore
/// auth_post!(model, "/api/device/network", SetNetworkConfigResponse, "Set network config",
///     body_string: config
/// )
/// ```
#[macro_export]
macro_rules! auth_post {
    // Pattern 1: Simple POST without body
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr) => {{
        if let Some(token) = &$model.auth_token {
            $crate::with_loading!(
                $model,
                $crate::HttpCmd::post($crate::macros::build_api_url($endpoint))
                    .header("Authorization", format!("Bearer {token}"))
                    .build()
                    .then_send(|result| {
                        $crate::Event::$response_event($crate::macros::handle_status_response(
                            result, $action,
                        ))
                    })
            )
        } else {
            crux_core::render::render()
        }
    }};

    // Pattern 2: POST with JSON body
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr, body_json: $body:expr) => {{
        if let Some(token) = &$model.auth_token {
            $crate::with_loading!(
                $model,
                $crate::HttpCmd::post($crate::macros::build_api_url($endpoint))
                    .header("Authorization", format!("Bearer {token}"))
                    .header("Content-Type", "application/json")
                    .body_json($body)
                    .expect(&format!("Failed to serialize {} request", $action))
                    .build()
                    .then_send(|result| {
                        $crate::Event::$response_event($crate::macros::handle_status_response(
                            result, $action,
                        ))
                    })
            )
        } else {
            crux_core::render::render()
        }
    }};

    // Pattern 3: POST with string body
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr, body_string: $body:expr) => {{
        if let Some(token) = &$model.auth_token {
            $crate::with_loading!(
                $model,
                $crate::HttpCmd::post($crate::macros::build_api_url($endpoint))
                    .header("Authorization", format!("Bearer {token}"))
                    .header("Content-Type", "application/json")
                    .body_string($body)
                    .build()
                    .then_send(|result| {
                        $crate::Event::$response_event($crate::macros::handle_status_response(
                            result, $action,
                        ))
                    })
            )
        } else {
            crux_core::render::render()
        }
    }};
}

/// Macro for handling response events with standard loading state and error handling.
///
/// # Patterns
///
/// Pattern 1: Only success message (for `Result<(), String>`)
/// ```ignore
/// handle_response!(model, result, {
///     success_message: "Operation successful",
/// })
/// ```
///
/// Pattern 2: Custom success handling
/// ```ignore
/// handle_response!(model, result, {
///     on_success: |m, value| {
///         m.some_field = value;
///     },
/// })
/// ```
///
/// Pattern 3: Custom success handler + success message
/// ```ignore
/// handle_response!(model, result, {
///     on_success: |m, value| {
///         m.some_field = value;
///     },
///     success_message: "Operation successful",
/// })
/// ```
///
/// Pattern 4: Custom success handler without loading state (for responses that don't set loading)
/// ```ignore
/// handle_response!(model, result, {
///     on_success: |m, info| {
///         m.healthcheck = Some(info);
///     },
///     no_loading: true,
/// })
/// ```
#[macro_export]
macro_rules! handle_response {
    // Pattern 1: Only success message (for Result<(), String>)
    ($model:expr, $result:expr, {
        success_message: $msg:expr $(,)?
    }) => {
        $crate::handle_result!($model, $result, (), {
            $model.success_message = Some($msg.to_string());
        })
    };

    // Pattern 2: Only custom success handler
    ($model:expr, $result:expr, {
        on_success: |$success_model:ident, $value:tt| $success_body:block $(,)?
    }) => {
        $crate::handle_result!($model, $result, $value, {
            #[allow(clippy::redundant_locals)]
            let $success_model = $model;
            $success_body
        })
    };

    // Pattern 3: Custom success handler + success message
    ($model:expr, $result:expr, {
        on_success: |$success_model:ident, $value:tt| $success_body:block,
        success_message: $msg:expr $(,)?
    }) => {
        $crate::handle_result!($model, $result, $value, {
            #[allow(clippy::redundant_locals)]
            let $success_model = $model;
            $success_body
            $model.success_message = Some($msg.to_string());
        })
    };

    // Pattern 4: Only on_success without loading state (for HealthcheckResponse)
    ($model:expr, $result:expr, {
        on_success: |$success_model:ident, $value:tt| $success_body:block,
        no_loading: true $(,)?
    }) => {
        $crate::handle_result!($model, $result, $value, {
            #[allow(clippy::redundant_locals)]
            let $success_model = $model;
            $success_body
        }, no_loading)
    };
}
