#[cfg(target_os = "macos")]
use std::time::{Duration, Instant};

#[cfg(target_os = "macos")]
use flowlink::{
    input::{
        macos::MacInputPlatform,
        types::{PermissionKind, RemoteMouseEvent},
        InputPlatform,
    },
    platform::PermissionState,
};

#[cfg(target_os = "macos")]
#[tokio::main]
async fn main() {
    let platform = MacInputPlatform::new();
    let permissions = platform.permissions();
    println!(
        "permissions: accessibility={:?}, input_monitoring={:?}",
        permissions.accessibility, permissions.input_monitoring
    );

    if permissions.accessibility != PermissionState::Granted {
        let _ = platform.request_permissions(PermissionKind::Accessibility);
    }
    if permissions.input_monitoring != PermissionState::Granted {
        let _ = platform.request_permissions(PermissionKind::InputMonitoring);
    }

    let (tx, mut rx) = tokio::sync::mpsc::channel(256);
    let _capture = platform
        .start_capture(tx)
        .expect("failed to start macOS event tap; grant Input Monitoring and retry");

    let started = Instant::now();
    let mut count = 0usize;
    while count < 1000 {
        if let Some(event) = rx.recv().await {
            count += 1;
            if count <= 20 || count.is_multiple_of(100) {
                println!("{count:04}: {event:?}");
            }
        }
    }
    println!(
        "captured {count} events in {:.2?}",
        Instant::now().duration_since(started)
    );

    let mut latencies = Vec::with_capacity(100);
    for _ in 0..100 {
        let before = Instant::now();
        platform
            .inject(RemoteMouseEvent::Move { dx: 1.0, dy: 0.0 })
            .expect("failed to inject mouse move; grant Accessibility and retry");
        latencies.push(before.elapsed());
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    latencies.sort();
    let median = latencies[latencies.len() / 2];
    println!("injected 100 moves; median call latency: {median:?}");
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("mac_input_spike only runs on macOS");
}
