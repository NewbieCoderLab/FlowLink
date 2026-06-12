#[cfg(target_os = "windows")]
use std::{
    mem::size_of,
    sync::mpsc as std_mpsc,
    sync::{Mutex, OnceLock},
    thread,
};

#[cfg(target_os = "windows")]
use tokio::sync::mpsc;

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{BOOL, HINSTANCE, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
    UI::{
        HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_HWHEEL,
            MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
            MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK,
            MOUSEEVENTF_WHEEL, MOUSEINPUT,
        },
        WindowsAndMessaging::{
            CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
            UnhookWindowsHookEx, HHOOK, MSG, MSLLHOOKSTRUCT, WHEEL_DELTA, WH_MOUSE_LL,
            WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL,
            WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN,
            WM_XBUTTONUP, XBUTTON1, XBUTTON2,
        },
    },
};

#[cfg(target_os = "windows")]
use crate::{
    input::{
        types::{
            CaptureHandle, DisplayInfo, InputError, InputResult, LocalMouseEvent, PermissionKind,
            Point, Rect, RemoteMouseEvent, ScreenTopology,
        },
        InputPlatform,
    },
    platform::{windows_permissions, PermissionStatus},
    protocol::messages::MouseButton,
    storage::files::now_ms,
};

#[cfg(target_os = "windows")]
const FLOW_SIGNATURE: usize = 0xF10F_1117;

#[cfg(target_os = "windows")]
static CAPTURE_TX: OnceLock<Mutex<Option<mpsc::Sender<LocalMouseEvent>>>> = OnceLock::new();

#[cfg(target_os = "windows")]
fn is_self_injected_extra_info(extra_info: usize) -> bool {
    extra_info == FLOW_SIGNATURE
}

#[cfg(target_os = "windows")]
pub fn platform_name() -> &'static str {
    "windows"
}

#[cfg(target_os = "windows")]
pub struct WinInputPlatform;

#[cfg(target_os = "windows")]
impl WinInputPlatform {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "windows")]
pub fn init_process_dpi_awareness_once() {
    enable_dpi_awareness();
}

#[cfg(target_os = "windows")]
impl Default for WinInputPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
impl InputPlatform for WinInputPlatform {
    fn permissions(&self) -> PermissionStatus {
        windows_permissions::permission_status()
    }

    fn request_permissions(&self, _kind: PermissionKind) -> InputResult<()> {
        Ok(())
    }

    fn screen_topology(&self) -> InputResult<ScreenTopology> {
        enable_dpi_awareness();
        screen_topology()
    }

    fn start_capture(&self, tx: mpsc::Sender<LocalMouseEvent>) -> InputResult<CaptureHandle> {
        let sender_slot = CAPTURE_TX.get_or_init(|| Mutex::new(None));
        {
            let mut guard = sender_slot
                .lock()
                .map_err(|_| InputError::Platform("capture sender lock poisoned".into()))?;
            if guard.is_some() {
                return Err(InputError::CaptureAlreadyRunning);
            }
            *guard = Some(tx);
        }

        let (thread_id_tx, thread_id_rx) = std_mpsc::channel();
        let join_handle = thread::Builder::new()
            .name("flowlink-win-input-capture".into())
            .spawn(move || unsafe {
                let thread_id = windows::Win32::System::Threading::GetCurrentThreadId();
                let _ = thread_id_tx.send(thread_id);
                let hook =
                    SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), HINSTANCE::default(), 0);
                let Ok(hook) = hook else {
                    clear_capture_sender();
                    return;
                };
                run_message_loop(hook);
                clear_capture_sender();
            })
            .map_err(|err| InputError::Platform(err.to_string()))?;

        let thread_id = thread_id_rx
            .recv()
            .map_err(|err| InputError::Platform(format!("capture thread unavailable: {err}")))?;

        Ok(CaptureHandle::new(join_handle, move || {
            let _ = unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
        }))
    }

    fn inject(&self, event: RemoteMouseEvent) -> InputResult<()> {
        inject_event(event)
    }

    fn warp_cursor(&self, position: Point) -> InputResult<()> {
        inject_event(RemoteMouseEvent::MoveTo {
            x: position.x,
            y: position.y,
        })
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) };
    }

    let event = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
    if is_self_injected_extra_info(event.dwExtraInfo) {
        return unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) };
    }

    if let Some(local_event) = local_event_from_hook(wparam.0 as u32, event) {
        if let Some(lock) = CAPTURE_TX.get() {
            if let Ok(guard) = lock.lock() {
                if let Some(tx) = guard.as_ref() {
                    let _ = tx.try_send(local_event);
                }
            }
        }
    }

    unsafe { CallNextHookEx(HHOOK::default(), code, wparam, lparam) }
}

#[cfg(target_os = "windows")]
fn local_event_from_hook(message: u32, event: &MSLLHOOKSTRUCT) -> Option<LocalMouseEvent> {
    let x = event.pt.x as f64;
    let y = event.pt.y as f64;
    let ts_ms = now_ms();
    match message {
        WM_MOUSEMOVE => Some(LocalMouseEvent::Move {
            x,
            y,
            dx: 0.0,
            dy: 0.0,
            ts_ms,
        }),
        WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_XBUTTONDOWN => {
            Some(LocalMouseEvent::Down {
                button: hook_button(message, event),
                x,
                y,
                ts_ms,
            })
        }
        WM_LBUTTONUP | WM_RBUTTONUP | WM_MBUTTONUP | WM_XBUTTONUP => Some(LocalMouseEvent::Up {
            button: hook_button(message, event),
            x,
            y,
            ts_ms,
        }),
        WM_MOUSEWHEEL => Some(LocalMouseEvent::Wheel {
            dx: 0.0,
            dy: wheel_delta(event),
            ts_ms,
        }),
        WM_MOUSEHWHEEL => Some(LocalMouseEvent::Wheel {
            dx: wheel_delta(event),
            dy: 0.0,
            ts_ms,
        }),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn hook_button(message: u32, event: &MSLLHOOKSTRUCT) -> MouseButton {
    match message {
        WM_LBUTTONDOWN | WM_LBUTTONUP => MouseButton::Left,
        WM_RBUTTONDOWN | WM_RBUTTONUP => MouseButton::Right,
        WM_MBUTTONDOWN | WM_MBUTTONUP => MouseButton::Middle,
        WM_XBUTTONDOWN | WM_XBUTTONUP => {
            let xbutton = high_word(event.mouseData);
            if xbutton == XBUTTON1 {
                MouseButton::Back
            } else if xbutton == XBUTTON2 {
                MouseButton::Forward
            } else {
                MouseButton::Other(xbutton as u8)
            }
        }
        _ => MouseButton::Other(0),
    }
}

#[cfg(target_os = "windows")]
fn wheel_delta(event: &MSLLHOOKSTRUCT) -> f64 {
    let delta = high_word(event.mouseData) as i16;
    delta as f64 / WHEEL_DELTA as f64
}

#[cfg(target_os = "windows")]
fn high_word(value: u32) -> u16 {
    ((value >> 16) & 0xffff) as u16
}

#[cfg(target_os = "windows")]
unsafe fn run_message_loop(_hook: HHOOK) {
    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {}
    let _ = unsafe { UnhookWindowsHookEx(_hook) };
}

#[cfg(target_os = "windows")]
fn clear_capture_sender() {
    if let Some(lock) = CAPTURE_TX.get() {
        if let Ok(mut guard) = lock.lock() {
            *guard = None;
        }
    }
}

#[cfg(target_os = "windows")]
fn inject_event(event: RemoteMouseEvent) -> InputResult<()> {
    enable_dpi_awareness();
    match event {
        RemoteMouseEvent::Move { dx, dy } => {
            send_mouse_input(dx.round() as i32, dy.round() as i32, 0, MOUSEEVENTF_MOVE)
        }
        RemoteMouseEvent::MoveTo { x, y } => {
            let (nx, ny) = normalize_virtual_position(x, y)?;
            send_mouse_input(
                nx,
                ny,
                0,
                MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
            )
        }
        RemoteMouseEvent::Down { button, .. } => {
            let (mouse_data, flags) = button_input(button, true);
            send_mouse_input(0, 0, mouse_data, flags)
        }
        RemoteMouseEvent::Up { button, .. } => {
            let (mouse_data, flags) = button_input(button, false);
            send_mouse_input(0, 0, mouse_data, flags)
        }
        RemoteMouseEvent::Wheel { dx, dy } => {
            for (mouse_data, flags) in wheel_inputs(dx, dy) {
                send_mouse_input(0, 0, mouse_data, flags)?;
            }
            Ok(())
        }
    }
}

#[cfg(target_os = "windows")]
fn wheel_inputs(
    dx: f64,
    dy: f64,
) -> Vec<(
    u32,
    windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS,
)> {
    let mut inputs = Vec::with_capacity(2);
    if dy != 0.0 {
        inputs.push((wheel_mouse_data(dy), MOUSEEVENTF_WHEEL));
    }
    if dx != 0.0 {
        inputs.push((wheel_mouse_data(dx), MOUSEEVENTF_HWHEEL));
    }
    inputs
}

#[cfg(target_os = "windows")]
fn wheel_mouse_data(notches: f64) -> u32 {
    (notches.round() as i32 * WHEEL_DELTA as i32) as u32
}

#[cfg(target_os = "windows")]
fn send_mouse_input(
    dx: i32,
    dy: i32,
    mouse_data: u32,
    flags: windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS,
) -> InputResult<()> {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: mouse_data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: FLOW_SIGNATURE,
            },
        },
    };
    let sent = unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
    if sent == 1 {
        Ok(())
    } else {
        Err(InputError::Platform("SendInput returned 0".into()))
    }
}

#[cfg(target_os = "windows")]
fn button_input(
    button: MouseButton,
    down: bool,
) -> (
    u32,
    windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS,
) {
    use windows::Win32::UI::Input::KeyboardAndMouse::{MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP};

    match (button, down) {
        (MouseButton::Left, true) => (0, MOUSEEVENTF_LEFTDOWN),
        (MouseButton::Left, false) => (0, MOUSEEVENTF_LEFTUP),
        (MouseButton::Right, true) => (0, MOUSEEVENTF_RIGHTDOWN),
        (MouseButton::Right, false) => (0, MOUSEEVENTF_RIGHTUP),
        (MouseButton::Middle | MouseButton::Other(_), true) => (0, MOUSEEVENTF_MIDDLEDOWN),
        (MouseButton::Middle | MouseButton::Other(_), false) => (0, MOUSEEVENTF_MIDDLEUP),
        (MouseButton::Back, true) => (XBUTTON1 as u32, MOUSEEVENTF_XDOWN),
        (MouseButton::Back, false) => (XBUTTON1 as u32, MOUSEEVENTF_XUP),
        (MouseButton::Forward, true) => (XBUTTON2 as u32, MOUSEEVENTF_XDOWN),
        (MouseButton::Forward, false) => (XBUTTON2 as u32, MOUSEEVENTF_XUP),
    }
}

#[cfg(target_os = "windows")]
fn normalize_virtual_position(x: f64, y: f64) -> InputResult<(i32, i32)> {
    let bounds = virtual_screen_bounds();
    normalize_position_in_bounds(x, y, bounds)
}

#[cfg(target_os = "windows")]
fn normalize_position_in_bounds(x: f64, y: f64, bounds: Rect) -> InputResult<(i32, i32)> {
    if bounds.width <= 1.0 || bounds.height <= 1.0 {
        return Err(InputError::Platform("invalid virtual screen bounds".into()));
    }

    let nx = ((x - bounds.x) * 65_535.0 / (bounds.width - 1.0)).round() as i32;
    let ny = ((y - bounds.y) * 65_535.0 / (bounds.height - 1.0)).round() as i32;
    Ok((nx.clamp(0, 65_535), ny.clamp(0, 65_535)))
}

#[cfg(target_os = "windows")]
fn screen_topology() -> InputResult<ScreenTopology> {
    let mut displays = Vec::<DisplayInfo>::new();
    let ok = unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut displays as *mut _ as isize),
        )
    };
    if !ok.as_bool() {
        return Err(InputError::Platform("EnumDisplayMonitors failed".into()));
    }

    Ok(ScreenTopology {
        virtual_bounds: virtual_bounds(&displays),
        displays,
    })
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let displays = unsafe { &mut *(data.0 as *mut Vec<DisplayInfo>) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if unsafe { GetMonitorInfoW(hmonitor, &mut info) }.as_bool() {
        let rect = info.rcMonitor;
        displays.push(DisplayInfo {
            id: hmonitor.0 as u64,
            bounds: Rect {
                x: rect.left as f64,
                y: rect.top as f64,
                width: (rect.right - rect.left) as f64,
                height: (rect.bottom - rect.top) as f64,
            },
            scale_factor: 1.0,
            is_primary: info.dwFlags & 1 == 1,
        });
    }
    true.into()
}

#[cfg(target_os = "windows")]
fn virtual_bounds(displays: &[DisplayInfo]) -> Rect {
    let Some(first) = displays.first() else {
        return virtual_screen_bounds();
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

#[cfg(target_os = "windows")]
fn virtual_screen_bounds() -> Rect {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };

    Rect {
        x: unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) } as f64,
        y: unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) } as f64,
        width: unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) } as f64,
        height: unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) } as f64,
    }
}

#[cfg(target_os = "windows")]
fn enable_dpi_awareness() {
    static ENABLE_DPI: OnceLock<()> = OnceLock::new();
    ENABLE_DPI.get_or_init(|| unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    });
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn button_input_maps_back_forward_to_xbutton_flags_and_data() {
        let back_down = button_input(MouseButton::Back, true);
        let back_up = button_input(MouseButton::Back, false);
        let forward_down = button_input(MouseButton::Forward, true);
        let forward_up = button_input(MouseButton::Forward, false);

        assert_eq!(
            back_down,
            (
                XBUTTON1 as u32,
                windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_XDOWN
            )
        );
        assert_eq!(
            back_up,
            (
                XBUTTON1 as u32,
                windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_XUP
            )
        );
        assert_eq!(
            forward_down,
            (
                XBUTTON2 as u32,
                windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_XDOWN
            )
        );
        assert_eq!(
            forward_up,
            (
                XBUTTON2 as u32,
                windows::Win32::UI::Input::KeyboardAndMouse::MOUSEEVENTF_XUP
            )
        );
    }

    #[test]
    fn self_injected_extra_info_matches_flow_signature_only() {
        assert!(is_self_injected_extra_info(FLOW_SIGNATURE));
        assert!(!is_self_injected_extra_info(0));
        assert!(!is_self_injected_extra_info(FLOW_SIGNATURE + 1));
    }

    #[test]
    fn wheel_inputs_map_logical_axes_to_windows_flags() {
        let inputs = wheel_inputs(1.0, -1.0);

        assert_eq!(inputs.len(), 2);
        assert_eq!(
            inputs[0],
            ((-(WHEEL_DELTA as i32)) as u32, MOUSEEVENTF_WHEEL)
        );
        assert_eq!(inputs[1], (WHEEL_DELTA, MOUSEEVENTF_HWHEEL));
    }

    #[test]
    fn wheel_inputs_round_to_wheel_delta_notches() {
        let inputs = wheel_inputs(-2.0, 2.0);

        assert_eq!(inputs[0], (WHEEL_DELTA * 2, MOUSEEVENTF_WHEEL));
        assert_eq!(
            inputs[1],
            ((-((WHEEL_DELTA * 2) as i32)) as u32, MOUSEEVENTF_HWHEEL)
        );
    }

    #[test]
    fn dpi_awareness_init_is_safe_to_call_repeatedly() {
        init_process_dpi_awareness_once();
        init_process_dpi_awareness_once();
    }

    #[test]
    fn normalizes_absolute_coordinates_against_virtual_desktop_bounds() {
        let bounds = Rect {
            x: -1920.0,
            y: 0.0,
            width: 3840.0,
            height: 1080.0,
        };

        assert_eq!(
            normalize_position_in_bounds(-1920.0, 0.0, bounds).unwrap(),
            (0, 0)
        );
        assert_eq!(
            normalize_position_in_bounds(1919.0, 1079.0, bounds).unwrap(),
            (65_535, 65_535)
        );
        assert_eq!(
            normalize_position_in_bounds(0.0, 540.0, bounds).unwrap(),
            (32_776, 32_798)
        );
    }

    #[test]
    fn rejects_invalid_virtual_desktop_bounds() {
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1080.0,
        };

        assert!(normalize_position_in_bounds(0.0, 0.0, bounds).is_err());
    }
}
