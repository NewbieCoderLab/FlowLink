#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "macos")]
use core_foundation::{
    base::TCFType,
    boolean::CFBoolean,
    dictionary::CFDictionary,
    string::{CFString, CFStringRef},
};

#[cfg(target_os = "macos")]
use crate::platform::PermissionState;

#[cfg(target_os = "macos")]
pub fn settings_label() -> &'static str {
    "Accessibility & Input Monitoring"
}

#[cfg(target_os = "macos")]
pub fn accessibility_status() -> PermissionState {
    if unsafe { AXIsProcessTrustedWithOptions(std::ptr::null()) } {
        PermissionState::Granted
    } else {
        PermissionState::NotDetermined
    }
}

#[cfg(target_os = "macos")]
pub fn input_monitoring_status() -> PermissionState {
    if unsafe { CGPreflightListenEventAccess() } {
        PermissionState::Granted
    } else {
        PermissionState::NotDetermined
    }
}

#[cfg(target_os = "macos")]
pub fn request_accessibility() -> Result<(), String> {
    let prompt_key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
    let prompt_value = CFBoolean::true_value();
    let options = CFDictionary::from_CFType_pairs(&[(prompt_key, prompt_value)]);
    let _ = unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) };
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn request_input_monitoring() -> Result<(), String> {
    if unsafe { CGRequestListenEventAccess() } {
        Ok(())
    } else {
        Err("input monitoring permission was not granted".into())
    }
}

#[cfg(target_os = "macos")]
pub fn open_settings_pane(kind: &str) -> Result<(), String> {
    let pane = match kind {
        "input_monitoring" => {
            "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
        }
        _ => "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
    };

    Command::new("open")
        .arg(pane)
        .status()
        .map_err(|err| err.to_string())
        .and_then(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(format!("open exited with status {status}"))
            }
        })
}

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    static kAXTrustedCheckOptionPrompt: CFStringRef;
    fn AXIsProcessTrustedWithOptions(options: core_foundation::dictionary::CFDictionaryRef)
        -> bool;
    fn CGPreflightListenEventAccess() -> bool;
    fn CGRequestListenEventAccess() -> bool;
}
