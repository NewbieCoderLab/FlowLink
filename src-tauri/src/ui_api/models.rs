use serde::{Deserialize, Serialize};

use crate::{
    config::LayoutConfig,
    discovery::DiscoveredPeer,
    identity::DeviceIdentity,
    input::types::{DisplayInfo, InputError, Rect, ScreenTopology},
    platform::PermissionStatus,
    protocol::messages::{ArchType, LayoutDirection, OsType},
    session::state::{ControlOwner, SessionSnapshot, SessionState},
    telemetry::metrics::DiagnosticsSnapshot,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiDevice {
    pub device_id: String,
    pub name: String,
    pub os: OsType,
    pub arch: ArchType,
    pub app_version: String,
    pub protocol_version: u16,
    pub address_label: String,
    pub status: String,
    pub last_seen_label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiPermissionStatus {
    pub accessibility: String,
    pub input_monitoring: String,
    pub screen_recording: String,
    pub windows_input: String,
    pub windows_integrity_level: Option<String>,
    pub can_capture_mouse: bool,
    pub can_inject_mouse: bool,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiSessionStatus {
    pub state: String,
    pub control_owner: String,
    pub peer_name: Option<String>,
    pub last_heartbeat_rtt_ms: Option<u32>,
    pub connected_since_ms: Option<u64>,
    pub updated_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiLayoutConfig {
    pub peer_id: String,
    pub direction: LayoutDirection,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiDiagnostics {
    pub discovered_peer_count: usize,
    pub trusted_peer_count: usize,
    pub layout_count: usize,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiDisplayInfo {
    pub id: u64,
    pub bounds: UiRect,
    pub scale_factor: f64,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiScreenTopology {
    pub displays: Vec<UiDisplayInfo>,
    pub virtual_bounds: UiRect,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiAppStatus {
    pub local_device: UiDevice,
    pub permission: UiPermissionStatus,
    pub session: UiSessionStatus,
    pub discovered_devices: Vec<UiDevice>,
    pub saved_layouts: Vec<UiLayoutConfig>,
    pub diagnostics: UiDiagnostics,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
}

impl From<InputError> for UiError {
    fn from(value: InputError) -> Self {
        let code = match &value {
            InputError::Unsupported => "input_unsupported",
            InputError::PermissionDenied(_) => "input_permission_denied",
            InputError::CaptureAlreadyRunning => "input_capture_already_running",
            InputError::Platform(_) => "input_platform_error",
        };

        Self {
            code: code.into(),
            message: value.to_string(),
            recoverable: true,
        }
    }
}

impl From<&DeviceIdentity> for UiDevice {
    fn from(value: &DeviceIdentity) -> Self {
        Self {
            device_id: value.device_id.clone(),
            name: value.device_name.clone(),
            os: value.os,
            arch: value.arch,
            app_version: value.app_version.clone(),
            protocol_version: value.protocol_version,
            address_label: format!("127.0.0.1:{}", 42424),
            status: "connected".into(),
            last_seen_label: "just now".into(),
        }
    }
}

impl From<&DiscoveredPeer> for UiDevice {
    fn from(value: &DiscoveredPeer) -> Self {
        Self {
            device_id: value.device_id.clone(),
            name: value.device_name.clone(),
            os: value.os,
            arch: value.arch,
            app_version: value.app_version.clone(),
            protocol_version: value.protocol_version,
            address_label: value
                .addresses
                .first()
                .cloned()
                .unwrap_or_else(|| "-".into()),
            status: if value.pairing_available {
                "available".into()
            } else {
                "stale".into()
            },
            last_seen_label: "recently".into(),
        }
    }
}

impl From<&PermissionStatus> for UiPermissionStatus {
    fn from(value: &PermissionStatus) -> Self {
        Self {
            accessibility: value.accessibility.as_str().into(),
            input_monitoring: value.input_monitoring.as_str().into(),
            screen_recording: value.screen_recording.as_str().into(),
            windows_input: value.windows_input.as_str().into(),
            windows_integrity_level: value.windows_integrity_level.clone(),
            can_capture_mouse: value.can_capture_mouse,
            can_inject_mouse: value.can_inject_mouse,
            updated_at_ms: value.updated_at_ms,
        }
    }
}

impl From<&SessionSnapshot> for UiSessionStatus {
    fn from(value: &SessionSnapshot) -> Self {
        Self {
            state: session_state_label(&value.state),
            control_owner: control_owner_label(&value.control_owner),
            peer_name: value.peer_name.clone(),
            last_heartbeat_rtt_ms: value.last_heartbeat_rtt_ms,
            connected_since_ms: value.connected_since_ms,
            updated_at_ms: value.updated_at_ms,
        }
    }
}

impl From<&LayoutConfig> for UiLayoutConfig {
    fn from(value: &LayoutConfig) -> Self {
        Self {
            peer_id: value.peer_id.clone(),
            direction: value.direction,
            enabled: value.enabled,
        }
    }
}

impl From<DiagnosticsSnapshot> for UiDiagnostics {
    fn from(value: DiagnosticsSnapshot) -> Self {
        Self {
            discovered_peer_count: value.discovered_peer_count,
            trusted_peer_count: value.trusted_peer_count,
            layout_count: value.layout_count,
            config_path: value.config_path,
        }
    }
}

impl From<&Rect> for UiRect {
    fn from(value: &Rect) -> Self {
        Self {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

impl From<&DisplayInfo> for UiDisplayInfo {
    fn from(value: &DisplayInfo) -> Self {
        Self {
            id: value.id,
            bounds: UiRect::from(&value.bounds),
            scale_factor: value.scale_factor,
            is_primary: value.is_primary,
        }
    }
}

impl From<&ScreenTopology> for UiScreenTopology {
    fn from(value: &ScreenTopology) -> Self {
        Self {
            displays: value.displays.iter().map(UiDisplayInfo::from).collect(),
            virtual_bounds: UiRect::from(&value.virtual_bounds),
        }
    }
}

fn session_state_label(value: &SessionState) -> String {
    match value {
        SessionState::Disconnected => "disconnected",
        SessionState::Discovered => "discovered",
        SessionState::Pairing => "pairing",
        SessionState::Paired => "paired",
        SessionState::Connecting => "connecting",
        SessionState::ConnectedIdle => "connected_idle",
        SessionState::ControllingRemote => "controlling_remote",
        SessionState::ControlledByRemote => "controlled_by_remote",
        SessionState::Reconnecting => "reconnecting",
        SessionState::Error => "error",
    }
    .into()
}

fn control_owner_label(value: &ControlOwner) -> String {
    match value {
        ControlOwner::Local => "local",
        ControlOwner::Remote => "remote",
        ControlOwner::None => "none",
    }
    .into()
}
