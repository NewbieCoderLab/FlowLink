pub mod defaults;
pub mod validate;

use serde::{Deserialize, Serialize};

use crate::protocol::messages::{ArchType, LayoutDirection, OsType, TimestampMs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub schema_version: u16,
    pub local_device_name_override: Option<String>,
    pub network: NetworkConfig,
    pub discovery: DiscoveryConfig,
    pub layouts: Vec<LayoutConfig>,
    pub ui: UiConfig,
    pub updated_at_ms: TimestampMs,
}

impl Default for AppConfig {
    fn default() -> Self {
        defaults::default_app_config()
    }
}

impl AppConfig {
    pub fn upsert_layout(&mut self, layout: LayoutConfig) {
        if let Some(existing) = self
            .layouts
            .iter_mut()
            .find(|item| item.peer_id == layout.peer_id)
        {
            *existing = layout;
        } else {
            self.layouts.push(layout);
        }
        self.updated_at_ms = crate::storage::files::now_ms();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_port: u16,
    pub connect_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub heartbeat_timeout_ms: u64,
    pub reconnect_min_delay_ms: u64,
    pub reconnect_max_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub mdns_enabled: bool,
    pub udp_broadcast_enabled: bool,
    pub udp_port: u16,
    pub announce_interval_ms: u64,
    pub stale_after_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub start_minimized: bool,
    pub show_diagnostics: bool,
    pub last_selected_peer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub peer_id: String,
    pub direction: LayoutDirection,
    pub edge_thickness_px: u32,
    pub corner_guard_px: u32,
    pub enabled: bool,
    pub updated_at_ms: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedPeer {
    pub peer_id: String,
    pub device_name: String,
    pub os: OsType,
    pub arch: ArchType,
    pub public_key: Vec<u8>,
    pub last_known_addresses: Vec<String>,
    pub last_seen_ms: Option<TimestampMs>,
    pub paired_at_ms: TimestampMs,
    pub app_version_at_pairing: String,
    pub protocol_version: u16,
    pub trust_state: TrustState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustState {
    Trusted,
    Blocked,
    Removed,
}
