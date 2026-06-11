pub mod macos_permissions;
pub mod windows_permissions;

use serde::{Deserialize, Serialize};

use crate::{
    identity::{DeviceIdentity, OsArchExt},
    storage::files::now_ms,
};

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
            can_capture_mouse: false,
            can_inject_mouse: false,
            updated_at_ms: now_ms(),
        }
    }

    pub fn from_identity(identity: &DeviceIdentity) -> Self {
        match identity.os_name() {
            "macos" => macos_permission_status(),
            "windows" => Self {
                accessibility: PermissionState::Unsupported,
                input_monitoring: PermissionState::Unsupported,
                screen_recording: PermissionState::Unsupported,
                windows_input: windows_input_status(),
                can_capture_mouse: windows_input_status() == PermissionState::Granted,
                can_inject_mouse: windows_input_status() == PermissionState::Granted,
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

fn macos_permission_status() -> PermissionStatus {
    let accessibility = macos_accessibility_status();
    let input_monitoring = macos_input_monitoring_status();

    PermissionStatus {
        accessibility,
        input_monitoring,
        screen_recording: PermissionState::Unsupported,
        windows_input: PermissionState::Unsupported,
        can_capture_mouse: input_monitoring == PermissionState::Granted,
        can_inject_mouse: accessibility == PermissionState::Granted,
        updated_at_ms: now_ms(),
    }
}

fn macos_accessibility_status() -> PermissionState {
    #[cfg(target_os = "macos")]
    {
        macos_permissions::accessibility_status()
    }

    #[cfg(not(target_os = "macos"))]
    {
        PermissionState::Unsupported
    }
}

fn macos_input_monitoring_status() -> PermissionState {
    #[cfg(target_os = "macos")]
    {
        macos_permissions::input_monitoring_status()
    }

    #[cfg(not(target_os = "macos"))]
    {
        PermissionState::Unsupported
    }
}

fn windows_input_status() -> PermissionState {
    #[cfg(target_os = "windows")]
    {
        windows_permissions::input_status()
    }

    #[cfg(not(target_os = "windows"))]
    {
        PermissionState::Unsupported
    }
}
