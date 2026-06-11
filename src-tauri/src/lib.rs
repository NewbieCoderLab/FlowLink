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
use tauri::Manager;

pub fn run() {
    let builder = tauri::Builder::default()
        .setup(|app| {
            let log_dir = app.path().app_log_dir().ok();
            telemetry::logging::init_logging(log_dir);
            let config_dir = app.path().app_config_dir().map_err(|err| err.to_string())?;
            let context = AppContext::load_or_default(config_dir).map_err(|err| err.to_string())?;
            app.manage(SharedAppState::new(context));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ui_api::commands::get_app_status,
            ui_api::commands::list_discovered_devices,
            ui_api::commands::save_layout,
            ui_api::commands::disconnect,
            ui_api::commands::start_pairing,
            ui_api::commands::confirm_pairing,
            ui_api::commands::connect_peer,
            ui_api::commands::open_permission_settings
        ]);

    builder
        .run(tauri::generate_context!())
        .expect("failed to run FlowLink application");
}
