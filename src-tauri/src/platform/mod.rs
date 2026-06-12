pub mod macos_permissions;
pub mod windows_permissions;

use serde::{Deserialize, Serialize};

use crate::storage::files::now_ms;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionState {
    Granted,
    Denied,
    NotDetermined,
    Unsupported,
    Unknown,
}

impl PermissionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionState::Granted => "granted",
            PermissionState::Denied => "denied",
            PermissionState::NotDetermined => "not_determined",
            PermissionState::Unsupported => "unsupported",
            PermissionState::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub accessibility: PermissionState,
    pub input_monitoring: PermissionState,
    pub screen_recording: PermissionState,
    pub windows_input: PermissionState,
    pub windows_integrity_level: Option<String>,
    pub can_capture_mouse: bool,
    pub can_inject_mouse: bool,
    pub updated_at_ms: u64,
}

impl PermissionStatus {
    pub fn unsupported() -> Self {
        Self {
            accessibility: PermissionState::Unsupported,
            input_monitoring: PermissionState::Unsupported,
            screen_recording: PermissionState::Unsupported,
            windows_input: PermissionState::Unsupported,
            windows_integrity_level: None,
            can_capture_mouse: false,
            can_inject_mouse: false,
            updated_at_ms: now_ms(),
        }
    }
}
