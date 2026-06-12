#[cfg(target_os = "windows")]
use std::{
    mem::size_of,
    sync::{Mutex, OnceLock},
    thread,
};

#[cfg(target_os = "windows")]
use tokio::sync::mpsc;

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{BOOL, HINSTANCE, LPARAM, LRESULT, POINT, RECT, WPARAM},
    Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
    UI::{
        HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_HWHEEL,
            MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP,
            MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK,
            MOUSEEVENTF_WHEEL, MOUSEINPUT, WHEEL_DELTA,
        },
        WindowsAndMessaging::{
            CallNextHookEx, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, MSG,
            MSLLHOOKSTRUCT, WH_MOUSE_LL, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
            WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN,
            WM_RBUTTONUP, WM_XBUTTONDOWN, WM_XBUTTONUP, XBUTTON1, XBUTTON2,
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
    platform::PermissionStatus,
    protocol::messages::MouseButton,
    storage::files::now_ms,
};

#[cfg(target_os = "windows")]
const FLOW_SIGNATURE: usize = 0xF10F_1117;

#[cfg(target_os = "windows")]
static CAPTURE_TX: OnceLock<Mutex<Option<mpsc::Sender<LocalMouseEvent>>>> = OnceLock::new();

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
impl Default for WinInputPlatform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
impl InputPlatform for WinInputPlatform {
    fn permissions(&self) -> PermissionStatus {
        PermissionStatus::from_identity(&crate::identity::DeviceIdentity::generate())
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

        let join_handle = thread::Builder::new()
            .name("flowlink-win-input-capture".into())
            .spawn(move || unsafe {
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

        Ok(CaptureHandle::detached(join_handle))
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
    if event.dwExtraInfo == FLOW_SIGNATURE {
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
fn run_message_loop(hook: HHOOK) {
    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {}
    let _ = unsafe { UnhookWindowsHookEx(hook) };
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
            send_mouse_input(0, 0, 0, button_flag(button, true))
        }
        RemoteMouseEvent::Up { button, .. } => {
            send_mouse_input(0, 0, 0, button_flag(button, false))
        }
        RemoteMouseEvent::Wheel { dx, dy } => {
            if dy != 0.0 {
                send_mouse_input(
                    0,
                    0,
                    (dy.round() as i32 * WHEEL_DELTA) as u32,
                    MOUSEEVENTF_WHEEL,
                )?;
            }
            if dx != 0.0 {
                send_mouse_input(
                    0,
                    0,
                    (dx.round() as i32 * WHEEL_DELTA) as u32,
                    MOUSEEVENTF_HWHEEL,
                )?;
            }
            Ok(())
        }
    }
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
fn button_flag(
    button: MouseButton,
    down: bool,
) -> windows::Win32::UI::Input::KeyboardAndMouse::MOUSE_EVENT_FLAGS {
    match (button, down) {
        (MouseButton::Left, true) => MOUSEEVENTF_LEFTDOWN,
        (MouseButton::Left, false) => MOUSEEVENTF_LEFTUP,
        (MouseButton::Right, true) => MOUSEEVENTF_RIGHTDOWN,
        (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP,
        (
            MouseButton::Middle | MouseButton::Back | MouseButton::Forward | MouseButton::Other(_),
            true,
        ) => MOUSEEVENTF_MIDDLEDOWN,
        (
            MouseButton::Middle | MouseButton::Back | MouseButton::Forward | MouseButton::Other(_),
            false,
        ) => MOUSEEVENTF_MIDDLEUP,
    }
}

#[cfg(target_os = "windows")]
fn normalize_virtual_position(x: f64, y: f64) -> InputResult<(i32, i32)> {
    let bounds = virtual_screen_bounds();
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
