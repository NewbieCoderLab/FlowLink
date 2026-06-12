#[cfg(target_os = "macos")]
use std::{sync::mpsc as std_mpsc, thread};

#[cfg(target_os = "macos")]
use core_foundation::runloop::CFRunLoop;
#[cfg(target_os = "macos")]
use core_graphics::{
    display::CGDisplay,
    event::{
        CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
        CGEventType, CGMouseButton, CallbackResult, EventField, ScrollEventUnit,
    },
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};
#[cfg(target_os = "macos")]
use tokio::sync::mpsc;

#[cfg(target_os = "macos")]
use crate::{
    input::{
        types::{
            CaptureHandle, DisplayInfo, InputError, InputResult, LocalMouseEvent, PermissionKind,
            Point, Rect, RemoteMouseEvent, ScreenTopology,
        },
        InputPlatform,
    },
    platform::{macos_permissions, PermissionState, PermissionStatus},
    protocol::messages::MouseButton,
    storage::files::now_ms,
};

#[cfg(target_os = "macos")]
const FLOW_TAG: i64 = 0x464c4f57;

#[cfg(target_os = "macos")]
pub fn platform_name() -> &'static str {
    "macos"
}

#[cfg(target_os = "macos")]
pub struct MacInputPlatform;

#[cfg(target_os = "macos")]
impl MacInputPlatform {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "macos")]
impl Default for MacInputPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "macos")]
impl InputPlatform for MacInputPlatform {
    fn permissions(&self) -> PermissionStatus {
        macos_permission_status()
    }

    fn request_permissions(&self, kind: PermissionKind) -> InputResult<()> {
        match kind {
            PermissionKind::Accessibility => macos_permissions::request_accessibility(),
            PermissionKind::InputMonitoring => macos_permissions::request_input_monitoring(),
            PermissionKind::WindowsInput => Err("windows input is not available on macOS".into()),
        }
        .map_err(InputError::Platform)
    }

    fn screen_topology(&self) -> InputResult<ScreenTopology> {
        screen_topology()
    }

    fn start_capture(&self, tx: mpsc::Sender<LocalMouseEvent>) -> InputResult<CaptureHandle> {
        let (run_loop_tx, run_loop_rx) = std_mpsc::channel();
        let join_handle = thread::Builder::new()
            .name("flowlink-mac-input-capture".into())
            .spawn(move || {
                let run_loop = CFRunLoop::get_current();
                let _ = run_loop_tx.send(run_loop.clone());
                let events = vec![
                    CGEventType::MouseMoved,
                    CGEventType::LeftMouseDown,
                    CGEventType::LeftMouseUp,
                    CGEventType::RightMouseDown,
                    CGEventType::RightMouseUp,
                    CGEventType::OtherMouseDown,
                    CGEventType::OtherMouseUp,
                    CGEventType::ScrollWheel,
                ];

                let result = CGEventTap::with_enabled(
                    CGEventTapLocation::Session,
                    CGEventTapPlacement::HeadInsertEventTap,
                    CGEventTapOptions::ListenOnly,
                    events,
                    move |_proxy, event_type, event| {
                        if event.get_integer_value_field(EventField::EVENT_SOURCE_USER_DATA)
                            == FLOW_TAG
                        {
                            return CallbackResult::Keep;
                        }

                        if let Some(local_event) = local_event_from_cg(event_type, event) {
                            let _ = tx.try_send(local_event);
                        }
                        CallbackResult::Keep
                    },
                    CFRunLoop::run_current,
                );
                if result.is_err() {
                    tracing::error!("failed to create macOS event tap");
                }
            })
            .map_err(|err| InputError::Platform(err.to_string()))?;

        let run_loop = run_loop_rx
            .recv()
            .map_err(|err| InputError::Platform(format!("capture run loop unavailable: {err}")))?;

        Ok(CaptureHandle::new(join_handle, move || {
            run_loop.stop();
        }))
    }

    fn inject(&self, event: RemoteMouseEvent) -> InputResult<()> {
        inject_event(event)
    }

    fn warp_cursor(&self, position: Point) -> InputResult<()> {
        CGDisplay::warp_mouse_cursor_position(CGPoint::new(position.x, position.y))
            .map_err(|err| InputError::Platform(format!("CGWarpMouseCursorPosition failed: {err}")))
    }
}

#[cfg(target_os = "macos")]
fn macos_permission_status() -> PermissionStatus {
    let accessibility = macos_permissions::accessibility_status();
    let input_monitoring = macos_permissions::input_monitoring_status();

    PermissionStatus {
        accessibility,
        input_monitoring,
        screen_recording: PermissionState::Unsupported,
        windows_input: PermissionState::Unsupported,
        windows_integrity_level: None,
        can_capture_mouse: input_monitoring == PermissionState::Granted,
        can_inject_mouse: accessibility == PermissionState::Granted,
        updated_at_ms: now_ms(),
    }
}

#[cfg(target_os = "macos")]
fn local_event_from_cg(event_type: CGEventType, event: &CGEvent) -> Option<LocalMouseEvent> {
    let location = event.location();
    let ts_ms = now_ms();
    match event_type {
        CGEventType::MouseMoved => Some(LocalMouseEvent::Move {
            x: location.x,
            y: location.y,
            dx: event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_X) as f64,
            dy: event.get_integer_value_field(EventField::MOUSE_EVENT_DELTA_Y) as f64,
            ts_ms,
        }),
        CGEventType::LeftMouseDown | CGEventType::RightMouseDown | CGEventType::OtherMouseDown => {
            Some(LocalMouseEvent::Down {
                button: mouse_button(event_type, event),
                x: location.x,
                y: location.y,
                ts_ms,
            })
        }
        CGEventType::LeftMouseUp | CGEventType::RightMouseUp | CGEventType::OtherMouseUp => {
            Some(LocalMouseEvent::Up {
                button: mouse_button(event_type, event),
                x: location.x,
                y: location.y,
                ts_ms,
            })
        }
        CGEventType::ScrollWheel => Some(LocalMouseEvent::Wheel {
            // CoreGraphics axis 1 is vertical and axis 2 is horizontal. Point
            // deltas align better with logical-pixel movement than raw wheel
            // ticks on high-resolution trackpads.
            dx: event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2)
                as f64,
            dy: event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1)
                as f64,
            ts_ms,
        }),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn mouse_button(event_type: CGEventType, event: &CGEvent) -> MouseButton {
    match event_type {
        CGEventType::LeftMouseDown | CGEventType::LeftMouseUp => MouseButton::Left,
        CGEventType::RightMouseDown | CGEventType::RightMouseUp => MouseButton::Right,
        _ => match event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER) {
            2 => MouseButton::Middle,
            other => MouseButton::Other(other as u8),
        },
    }
}

#[cfg(target_os = "macos")]
fn inject_event(event: RemoteMouseEvent) -> InputResult<()> {
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .map_err(|_| InputError::Platform("failed to create CGEventSource".into()))?;

    match event {
        RemoteMouseEvent::Move { dx, dy } => {
            let current = CGEvent::new(source.clone())
                .map_err(|_| InputError::Platform("failed to create CGEvent".into()))?
                .location();
            post_mouse_event(
                source,
                CGEventType::MouseMoved,
                CGPoint::new(current.x + dx, current.y + dy),
                CGMouseButton::Left,
                0,
            )
        }
        RemoteMouseEvent::MoveTo { x, y } => post_mouse_event(
            source,
            CGEventType::MouseMoved,
            CGPoint::new(x, y),
            CGMouseButton::Left,
            0,
        ),
        RemoteMouseEvent::Down { button, x, y } => {
            let (event_type, mouse_button) = mac_button_event(button, true);
            post_mouse_event(
                source,
                event_type,
                CGPoint::new(x, y),
                mouse_button,
                mac_button_number(button),
            )
        }
        RemoteMouseEvent::Up { button, x, y } => {
            let (event_type, mouse_button) = mac_button_event(button, false);
            post_mouse_event(
                source,
                event_type,
                CGPoint::new(x, y),
                mouse_button,
                mac_button_number(button),
            )
        }
        RemoteMouseEvent::Wheel { dx, dy } => {
            let event = CGEvent::new_scroll_event(
                source,
                ScrollEventUnit::PIXEL,
                2,
                dy.round() as i32,
                dx.round() as i32,
                0,
            )
            .map_err(|_| InputError::Platform("failed to create scroll event".into()))?;
            event.set_integer_value_field(EventField::EVENT_SOURCE_USER_DATA, FLOW_TAG);
            event.post(CGEventTapLocation::HID);
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
fn post_mouse_event(
    source: CGEventSource,
    event_type: CGEventType,
    position: CGPoint,
    button: CGMouseButton,
    button_number: i64,
) -> InputResult<()> {
    let event = CGEvent::new_mouse_event(source, event_type, position, button)
        .map_err(|_| InputError::Platform("failed to create mouse event".into()))?;
    event.set_integer_value_field(EventField::EVENT_SOURCE_USER_DATA, FLOW_TAG);
    event.set_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER, button_number);
    event.post(CGEventTapLocation::HID);
    Ok(())
}

#[cfg(target_os = "macos")]
fn mac_button_event(button: MouseButton, down: bool) -> (CGEventType, CGMouseButton) {
    match button {
        MouseButton::Left => (
            if down {
                CGEventType::LeftMouseDown
            } else {
                CGEventType::LeftMouseUp
            },
            CGMouseButton::Left,
        ),
        MouseButton::Right => (
            if down {
                CGEventType::RightMouseDown
            } else {
                CGEventType::RightMouseUp
            },
            CGMouseButton::Right,
        ),
        MouseButton::Middle | MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => (
            if down {
                CGEventType::OtherMouseDown
            } else {
                CGEventType::OtherMouseUp
            },
            CGMouseButton::Center,
        ),
    }
}

#[cfg(target_os = "macos")]
fn mac_button_number(button: MouseButton) -> i64 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Back => 3,
        MouseButton::Forward => 4,
        MouseButton::Other(value) => value as i64,
    }
}

#[cfg(target_os = "macos")]
fn screen_topology() -> InputResult<ScreenTopology> {
    let display_ids = CGDisplay::active_displays()
        .map_err(|err| InputError::Platform(format!("CGGetActiveDisplayList failed: {err}")))?;

    let main_id = CGDisplay::main().id;
    let displays = display_ids
        .into_iter()
        .map(|id| {
            let display = CGDisplay::new(id);
            let bounds = display.bounds();
            DisplayInfo {
                id: id as u64,
                bounds: Rect {
                    x: bounds.origin.x,
                    y: bounds.origin.y,
                    width: bounds.size.width,
                    height: bounds.size.height,
                },
                scale_factor: display_scale_factor(&display),
                is_primary: id == main_id,
            }
        })
        .collect::<Vec<_>>();

    Ok(ScreenTopology {
        virtual_bounds: virtual_bounds(&displays),
        displays,
    })
}

#[cfg(target_os = "macos")]
fn display_scale_factor(display: &CGDisplay) -> f64 {
    display
        .display_mode()
        .map(|mode| {
            let width = mode.width().max(1) as f64;
            mode.pixel_width() as f64 / width
        })
        .unwrap_or(1.0)
}

#[cfg(target_os = "macos")]
fn virtual_bounds(displays: &[DisplayInfo]) -> Rect {
    let Some(first) = displays.first() else {
        return Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
    };

    let mut left = first.bounds.x;
    let mut top = first.bounds.y;
    let mut right = first.bounds.x + first.bounds.width;
    let mut bottom = first.bounds.y + first.bounds.height;

    for display in displays.iter().skip(1) {
        left = left.min(display.bounds.x);
        top = top.min(display.bounds.y);
        right = right.max(display.bounds.x + display.bounds.width);
        bottom = bottom.max(display.bounds.y + display.bounds.height);
    }

    Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}
