use flowlink_lib::{input::types::InputError, ui_api::models::UiError};

#[test]
fn input_errors_map_to_recoverable_ui_codes() {
    let unsupported = UiError::from(InputError::Unsupported);
    assert_eq!(unsupported.code, "input_unsupported");
    assert!(unsupported.recoverable);

    let denied = UiError::from(InputError::PermissionDenied("input_monitoring"));
    assert_eq!(denied.code, "input_permission_denied");
    assert!(denied.message.contains("input_monitoring"));
    assert!(denied.recoverable);

    let running = UiError::from(InputError::CaptureAlreadyRunning);
    assert_eq!(running.code, "input_capture_already_running");
    assert!(running.recoverable);

    let platform = UiError::from(InputError::Platform("hook failed".into()));
    assert_eq!(platform.code, "input_platform_error");
    assert!(platform.message.contains("hook failed"));
    assert!(platform.recoverable);
}
