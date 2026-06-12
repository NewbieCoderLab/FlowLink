pub mod app;
pub mod config;
pub mod discovery;
pub mod identity;
pub mod input;
pub mod network;
pub mod pairing;
pub mod platform;
pub mod protocol;
pub mod session;
pub mod storage;
pub mod telemetry;
pub mod ui_api;

use app::context::{AppContext, SharedAppState};
use tauri::{Emitter, Manager};

pub fn run() {
    #[cfg(target_os = "windows")]
    input::windows::init_process_dpi_awareness_once();

    let builder = tauri::Builder::default()
        .setup(|app| {
            let log_dir = app.path().app_log_dir().ok();
            telemetry::logging::init_logging(log_dir);
            let config_dir = app.path().app_config_dir().map_err(|err| err.to_string())?;
            let context = AppContext::load_or_default(config_dir).map_err(|err| err.to_string())?;
            let shared_state = SharedAppState::new(context);
            start_discovery_runtime(app.handle().clone(), shared_state.clone());
            app.manage(shared_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ui_api::commands::get_app_status,
            ui_api::commands::list_discovered_devices,
            ui_api::commands::get_screen_topology,
            ui_api::commands::save_layout,
            ui_api::commands::disconnect,
            ui_api::commands::start_pairing,
            ui_api::commands::confirm_pairing,
            ui_api::commands::connect_peer,
            ui_api::commands::probe_peer_ip,
            ui_api::commands::open_permission_settings
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("failed to run FlowLink application");
}

fn start_discovery_runtime(app_handle: tauri::AppHandle, state: SharedAppState) {
    let (identity, network, discovery_config) = match state.0.read() {
        Ok(app) => (
            app.local_identity.clone(),
            app.config.network.clone(),
            app.config.discovery.clone(),
        ),
        Err(_) => return,
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel(256);
    match discovery::start_discovery_tasks(identity, network, discovery_config.clone(), tx) {
        Ok(runtime) => {
            std::mem::forget(runtime);
        }
        Err(err) => {
            tracing::warn!("failed to start discovery tasks: {err}");
            return;
        }
    }

    let receiver_state = state.clone();
    let receiver_app = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(peer) = rx.recv().await {
            let discovered = match receiver_state.0.write() {
                Ok(mut app) => app.handle_discovered_peer(peer),
                Err(_) => None,
            };
            if let Some(peer) = discovered {
                let _ = receiver_app.emit(ui_api::events::DEVICE_DISCOVERED, peer);
            }
        }
    });

    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(
            discovery_config.stale_after_ms.max(1_000),
        ));
        loop {
            interval.tick().await;
            let stale = match state.0.write() {
                Ok(mut app) => app.evict_stale_discovered_peers(crate::storage::files::now_ms()),
                Err(_) => Vec::new(),
            };
            for peer in stale {
                let _ = app_handle.emit(ui_api::events::DEVICE_STALE, peer);
            }
        }
    });
}
