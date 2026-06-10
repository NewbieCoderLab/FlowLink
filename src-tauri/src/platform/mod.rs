pub mod macos_permissions;
pub mod windows_permissions;

use serde::{Deserialize, Serialize};

use crate::{identity::{DeviceIdentity, OsArchExt}, storage::files::now_ms};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
    pub can_capture_mouse: bool,
    pub can_inject_mouse: bool,
    pub updated_at_ms: u64,
}

impl PermissionStatus {
    pub fn from_identity(identity: &DeviceIdentity) -> Self {
        match identity.os_name() {
            "macos" => Self {
                accessibility: PermissionState::NotDetermined,
                input_monitoring: PermissionState::NotDetermined,
                screen_recording: PermissionState::Unsupported,
                windows_input: PermissionState::Unsupported,
                can_capture_mouse: false,
                can_inject_mouse: false,
                updated_at_ms: now_ms(),
            },
            "windows" => Self {
                accessibility: PermissionState::Unsupported,
                input_monitoring: PermissionState::Unsupported,
                screen_recording: PermissionState::Unsupported,
                windows_input: PermissionState::Unknown,
                can_capture_mouse: false,
                can_inject_mouse: false,
                updated_at_ms: now_ms(),
            },
            _ => Self {
                accessibility: PermissionState::Unknown,
                input_monitoring: PermissionState::Unknown,
                screen_recording: PermissionState::Unknown,
                windows_input: PermissionState::Unknown,
                can_capture_mouse: false,
                can_inject_mouse: false,
                updated_at_ms: now_ms(),
            },
        }
    }
}
