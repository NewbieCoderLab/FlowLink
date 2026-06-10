use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticsSnapshot {
    pub discovered_peer_count: usize,
    pub trusted_peer_count: usize,
    pub layout_count: usize,
    pub config_path: String,
}

