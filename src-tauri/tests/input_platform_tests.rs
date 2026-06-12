use flowlink_lib::input::platform_input;

#[test]
fn platform_input_resolves_for_current_target() {
    let permissions = platform_input().permissions();

    #[cfg(target_os = "macos")]
    {
        assert_ne!(
            permissions.accessibility,
            flowlink_lib::platform::PermissionState::Unsupported
        );
    }

    #[cfg(target_os = "windows")]
    {
        assert_ne!(
            permissions.windows_input,
            flowlink_lib::platform::PermissionState::Unsupported
        );
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        assert_eq!(
            permissions.accessibility,
            flowlink_lib::platform::PermissionState::Unsupported
        );
        assert_eq!(
            permissions.windows_input,
            flowlink_lib::platform::PermissionState::Unsupported
        );
    }
}
