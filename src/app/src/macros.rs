/// Macro for simple model field updates with automatic rendering.
/// Reduces boilerplate for events that just update a field and render.
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
/// update_fields!(
///     model.is_connected, true;
///     model.error_message, None;
/// )
/// ```
#[macro_export]
macro_rules! update_field {
    ($model_field:expr, $value:expr) => {{
        $model_field = $value;
        crux_core::render::render()
    }};
}

/// Macro for updating multiple model fields at once.
#[macro_export]
macro_rules! update_fields {
    ($($model_field:expr, $value:expr);+ $(;)?) => {{
        $(
            $model_field = $value;
        )+
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
