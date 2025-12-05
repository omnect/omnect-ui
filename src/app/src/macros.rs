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

/// Helper function for standardized HTTP error messages
pub fn http_error(action: &str, status: impl std::fmt::Display) -> String {
    format!("{action} failed: HTTP {status}")
}

/// Helper function to extract error message from response body or fallback to status
pub fn extract_error(
    action: &str,
    response: &mut crux_http::Response<Vec<u8>>,
) -> String {
    // Check for original status header from shell hack
    let status = if let Some(original) = response.header("x-original-status") {
         original.as_str().to_string()
    } else {
         response.status().to_string()
    };

    match response.take_body() {
        Some(body) => {
            if body.is_empty() {
                format!("{action} failed: HTTP {status} (Empty body)")
            } else {
                match String::from_utf8(body) {
                    Ok(msg) => format!("Error: {}", msg),
                    Err(e) => format!("{action} failed: HTTP {status} (Invalid UTF-8: {e})"),
                }
            }
        },
        None => format!("{action} failed: HTTP {status} (No body)"),
    }
}

/// Helper function to check if a response is successful, handling the shell workaround.
/// Returns true if the response status is 2xx AND there is no 'x-original-status' header
/// indicating a masked error.
pub fn is_success(response: &crux_http::Response<Vec<u8>>) -> bool {
    let is_hack_error = response.header("x-original-status").is_some();
    response.status().is_success() && !is_hack_error
}

/// Macro for unauthenticated POST requests with standard error handling.
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
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr, body_json: $body:expr, expect_json: $response_type:ty) => {{
        $model.is_loading = true;
        match $crate::HttpCmd::post(format!("http://omnect-device{}", $endpoint))
            .header("Content-Type", "application/json")
            .body_json($body)
        {
            Ok(builder) => crux_core::Command::all([
                crux_core::render::render(),
                builder
                    .build()
                    .then_send(|result| match result {
                        Ok(mut response) => {
                            // Check for shell hack
                            let is_hack_error = response.header("x-original-status").is_some();

                            if response.status().is_success() && !is_hack_error {
                                match response.take_body() {
                                    Some(body) => match serde_json::from_slice::<$response_type>(&body) {
                                        Ok(data) => $crate::Event::$response_event(Ok(data)),
                                        Err(e) => $crate::Event::$response_event(Err(format!("JSON parse error: {e}"))),
                                    },
                                    None => $crate::Event::$response_event(Err(
                                        "Empty response body".to_string()
                                    )),
                                }
                            } else {
                                $crate::Event::$response_event(Err(
                                    $crate::macros::extract_error($action, &mut response)
                                ))
                            }
                        },
                        Err(e) => $crate::Event::$response_event(Err(e.to_string())),
                    }),
            ]),
            Err(e) => {
                $model.is_loading = false;
                $model.error_message = Some(format!("Failed to create {} request: {}", $action, e));
                crux_core::render::render()
            }
        }
    }};

    // Pattern 2: POST with JSON body expecting status only
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr, body_json: $body:expr) => {{
        $model.is_loading = true;
        match $crate::HttpCmd::post(format!("http://omnect-device{}", $endpoint))
            .header("Content-Type", "application/json")
            .body_json($body)
        {
            Ok(builder) => crux_core::Command::all([
                crux_core::render::render(),
                builder.build().then_send(|result| match result {
                    Ok(mut response) => {
                        // Check for shell hack
                        let is_hack_error = response.header("x-original-status").is_some();

                        if response.status().is_success() && !is_hack_error {
                            $crate::Event::$response_event(Ok(()))
                        } else {
                            $crate::Event::$response_event(Err(
                                $crate::macros::extract_error($action, &mut response)
                            ))
                        }
                    }
                    Err(e) => $crate::Event::$response_event(Err(e.to_string())),
                }),
            ]),
            Err(e) => {
                $model.is_loading = false;
                $model.error_message = Some(format!("Failed to create {} request: {}", $action, e));
                crux_core::render::render()
            }
        }
    }};

    // Pattern 3: GET expecting JSON response
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr, method: get, expect_json: $response_type:ty) => {{
        $model.is_loading = true;
        crux_core::Command::all([
            crux_core::render::render(),
            $crate::HttpCmd::get(format!("http://omnect-device{}", $endpoint))
                .build()
                .then_send(|result| match result {
                    Ok(mut response) => {
                        // Check for shell hack
                        let is_hack_error = response.header("x-original-status").is_some();

                        if response.status().is_success() && !is_hack_error {
                            match response.take_body() {
                                Some(body) => match serde_json::from_slice::<$response_type>(&body) {
                                    Ok(data) => $crate::Event::$response_event(Ok(data)),
                                    Err(e) => $crate::Event::$response_event(Err(format!("JSON parse error: {e}"))),
                                },
                                None => $crate::Event::$response_event(Err(
                                    "Empty response body".to_string()
                                )),
                            }
                        } else {
                            $crate::Event::$response_event(Err(
                                $crate::macros::extract_error($action, &mut response)
                            ))
                        }
                    },
                    Err(e) => $crate::Event::$response_event(Err(e.to_string())),
                }),
        ])
    }};
}

/// Macro for authenticated POST requests with standard error handling.
/// Reduces boilerplate for POST requests that require authentication.
///
/// NOTE: Endpoints are prefixed with `http://omnect-device` as a workaround.
/// `crux_http` (v0.15) panics when given a relative URL in some environments (e.g. `cargo test`).
/// The UI shell (`useCore.ts`) strips this prefix before sending the request.
/// This workaround should be removed once `crux_http` supports relative URLs gracefully.
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
        $model.is_loading = true;
        if let Some(token) = &$model.auth_token {
            crux_core::Command::all([
                crux_core::render::render(),
                $crate::HttpCmd::post(format!("http://omnect-device{}", $endpoint))
                    .header("Authorization", format!("Bearer {token}"))
                    .build()
                    .then_send(|result| match result {
                        Ok(mut response) => {
                            // Check for shell hack
                            let is_hack_error = response.header("x-original-status").is_some();

                            if response.status().is_success() && !is_hack_error {
                                $crate::Event::$response_event(Ok(()))
                            } else {
                                $crate::Event::$response_event(Err(
                                    $crate::macros::extract_error($action, &mut response)
                                ))
                            }
                        }
                        Err(e) => $crate::Event::$response_event(Err(e.to_string())),
                    }),
            ])
        } else {
            $model.is_loading = false;
            $model.error_message = Some(format!("{} failed: Not authenticated", $action));
            crux_core::render::render()
        }
    }};

    // Pattern 2: POST with JSON body
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr, body_json: $body:expr) => {{
        $model.is_loading = true;
        if let Some(token) = &$model.auth_token {
            match $crate::HttpCmd::post(format!("http://omnect-device{}", $endpoint))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body_json($body)
            {
                Ok(builder) => crux_core::Command::all([
                    crux_core::render::render(),
                    builder.build().then_send(|result| match result {
                        Ok(mut response) => {
                            // Check for shell hack
                            let is_hack_error = response.header("x-original-status").is_some();

                            if response.status().is_success() && !is_hack_error {
                                $crate::Event::$response_event(Ok(()))
                            } else {
                                $crate::Event::$response_event(Err(
                                    $crate::macros::extract_error($action, &mut response)
                                ))
                            }
                        }
                        Err(e) => $crate::Event::$response_event(Err(format!("CRUX_ERR: {}", e))),
                    }),
                ]),
                Err(e) => {
                    $model.is_loading = false;
                    $model.error_message =
                        Some(format!("Failed to create {} request: {}", $action, e));
                    crux_core::render::render()
                }
            }
        } else {
            $model.is_loading = false;
            $model.error_message = Some(format!("{} failed: Not authenticated", $action));
            crux_core::render::render()
        }
    }};

    // Pattern 3: POST with string body
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr, body_string: $body:expr) => {{
        $model.is_loading = true;
        if let Some(token) = &$model.auth_token {
            crux_core::Command::all([
                crux_core::render::render(),
                $crate::HttpCmd::post(format!("http://omnect-device{}", $endpoint))
                    .header("Authorization", format!("Bearer {token}"))
                    .header("Content-Type", "application/json")
                    .body_string($body)
                    .build()
                    .then_send(|result| match result {
                        Ok(mut response) => {
                            // Check for shell hack
                            let is_hack_error = response.header("x-original-status").is_some();

                            if response.status().is_success() && !is_hack_error {
                                $crate::Event::$response_event(Ok(()))
                            } else {
                                $crate::Event::$response_event(Err(
                                    $crate::macros::extract_error($action, &mut response)
                                ))
                            }
                        }
                        Err(e) => $crate::Event::$response_event(Err(e.to_string())),
                    }),
            ])
        } else {
            $model.is_loading = false;
            $model.error_message = Some(format!("{} failed: Not authenticated", $action));
            crux_core::render::render()
        }
    }};

    // Pattern 4: POST with JSON body expecting JSON response
    ($model:expr, $endpoint:expr, $response_event:ident, $action:expr, body_json: $body:expr, expect_json: $response_type:ty) => {{
        $model.is_loading = true;
        if let Some(token) = &$model.auth_token {
            match $crate::HttpCmd::post(format!("http://omnect-device{}", $endpoint))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body_json($body)
            {
                Ok(builder) => crux_core::Command::all([
                    crux_core::render::render(),
                    builder.build().then_send(
                        |result| match result {
                            Ok(mut response) => {
                                // Check for shell hack
                                let is_hack_error = response.header("x-original-status").is_some();

                                if response.status().is_success() && !is_hack_error {
                                    match response.take_body() {
                                        Some(body) => match serde_json::from_slice::<$response_type>(&body) {
                                            Ok(data) => $crate::Event::$response_event(Ok(data)),
                                            Err(e) => $crate::Event::$response_event(Err(format!("JSON parse error: {e}"))),
                                        },
                                        None => $crate::Event::$response_event(Err(
                                            "Empty response body".to_string()
                                        )),
                                    }
                                } else {
                                    $crate::Event::$response_event(Err(
                                        $crate::macros::extract_error($action, &mut response)
                                    ))
                                }
                            },
                            Err(e) => $crate::Event::$response_event(Err(e.to_string())),
                        },
                    ),
                ]),
                Err(e) => {
                    $model.is_loading = false;
                    $model.error_message =
                        Some(format!("Failed to create {} request: {}", $action, e));
                    crux_core::render::render()
                }
            }
        } else {
            $model.is_loading = false;
            $model.error_message = Some(format!("{} failed: Not authenticated", $action));
            crux_core::render::render()
        }
    }};
}

/// Macro for simple HTTP GET requests expecting JSON response.
/// Does not set loading state or require authentication.
///
/// # Example
/// ```ignore
/// http_get!("http://omnect-device/healthcheck", HealthcheckResponse, HealthcheckInfo)
/// ```
#[macro_export]
macro_rules! http_get {
    ($url:expr, $response_event:ident, $response_type:ty) => {
        $crate::HttpCmd::get($url)
            .build()
            .then_send(|result| {
                $crate::Event::$response_event(match result {
                    Ok(mut response) => {
                        // Check for shell hack
                        let is_hack_error = response.header("x-original-status").is_some();

                        if response.status().is_success() && !is_hack_error {
                            response.body_json().map_err(|e| format!("Failed to parse response: {e}"))
                        } else {
                            Err(format!("Request failed: {}", response.status()))
                        }
                    }
                    Err(e) => Err(e.to_string()),
                })
            })
    };
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
    }) => {{
        $model.is_loading = false;
        match $result {
            Ok(()) => {
                $model.success_message = Some($msg.to_string());
            }
            Err(e) => {
                $model.error_message = Some(e);
            }
        }
        crux_core::render::render()
    }};

    // Pattern 2: Only custom success handler
    ($model:expr, $result:expr, {
        on_success: |$success_model:ident, $value:tt| $success_body:block $(,)?
    }) => {{
        $model.is_loading = false;
        match $result {
            Ok($value) => {
                #[allow(clippy::redundant_locals)]
                let $success_model = $model;
                $success_body
            }
            Err(e) => {
                $model.error_message = Some(e);
            }
        }
        crux_core::render::render()
    }};

    // Pattern 3: Custom success handler + success message
    ($model:expr, $result:expr, {
        on_success: |$success_model:ident, $value:tt| $success_body:block,
        success_message: $msg:expr $(,)?
    }) => {{
        $model.is_loading = false;
        match $result {
            Ok($value) => {
                #[allow(clippy::redundant_locals)]
                let $success_model = $model;
                $success_body
                $model.success_message = Some($msg.to_string());
            }
            Err(e) => {
                $model.error_message = Some(e);
            }
        }
        crux_core::render::render()
    }};

    // Pattern 4: Only on_success without loading state (for HealthcheckResponse)
    ($model:expr, $result:expr, {
        on_success: |$success_model:ident, $value:tt| $success_body:block,
        no_loading: true $(,)?
    }) => {{
        match $result {
            Ok($value) => {
                #[allow(clippy::redundant_locals)]
                let $success_model = $model;
                $success_body
            }
            Err(e) => {
                $model.error_message = Some(e);
            }
        }
        crux_core::render::render()
    }};
}