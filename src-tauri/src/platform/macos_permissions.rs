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
    let mut last_error = None;
    for pane in settings_pane_urls(kind) {
        match open_settings_url(pane) {
            Ok(()) => return Ok(()),
            Err(err) => last_error = Some(err),
        }
    }

    Err(last_error.unwrap_or_else(|| "no settings pane URL configured".into()))
}

#[cfg(target_os = "macos")]
fn settings_pane_urls(kind: &str) -> &'static [&'static str] {
    match kind {
        "input_monitoring" => &[
            "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent",
            "x-apple.systempreferences:com.apple.preference.security?Privacy",
        ],
        "accessibility" => &[
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
            "x-apple.systempreferences:com.apple.preference.security?Privacy",
        ],
        _ => &["x-apple.systempreferences:com.apple.preference.security?Privacy"],
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn accessibility_settings_urls_target_accessibility_first() {
        let urls = settings_pane_urls("accessibility");

        assert_eq!(
            urls[0],
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
        );
        assert!(urls.contains(&"x-apple.systempreferences:com.apple.preference.security?Privacy"));
    }

    #[test]
    fn input_monitoring_settings_urls_target_listen_event_first() {
        let urls = settings_pane_urls("input_monitoring");

        assert_eq!(
            urls[0],
            "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent"
        );
        assert!(urls.contains(&"x-apple.systempreferences:com.apple.preference.security?Privacy"));
    }
}

#[cfg(target_os = "macos")]
fn open_settings_url(url: &str) -> Result<(), String> {
    Command::new("open")
        .arg(url)
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
