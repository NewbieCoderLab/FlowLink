use std::time::{Duration, Instant};

use flowlink_lib::input::{
    platform_input,
    types::{LocalMouseEvent, Point, RemoteMouseEvent, ScreenTopology},
    InputPlatform,
};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let platform = platform_input();
    let permissions = platform.permissions();

    println!(
        "permissions: capture={}, inject={}, accessibility={:?}, input_monitoring={:?}, windows_input={:?}, windows_integrity={:?}",
        permissions.can_capture_mouse,
        permissions.can_inject_mouse,
        permissions.accessibility,
        permissions.input_monitoring,
        permissions.windows_input,
        permissions.windows_integrity_level
    );

    match platform.screen_topology() {
        Ok(topology) => {
            print_screen_topology(&topology);
            if args.warp_corners {
                run_warp_corners_smoke(platform.as_ref(), &topology).await;
            }
        }
        Err(err) => eprintln!("screen topology failed: {err}"),
    }

    if args.inject_move {
        run_inject_move_smoke(platform.as_ref()).await;
    }

    if args.capture {
        run_capture_smoke(platform.as_ref(), args.capture_events).await;
    }

    if !args.capture && !args.inject_move && !args.warp_corners {
        println!("run with --capture, --inject-move, or --warp-corners for interactive checks.");
    }
}

#[derive(Debug)]
struct Args {
    capture: bool,
    capture_events: usize,
    inject_move: bool,
    warp_corners: bool,
}

impl Args {
    fn parse() -> Self {
        let args = std::env::args().skip(1).collect::<Vec<_>>();
        let capture = args.iter().any(|arg| arg == "--capture");
        let inject_move = args.iter().any(|arg| arg == "--inject-move");
        let warp_corners = args.iter().any(|arg| arg == "--warp-corners");
        let capture_events = args
            .windows(2)
            .find_map(|pair| {
                (pair[0] == "--events")
                    .then(|| pair[1].parse::<usize>().ok())
                    .flatten()
            })
            .unwrap_or(20);

        Self {
            capture,
            capture_events,
            inject_move,
            warp_corners,
        }
    }
}

fn print_screen_topology(topology: &ScreenTopology) {
    println!(
        "screen topology: {} display(s), virtual bounds {:?}",
        topology.displays.len(),
        topology.virtual_bounds
    );
    for display in &topology.displays {
        println!(
            "  display id={} primary={} scale={} bounds={:?}",
            display.id, display.is_primary, display.scale_factor, display.bounds
        );
    }
}

async fn run_capture_smoke(platform: &dyn InputPlatform, target_events: usize) {
    let (tx, mut rx) = tokio::sync::mpsc::channel(256);
    let capture = match platform.start_capture(tx) {
        Ok(capture) => capture,
        Err(err) => {
            eprintln!("capture failed: {err}");
            return;
        }
    };

    println!("capturing {target_events} local mouse events...");
    let started = Instant::now();
    let mut stats = EventStats::default();
    while stats.total() < target_events {
        match tokio::time::timeout(Duration::from_secs(15), rx.recv()).await {
            Ok(Some(event)) => {
                stats.record(&event);
                println!("{:04}: {event:?}", stats.total());
            }
            Ok(None) => break,
            Err(_) => {
                eprintln!("capture timed out after 15s");
                break;
            }
        }
    }
    drop(capture);

    println!(
        "captured {} event(s) in {:.2?}",
        stats.total(),
        started.elapsed()
    );
    println!(
        "move={}, down={}, up={}, wheel={}",
        stats.moves, stats.downs, stats.ups, stats.wheels
    );
}

async fn run_inject_move_smoke(platform: &dyn InputPlatform) {
    println!("injecting 20 small relative moves...");
    for _ in 0..20 {
        if let Err(err) = platform.inject(RemoteMouseEvent::Move { dx: 2.0, dy: 0.0 }) {
            eprintln!("inject move failed: {err}");
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn run_warp_corners_smoke(platform: &dyn InputPlatform, topology: &ScreenTopology) {
    let points = virtual_corner_points(topology);
    println!(
        "warping cursor through {} virtual desktop corner(s)...",
        points.len()
    );
    for point in points {
        println!("warp: x={:.1}, y={:.1}", point.x, point.y);
        if let Err(err) = platform.warp_cursor(point) {
            eprintln!("warp failed: {err}");
            return;
        }
        tokio::time::sleep(Duration::from_millis(700)).await;
    }
}

fn virtual_corner_points(topology: &ScreenTopology) -> Vec<Point> {
    let bounds = topology.virtual_bounds;
    if bounds.width <= 1.0 || bounds.height <= 1.0 {
        return Vec::new();
    }

    let left = bounds.x;
    let top = bounds.y;
    let right = bounds.x + bounds.width - 1.0;
    let bottom = bounds.y + bounds.height - 1.0;
    vec![
        Point { x: left, y: top },
        Point { x: right, y: top },
        Point {
            x: right,
            y: bottom,
        },
        Point { x: left, y: bottom },
    ]
}

#[derive(Debug, Default)]
struct EventStats {
    moves: usize,
    downs: usize,
    ups: usize,
    wheels: usize,
}

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
