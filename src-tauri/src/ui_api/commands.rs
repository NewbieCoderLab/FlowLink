use tauri::{AppHandle, Emitter, State};

use crate::{
    app::context::SharedAppState,
    config::LayoutConfig,
    input::types::PermissionKind,
    session::controller::emergency_disconnect,
    ui_api::events,
    ui_api::models::{
        UiAppStatus, UiDevice, UiDiagnostics, UiError, UiLayoutConfig, UiPermissionStatus,
        UiSessionStatus,
    },
};

#[tauri::command]
pub async fn get_app_status(state: State<'_, SharedAppState>) -> Result<UiAppStatus, UiError> {
    let app = state.0.read().map_err(lock_error)?;
    let local_device = UiDevice::from(&app.local_identity);
    let discovered_devices = app
        .list_discovered_devices()
        .iter()
        .map(UiDevice::from)
        .collect::<Vec<_>>();
    let saved_layouts = app
        .config
        .layouts
        .iter()
        .map(UiLayoutConfig::from)
        .collect();

    Ok(UiAppStatus {
        local_device,
        permission: UiPermissionStatus::from(&app.permissions),
        session: UiSessionStatus::from(&app.session),
        discovered_devices,
        saved_layouts,
        diagnostics: UiDiagnostics::from(app.diagnostics()),
    })
}

#[tauri::command]
pub async fn list_discovered_devices(
    state: State<'_, SharedAppState>,
) -> Result<Vec<UiDevice>, UiError> {
    let app = state.0.read().map_err(lock_error)?;
    Ok(app
        .list_discovered_devices()
        .iter()
        .map(UiDevice::from)
        .collect())
}

#[tauri::command]
pub async fn save_layout(
    state: State<'_, SharedAppState>,
    layout: UiLayoutConfig,
) -> Result<(), UiError> {
    let mut app = state.0.write().map_err(lock_error)?;
    let layout = LayoutConfig {
        peer_id: layout.peer_id,
        direction: layout.direction,
        edge_thickness_px: 1,
        corner_guard_px: 32,
        enabled: layout.enabled,
        updated_at_ms: crate::storage::files::now_ms(),
    };
    app.save_layout(layout).map_err(app_error)
}

#[tauri::command]
pub async fn disconnect(state: State<'_, SharedAppState>) -> Result<(), UiError> {
    let mut app = state.0.write().map_err(lock_error)?;
    emergency_disconnect(&mut app.session);
    Ok(())
}

#[tauri::command]
pub async fn start_pairing(_device_id: String) -> Result<String, UiError> {
    Ok(crate::pairing::flow::PairingFlow::new().pairing_id)
}

#[tauri::command]
pub async fn confirm_pairing(_pairing_id: String) -> Result<(), UiError> {
    Ok(())
}

#[tauri::command]
pub async fn connect_peer(_peer_id: String) -> Result<(), UiError> {
    Ok(())
}

#[tauri::command]
pub async fn open_permission_settings(
    app_handle: AppHandle,
    state: State<'_, SharedAppState>,
    permission: String,
) -> Result<(), UiError> {
    if let Some(kind) = permission_kind(&permission) {
        let mut app = state.0.write().map_err(lock_error)?;
        let _ = app.input.request_permissions(kind);
        app.refresh_permissions();
    }

    #[cfg(target_os = "macos")]
    crate::platform::macos_permissions::open_settings_pane(&permission).map_err(|message| {
        UiError {
            code: "open_permission_settings_failed".into(),
            message,
            recoverable: true,
        }
    })?;

    let _ = app_handle.emit(events::PERMISSION_UPDATED, ());

    Ok(())
}

fn lock_error<T>(_err: T) -> UiError {
    UiError {
        code: "lock_failed".into(),
        message: "Shared application state is unavailable".into(),
        recoverable: true,
    }
}

fn app_error(err: crate::app::context::AppError) -> UiError {
    UiError {
        code: "app_error".into(),
        message: err.to_string(),
        recoverable: true,
    }
}

fn permission_kind(value: &str) -> Option<PermissionKind> {
    match value {
        "accessibility" => Some(PermissionKind::Accessibility),
        "input_monitoring" => Some(PermissionKind::InputMonitoring),
        "windows_input" => Some(PermissionKind::WindowsInput),
        _ => None,
    }
}
