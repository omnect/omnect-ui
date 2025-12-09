use serde::{Deserialize, Serialize};

/// Operating system information
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
}

/// System information from WebSocket
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    pub os: OsInfo,
    pub azure_sdk_version: String,
    pub omnect_device_service_version: String,
    pub boot_time: Option<String>,
}

/// Online status from WebSocket
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OnlineStatus {
    pub iothub: bool,
}

/// Duration type for timeouts
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Duration {
    pub nanos: u32,
    pub secs: u64,
}

/// Timeout configurations from WebSocket
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Timeouts {
    pub wait_online_timeout: Duration,
}

/// Overlay spinner state (UI state)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverlaySpinnerState {
    overlay: bool,
    title: String,
    text: Option<String>,
    timed_out: bool,
}

impl OverlaySpinnerState {
    /// Create a new overlay spinner with the given title (shown by default)
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            overlay: true,
            title: title.into(),
            text: None,
            timed_out: false,
        }
    }

    /// Builder pattern: add optional text to the spinner
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Update the optional text message
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = Some(text.into());
    }

    /// Mark the spinner as timed out
    pub fn set_timed_out(&mut self) {
        self.timed_out = true;
    }

    /// Show the overlay spinner
    pub fn show(&mut self) {
        self.overlay = true;
    }

    /// Hide the overlay spinner
    pub fn hide(&mut self) {
        self.overlay = false;
    }

    /// Check if the overlay is currently visible
    pub fn is_visible(&self) -> bool {
        self.overlay
    }

    /// Reset to default (hidden) state
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Get the overlay visibility state
    pub fn overlay(&self) -> bool {
        self.overlay
    }

    /// Get the title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the optional text
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Check if the spinner has timed out
    pub fn timed_out(&self) -> bool {
        self.timed_out
    }
}
