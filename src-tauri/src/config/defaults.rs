use crate::{
    config::{AppConfig, DiscoveryConfig, NetworkConfig, UiConfig},
    storage::files::now_ms,
};

pub fn default_app_config() -> AppConfig {
    AppConfig {
        schema_version: 1,
        local_device_name_override: None,
        network: NetworkConfig {
            listen_port: 42424,
            connect_timeout_ms: 3000,
            heartbeat_interval_ms: 1000,
            heartbeat_timeout_ms: 3000,
            reconnect_min_delay_ms: 500,
            reconnect_max_delay_ms: 10_000,
        },
        discovery: DiscoveryConfig {
            mdns_enabled: true,
            udp_broadcast_enabled: true,
            udp_port: 42425,
            announce_interval_ms: 1500,
            stale_after_ms: 10_000,
        },
        layouts: Vec::new(),
        ui: UiConfig {
            start_minimized: false,
            show_diagnostics: true,
            last_selected_peer_id: None,
        },
        updated_at_ms: now_ms(),
    }
}

