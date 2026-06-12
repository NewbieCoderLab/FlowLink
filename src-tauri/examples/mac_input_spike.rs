#[cfg(target_os = "macos")]
use std::{
    env,
    time::{Duration, Instant},
};

#[cfg(target_os = "macos")]
use core_graphics::{
    event::CGEvent,
    event_source::{CGEventSource, CGEventSourceStateID},
};
#[cfg(target_os = "macos")]
use flowlink_lib::{
    input::{
        macos::MacInputPlatform,
        types::{LocalMouseEvent, PermissionKind, RemoteMouseEvent, ScreenTopology},
        InputPlatform,
    },
    platform::PermissionState,
};

#[cfg(target_os = "macos")]
const TARGET_EVENTS: usize = 1000;

#[cfg(target_os = "macos")]
#[tokio::main]
async fn main() {
    let inject_move = env::args().any(|arg| arg == "--inject-move");
    let inject_click = env::args().any(|arg| arg == "--inject-click");
    let verify_self_filter = env::args().any(|arg| arg == "--verify-self-filter");
    let print_topology = env::args().any(|arg| arg == "--topology");
    let platform = MacInputPlatform::new();
    let permissions = platform.permissions();
    print_permission_banner(&permissions);

    if print_topology {
        match platform.screen_topology() {
            Ok(topology) => print_screen_topology(&topology),
            Err(err) => eprintln!("failed to read screen topology: {err}"),
        }
    }

    if permissions.input_monitoring != PermissionState::Granted {
        eprintln!("Input Monitoring is required for this spike.");
        eprintln!("Grant it to the terminal app running this command, then rerun the spike.");
        let _ = platform.request_permissions(PermissionKind::InputMonitoring);
        return;
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel(256);
    let _capture = platform
        .start_capture(tx)
        .expect("failed to start macOS event tap; grant Input Monitoring and retry");

    println!("capturing {TARGET_EVENTS} local mouse events...");
    println!("move the mouse, click buttons, and use the wheel to cover all event kinds.");

    let started = Instant::now();
    let mut stats = EventStats::default();
    while stats.total() < TARGET_EVENTS {
        if let Some(event) = rx.recv().await {
            stats.record(&event);
            let total = stats.total();
            if total <= 20 || total.is_multiple_of(100) {
                println!("{total:04}: {event:?}");
            }
        }
    }

    let elapsed = Instant::now().duration_since(started);
    println!("captured {} events in {elapsed:.2?}", stats.total());
    println!("move: {}", stats.moves);
    println!("down: {}", stats.downs);
    println!("up: {}", stats.ups);
    println!("wheel: {}", stats.wheels);

    if inject_move {
        run_move_injection_spike(&platform).await;
    }
    if inject_click {
        run_click_injection_spike(&platform).await;
    }
    if verify_self_filter {
        run_self_filter_spike(&platform, &mut rx).await;
    }
    if !inject_move && !inject_click && !verify_self_filter && !print_topology {
        println!(
            "run again with --inject-move, --inject-click, --verify-self-filter, or --topology."
        );
    }
}

#[cfg(target_os = "macos")]
fn print_permission_banner(permissions: &flowlink_lib::platform::PermissionStatus) {
    println!(
        "permissions: accessibility={:?}, input_monitoring={:?}",
        permissions.accessibility, permissions.input_monitoring
    );
    println!(
        "capture={}, inject={}",
        permissions.can_capture_mouse, permissions.can_inject_mouse
    );
    if permissions.accessibility != PermissionState::Granted {
        println!("Accessibility is not required for P014 capture-only spike.");
        println!("It will be required later for P015/P016 injection tests.");
    }
}

#[cfg(target_os = "macos")]
#[derive(Debug, Default)]
struct EventStats {
    moves: usize,
    downs: usize,
    ups: usize,
    wheels: usize,
}

#[cfg(target_os = "macos")]
impl EventStats {
    fn record(&mut self, event: &LocalMouseEvent) {
        match event {
            LocalMouseEvent::Move { .. } => self.moves += 1,
            LocalMouseEvent::Down { .. } => self.downs += 1,
            LocalMouseEvent::Up { .. } => self.ups += 1,
            LocalMouseEvent::Wheel { .. } => self.wheels += 1,
        }
    }

    fn total(&self) -> usize {
        self.moves + self.downs + self.ups + self.wheels
    }
}

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
async fn run_move_injection_spike(platform: &MacInputPlatform) {
    if !ensure_accessibility(platform, "--inject-move") {
        return;
    }

    println!("injecting 100 small mouse moves via CGEventPost...");
    let mut latencies = Vec::with_capacity(100);
    for step in 0..100 {
        let before = Instant::now();
        platform
            .inject(RemoteMouseEvent::Move { dx: 1.0, dy: 0.0 })
            .expect("failed to inject mouse move; grant Accessibility and retry");
        latencies.push(before.elapsed());
        if (step + 1) % 25 == 0 {
            println!("injected {} moves", step + 1);
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

#[cfg(target_os = "macos")]
async fn run_self_filter_spike(
    platform: &MacInputPlatform,
    rx: &mut tokio::sync::mpsc::Receiver<LocalMouseEvent>,
) {
    if !ensure_accessibility(platform, "--verify-self-filter") {
        return;
    }

    println!("draining queued local events before self-filter check...");
    drain_events(rx).await;
    println!("injecting 20 tagged moves; event tap should filter them out...");
    for _ in 0..20 {
        platform
            .inject(RemoteMouseEvent::Move { dx: 1.0, dy: 0.0 })
            .expect("failed to inject tagged mouse move");
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    tokio::time::sleep(Duration::from_millis(200)).await;
    let leaked = drain_events(rx).await;
    if leaked == 0 {
        println!("self-filter check passed: no injected events were captured");
    } else {
        eprintln!("self-filter check failed: captured {leaked} injected-looking events");
    }
}

#[cfg(target_os = "macos")]
async fn run_click_injection_spike(platform: &MacInputPlatform) {
    if !ensure_accessibility(platform, "--inject-click") {
        return;
    }

    println!("move the cursor over a safe target; injecting left click in 2 seconds...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    let (x, y) = current_cursor_position();
    let before = Instant::now();
    platform
        .inject(RemoteMouseEvent::Down {
            button: flowlink_lib::protocol::messages::MouseButton::Left,
            x,
            y,
        })
        .expect("failed to inject mouse down; grant Accessibility and retry");
    tokio::time::sleep(Duration::from_millis(50)).await;
    platform
        .inject(RemoteMouseEvent::Up {
            button: flowlink_lib::protocol::messages::MouseButton::Left,
            x,
            y,
        })
        .expect("failed to inject mouse up; grant Accessibility and retry");

    println!("injected one left click in {:?}", before.elapsed());
}

#[cfg(target_os = "macos")]
fn ensure_accessibility(platform: &MacInputPlatform, flag: &str) -> bool {
    let permissions = platform.permissions();
    if permissions.accessibility == PermissionState::Granted {
        return true;
    }

    eprintln!("Accessibility is required for {flag}.");
    eprintln!("Grant it to the terminal app running this command, then rerun the spike.");
    let _ = platform.request_permissions(PermissionKind::Accessibility);
    false
}

#[cfg(target_os = "macos")]
fn current_cursor_position() -> (f64, f64) {
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .expect("failed to create CGEventSource");
    let location = CGEvent::new(source)
        .expect("failed to create CGEvent")
        .location();
    (location.x, location.y)
}

#[cfg(target_os = "macos")]
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

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("mac_input_spike only runs on macOS");
}
