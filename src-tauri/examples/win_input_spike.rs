#[cfg(target_os = "windows")]
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use flowlink::{
    input::{
        types::{PermissionKind, RemoteMouseEvent},
        windows::WinInputPlatform,
        InputPlatform,
    },
    platform::PermissionState,
};

#[cfg(target_os = "windows")]
#[tokio::main]
async fn main() {
    let platform = WinInputPlatform::new();
    let permissions = platform.permissions();
    println!("permissions: windows_input={:?}", permissions.windows_input);

    if permissions.windows_input != PermissionState::Granted {
        let _ = platform.request_permissions(PermissionKind::WindowsInput);
    }

    println!("screen topology: {:#?}", platform.screen_topology());

    let (tx, mut rx) = tokio::sync::mpsc::channel(256);
    let _capture = platform
        .start_capture(tx)
        .expect("failed to install WH_MOUSE_LL hook");

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
            .expect("SendInput failed");
        latencies.push(before.elapsed());
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    latencies.sort();
    let median = latencies[latencies.len() / 2];
    println!("injected 100 moves; median call latency: {median:?}");
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("win_input_spike only runs on Windows");
}
