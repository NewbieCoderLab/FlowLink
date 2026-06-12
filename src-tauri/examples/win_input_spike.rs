#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use flowlink_lib::protocol::messages::MouseButton;
#[cfg(target_os = "windows")]
use flowlink_lib::{
    input::{
        types::{LocalMouseEvent, PermissionKind, RemoteMouseEvent, ScreenTopology},
        windows::WinInputPlatform,
        InputPlatform,
    },
    platform::PermissionState,
};

#[cfg(target_os = "windows")]
#[tokio::main]
async fn main() {
    let inject_move = std::env::args().any(|arg| arg == "--inject-move");
    let inject_click = std::env::args().any(|arg| arg == "--inject-click");
    let inject_wheel = std::env::args().any(|arg| arg == "--inject-wheel");
    let verify_self_filter = std::env::args().any(|arg| arg == "--verify-self-filter");
    let print_topology = std::env::args().any(|arg| arg == "--topology");
    let warp_corners = std::env::args().any(|arg| arg == "--warp-corners");
    let platform = WinInputPlatform::new();
    let permissions = platform.permissions();
    println!("permissions: windows_input={:?}", permissions.windows_input);

    if permissions.windows_input != PermissionState::Granted {
        let _ = platform.request_permissions(PermissionKind::WindowsInput);
    }

    match platform.screen_topology() {
        Ok(topology) if print_topology => print_screen_topology(&topology),
        Ok(topology) => println!(
            "screen topology: {} display(s), virtual bounds {:?}",
            topology.displays.len(),
            topology.virtual_bounds
        ),
        Err(err) => eprintln!("failed to read screen topology: {err}"),
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel(256);
    let _capture = platform
        .start_capture(tx)
        .expect("failed to install WH_MOUSE_LL hook");

    let started = Instant::now();
    let mut count = 0usize;
    while count < 1000 {
        if let Some(event) = rx.recv().await {
            count += 1;
            println!("{}", format_event_line(count, &event));
        }
    }
    println!(
        "captured {count} events in {:.2?}",
        Instant::now().duration_since(started)
    );

    if inject_move {
        run_move_injection_spike(&platform).await;
    }
    if inject_click {
        run_click_injection_spike(&platform).await;
    }
    if inject_wheel {
        run_wheel_injection_spike(&platform).await;
    }
    if verify_self_filter {
        run_self_filter_spike(&platform, &mut rx).await;
    }
    if warp_corners {
        run_warp_corners_spike(&platform).await;
    }
    if !inject_move
        && !inject_click
        && !inject_wheel
        && !verify_self_filter
        && !warp_corners
        && !print_topology
    {
        println!(
            "run again with --inject-move, --inject-click, --inject-wheel, --verify-self-filter, --topology, or --warp-corners."
        );
    }
}

#[cfg(target_os = "windows")]
async fn run_move_injection_spike(platform: &WinInputPlatform) {
    println!("injecting 100 visible mouse moves via SendInput...");
    let plan = move_injection_plan();
    let mut latencies = Vec::with_capacity(plan.len());
    for (index, event) in plan.into_iter().enumerate() {
        let before = Instant::now();
        platform.inject(event).expect("SendInput failed");
        latencies.push(before.elapsed());
        if (index + 1) % 25 == 0 {
            println!("injected {} moves", index + 1);
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    latencies.sort();
    let median = latencies[latencies.len() / 2];
    let p95 = latencies[(latencies.len() * 95 / 100).min(latencies.len() - 1)];
    println!("injected 100 moves");
    println!("median call latency: {median:?}");
    println!("p95 call latency: {p95:?}");
}

#[cfg(target_os = "windows")]
fn move_injection_plan() -> Vec<RemoteMouseEvent> {
    (0..100)
        .map(|_| RemoteMouseEvent::Move { dx: 4.0, dy: 0.0 })
        .collect()
}

#[cfg(target_os = "windows")]
fn print_screen_topology(topology: &ScreenTopology) {
    println!("screen topology:");
    println!("  virtual bounds: {:?}", topology.virtual_bounds);
    for display in &topology.displays {
        println!(
            "  display id={} primary={} scale={} bounds={:?}",
            display.id, display.is_primary, display.scale_factor, display.bounds
        );
    }
}

#[cfg(target_os = "windows")]
async fn run_warp_corners_spike(platform: &WinInputPlatform) {
    let topology = platform
        .screen_topology()
        .expect("failed to read screen topology before corner warp");
    let points = virtual_corner_points(&topology);
    println!(
        "warping cursor through {} virtual desktop corners...",
        points.len()
    );
    for (index, (x, y)) in points.into_iter().enumerate() {
        println!("warp {}: x={x:.1}, y={y:.1}", index + 1);
        platform
            .warp_cursor(flowlink_lib::input::types::Point { x, y })
            .expect("failed to warp cursor");
        tokio::time::sleep(Duration::from_millis(700)).await;
    }
}

#[cfg(target_os = "windows")]
fn virtual_corner_points(topology: &ScreenTopology) -> Vec<(f64, f64)> {
    let bounds = topology.virtual_bounds;
    if bounds.width <= 1.0 || bounds.height <= 1.0 {
        return Vec::new();
    }

    let left = bounds.x;
    let top = bounds.y;
    let right = bounds.x + bounds.width - 1.0;
    let bottom = bounds.y + bounds.height - 1.0;
    vec![(left, top), (right, top), (right, bottom), (left, bottom)]
}

#[cfg(target_os = "windows")]
async fn run_click_injection_spike(platform: &WinInputPlatform) {
    println!(
        "move the cursor over a safe target; injecting left/right/middle clicks in 2 seconds..."
    );
    tokio::time::sleep(Duration::from_secs(2)).await;

    let (x, y) = current_cursor_position();
    let plan = click_injection_plan(x, y);
    let mut latencies = Vec::with_capacity(plan.len());
    for (index, event) in plan.into_iter().enumerate() {
        let label = click_event_label(&event);
        let before = Instant::now();
        platform.inject(event).expect("SendInput click failed");
        latencies.push(before.elapsed());
        println!("injected {label}");
        if index % 2 == 1 {
            tokio::time::sleep(Duration::from_millis(250)).await;
        } else {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    latencies.sort();
    let median = latencies[latencies.len() / 2];
    println!("injected left/right/middle clicks at x={x:.1}, y={y:.1}");
    println!("median click SendInput latency: {median:?}");
}

#[cfg(target_os = "windows")]
fn click_injection_plan(x: f64, y: f64) -> Vec<RemoteMouseEvent> {
    [MouseButton::Left, MouseButton::Right, MouseButton::Middle]
        .into_iter()
        .flat_map(|button| {
            [
                RemoteMouseEvent::Down { button, x, y },
                RemoteMouseEvent::Up { button, x, y },
            ]
        })
        .collect()
}

#[cfg(target_os = "windows")]
fn click_event_label(event: &RemoteMouseEvent) -> String {
    match event {
        RemoteMouseEvent::Down { button, .. } => format!("{button:?} down"),
        RemoteMouseEvent::Up { button, .. } => format!("{button:?} up"),
        _ => "non-click event".into(),
    }
}

#[cfg(target_os = "windows")]
async fn run_wheel_injection_spike(platform: &WinInputPlatform) {
    println!("move the cursor over a scrollable target; injecting wheel/hwheel in 2 seconds...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    let plan = wheel_injection_plan();
    let mut latencies = Vec::with_capacity(plan.len());
    for event in plan {
        let label = wheel_event_label(&event);
        let before = Instant::now();
        platform.inject(event).expect("SendInput wheel failed");
        latencies.push(before.elapsed());
        println!("injected {label}");
        tokio::time::sleep(Duration::from_millis(350)).await;
    }

    latencies.sort();
    let median = latencies[latencies.len() / 2];
    println!("injected vertical and horizontal wheel checks");
    println!("median wheel SendInput latency: {median:?}");
}

#[cfg(target_os = "windows")]
fn wheel_injection_plan() -> Vec<RemoteMouseEvent> {
    vec![
        RemoteMouseEvent::Wheel { dx: 0.0, dy: 1.0 },
        RemoteMouseEvent::Wheel { dx: 0.0, dy: -1.0 },
        RemoteMouseEvent::Wheel { dx: 1.0, dy: 0.0 },
        RemoteMouseEvent::Wheel { dx: -1.0, dy: 0.0 },
    ]
}

#[cfg(target_os = "windows")]
fn wheel_event_label(event: &RemoteMouseEvent) -> String {
    match event {
        RemoteMouseEvent::Wheel { dx, dy } if *dy > 0.0 => format!("vertical wheel up dy={dy}"),
        RemoteMouseEvent::Wheel { dx, dy } if *dy < 0.0 => format!("vertical wheel down dy={dy}"),
        RemoteMouseEvent::Wheel { dx, dy } if *dx > 0.0 => {
            format!("horizontal wheel right dx={dx}")
        }
        RemoteMouseEvent::Wheel { dx, dy } if *dx < 0.0 => format!("horizontal wheel left dx={dx}"),
        _ => "zero wheel event".into(),
    }
}

#[cfg(target_os = "windows")]
async fn run_self_filter_spike(
    platform: &WinInputPlatform,
    rx: &mut tokio::sync::mpsc::Receiver<LocalMouseEvent>,
) {
    println!("draining queued local events before self-filter check...");
    let drained = drain_events(rx).await;
    println!("drained {drained} queued events");

    println!("injecting 20 tagged SendInput moves; hook should filter them out...");
    for _ in 0..20 {
        platform
            .inject(RemoteMouseEvent::Move { dx: 2.0, dy: 0.0 })
            .expect("SendInput self-filter move failed");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    tokio::time::sleep(Duration::from_millis(200)).await;
    let leaked = drain_events(rx).await;
    if leaked == 0 {
        println!("self-filter check passed: no injected events reached capture queue");
    } else {
        eprintln!("self-filter check failed: captured {leaked} injected-looking events");
    }
}

#[cfg(target_os = "windows")]
async fn drain_events(rx: &mut tokio::sync::mpsc::Receiver<LocalMouseEvent>) -> usize {
    let mut drained = 0;
    loop {
        match rx.try_recv() {
            Ok(_) => drained += 1,
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => return drained,
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => return drained,
        }
    }
}

#[cfg(target_os = "windows")]
fn current_cursor_position() -> (f64, f64) {
    use windows::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};

    let mut point = POINT::default();
    if unsafe { GetCursorPos(&mut point) }.is_ok() {
        (point.x as f64, point.y as f64)
    } else {
        (0.0, 0.0)
    }
}

#[cfg(target_os = "windows")]
fn format_event_line(index: usize, event: &LocalMouseEvent) -> String {
    match event {
        LocalMouseEvent::Move {
            x,
            y,
            dx,
            dy,
            ts_ms,
        } => {
            format!("{index:04} move  x={x:.1} y={y:.1} dx={dx:.1} dy={dy:.1} ts={ts_ms}")
        }
        LocalMouseEvent::Down {
            button,
            x,
            y,
            ts_ms,
        } => {
            format!("{index:04} down  button={button:?} x={x:.1} y={y:.1} ts={ts_ms}")
        }
        LocalMouseEvent::Up {
            button,
            x,
            y,
            ts_ms,
        } => {
            format!("{index:04} up    button={button:?} x={x:.1} y={y:.1} ts={ts_ms}")
        }
        LocalMouseEvent::Wheel { dx, dy, ts_ms } => {
            format!("{index:04} wheel dx={dx:.2} dy={dy:.2} ts={ts_ms}")
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("win_input_spike only runs on Windows");
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn formats_mouse_events_for_spike_output() {
        let lines = [
            format_event_line(
                1,
                &LocalMouseEvent::Move {
                    x: 10.0,
                    y: 20.0,
                    dx: 1.0,
                    dy: -1.0,
                    ts_ms: 100,
                },
            ),
            format_event_line(
                2,
                &LocalMouseEvent::Down {
                    button: MouseButton::Left,
                    x: 10.0,
                    y: 20.0,
                    ts_ms: 101,
                },
            ),
            format_event_line(
                3,
                &LocalMouseEvent::Up {
                    button: MouseButton::Left,
                    x: 10.0,
                    y: 20.0,
                    ts_ms: 102,
                },
            ),
            format_event_line(
                4,
                &LocalMouseEvent::Wheel {
                    dx: 0.0,
                    dy: 1.0,
                    ts_ms: 103,
                },
            ),
        ];

        assert!(lines[0].contains("0001 move"));
        assert!(lines[1].contains("0002 down"));
        assert!(lines[1].contains("button=Left"));
        assert!(lines[2].contains("0003 up"));
        assert!(lines[3].contains("0004 wheel"));
        assert!(lines[3].contains("dy=1.00"));
    }

    #[test]
    fn builds_visible_move_injection_plan() {
        let plan = move_injection_plan();

        assert_eq!(plan.len(), 100);
        assert!(plan
            .iter()
            .all(|event| matches!(event, RemoteMouseEvent::Move { dx: 4.0, dy: 0.0 })));
    }

    #[test]
    fn builds_virtual_corner_points_from_topology() {
        let topology = ScreenTopology {
            virtual_bounds: flowlink_lib::input::types::Rect {
                x: -1920.0,
                y: 0.0,
                width: 3840.0,
                height: 1080.0,
            },
            displays: Vec::new(),
        };

        assert_eq!(
            virtual_corner_points(&topology),
            vec![
                (-1920.0, 0.0),
                (1919.0, 0.0),
                (1919.0, 1079.0),
                (-1920.0, 1079.0)
            ]
        );
    }

    #[test]
    fn builds_left_right_middle_click_injection_plan() {
        let plan = click_injection_plan(10.0, 20.0);

        assert_eq!(plan.len(), 6);
        assert!(matches!(
            plan[0],
            RemoteMouseEvent::Down {
                button: MouseButton::Left,
                x: 10.0,
                y: 20.0
            }
        ));
        assert!(matches!(
            plan[1],
            RemoteMouseEvent::Up {
                button: MouseButton::Left,
                x: 10.0,
                y: 20.0
            }
        ));
        assert!(matches!(
            plan[2],
            RemoteMouseEvent::Down {
                button: MouseButton::Right,
                x: 10.0,
                y: 20.0
            }
        ));
        assert!(matches!(
            plan[3],
            RemoteMouseEvent::Up {
                button: MouseButton::Right,
                x: 10.0,
                y: 20.0
            }
        ));
        assert!(matches!(
            plan[4],
            RemoteMouseEvent::Down {
                button: MouseButton::Middle,
                x: 10.0,
                y: 20.0
            }
        ));
        assert!(matches!(
            plan[5],
            RemoteMouseEvent::Up {
                button: MouseButton::Middle,
                x: 10.0,
                y: 20.0
            }
        ));
    }

    #[test]
    fn builds_vertical_and_horizontal_wheel_injection_plan() {
        let plan = wheel_injection_plan();

        assert_eq!(plan.len(), 4);
        assert!(matches!(
            plan[0],
            RemoteMouseEvent::Wheel { dx: 0.0, dy: 1.0 }
        ));
        assert!(matches!(
            plan[1],
            RemoteMouseEvent::Wheel { dx: 0.0, dy: -1.0 }
        ));
        assert!(matches!(
            plan[2],
            RemoteMouseEvent::Wheel { dx: 1.0, dy: 0.0 }
        ));
        assert!(matches!(
            plan[3],
            RemoteMouseEvent::Wheel { dx: -1.0, dy: 0.0 }
        ));
        assert!(wheel_event_label(&plan[0]).contains("up"));
        assert!(wheel_event_label(&plan[1]).contains("down"));
        assert!(wheel_event_label(&plan[2]).contains("right"));
        assert!(wheel_event_label(&plan[3]).contains("left"));
    }
}
