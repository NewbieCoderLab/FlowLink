#[cfg(target_os = "windows")]
use crate::platform::PermissionState;

#[cfg(target_os = "windows")]
pub fn settings_label() -> &'static str {
    "Windows Input Control"
}

#[cfg(target_os = "windows")]
pub fn input_status() -> PermissionState {
    // Low-level mouse hooks and SendInput work for normal desktop apps.
    // UIPI can still block elevated windows, which S1 reports as a limitation.
    PermissionState::Granted
}
