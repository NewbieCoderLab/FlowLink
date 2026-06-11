pub mod cache;
pub mod mdns;
pub mod udp;

use serde::{Deserialize, Serialize};

use crate::protocol::messages::{ArchType, DiscoverySource, OsType, TimestampMs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub device_id: String,
    pub device_name: String,
    pub os: OsType,
    pub arch: ArchType,
    pub app_version: String,
    pub protocol_version: u16,
    pub addresses: Vec<String>,
    pub service_port: u16,
    pub pairing_available: bool,
    pub last_seen_ms: TimestampMs,
    pub source: DiscoverySource,
}

impl DiscoveredPeer {
    pub fn demo_peer() -> Self {
        Self {
            device_id: "peer-demo-device".into(),
            device_name: "Office Windows PC".into(),
            os: OsType::Windows,
            arch: ArchType::X86_64,
            app_version: "0.1.0".into(),
            protocol_version: 1,
            addresses: vec!["192.168.1.42:42424".into()],
            service_port: 42424,
            pairing_available: true,
            last_seen_ms: crate::storage::files::now_ms(),
            source: DiscoverySource::Mdns,
        }
    }
}
