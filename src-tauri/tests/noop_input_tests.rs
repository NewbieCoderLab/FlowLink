use flowlink_lib::{
    input::{
        noop::NoopInputPlatform,
        types::{PermissionKind, Point, RemoteMouseEvent},
        InputPlatform,
    },
    platform::PermissionState,
};
use tokio::sync::mpsc;

#[test]
fn noop_input_platform_reports_unsupported_without_panicking() {
    let platform = NoopInputPlatform;
    let permissions = platform.permissions();

    assert_eq!(permissions.accessibility, PermissionState::Unsupported);
    assert_eq!(permissions.input_monitoring, PermissionState::Unsupported);
    assert_eq!(permissions.screen_recording, PermissionState::Unsupported);
    assert_eq!(permissions.windows_input, PermissionState::Unsupported);
    assert_eq!(permissions.windows_integrity_level, None);
    assert!(!permissions.can_capture_mouse);
    assert!(!permissions.can_inject_mouse);
}

#[test]
fn noop_input_platform_returns_empty_topology() {
    let platform = NoopInputPlatform;
    let topology = platform
        .screen_topology()
        .expect("noop topology should exist");

    assert!(topology.displays.is_empty());
    assert_eq!(topology.virtual_bounds.width, 0.0);
    assert_eq!(topology.virtual_bounds.height, 0.0);
}

#[test]
fn noop_input_platform_rejects_platform_actions() {
    let platform = NoopInputPlatform;
    let (tx, _rx) = mpsc::channel(1);

    assert!(platform
        .request_permissions(PermissionKind::WindowsInput)
        .is_err());
    assert!(platform.start_capture(tx).is_err());
    assert!(platform
        .inject(RemoteMouseEvent::Move { dx: 1.0, dy: 1.0 })
        .is_err());
    assert!(platform.warp_cursor(Point { x: 0.0, y: 0.0 }).is_err());
}
