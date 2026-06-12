pub mod cache;
pub mod mdns;
pub mod udp;

use std::thread;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::warn;

use crate::{
    config::{DiscoveryConfig, NetworkConfig},
    identity::DeviceIdentity,
    protocol::messages::{ArchType, DiscoverySource, OsType, TimestampMs},
};

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

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("mDNS error: {0}")]
    Mdns(String),
    #[error("invalid discovery record: {0}")]
    InvalidRecord(String),
    #[error("discovery channel is closed")]
    ChannelClosed,
}

pub struct DiscoveryRuntime {
    _handles: Vec<thread::JoinHandle<()>>,
    _mdns_daemon: Option<mdns_sd::ServiceDaemon>,
}

impl DiscoveryRuntime {
    pub fn handles_len(&self) -> usize {
        self._handles.len()
    }
}

pub fn start_discovery_tasks(
    identity: DeviceIdentity,
    network: NetworkConfig,
    config: DiscoveryConfig,
    tx: mpsc::Sender<DiscoveredPeer>,
) -> Result<DiscoveryRuntime, DiscoveryError> {
    let mut handles = Vec::new();
    let mut mdns_daemon = None;

    if config.mdns_enabled {
        match start_mdns_tasks(&identity, network.listen_port, tx.clone()) {
            Ok((daemon, handle)) => {
                handles.push(handle);
                mdns_daemon = Some(daemon);
            }
            Err(err) => warn!("mDNS discovery failed to start; UDP fallback may still work: {err}"),
        }
    }

    if config.udp_broadcast_enabled {
        match udp::start_udp_announcer(
            identity.clone(),
            network.listen_port,
            config.udp_port,
            config.announce_interval_ms,
        ) {
            Ok(handle) => handles.push(handle),
            Err(err) => warn!("UDP discovery announcer failed to start: {err}"),
        }

        match udp::start_udp_listener(config.udp_port, identity.device_id.clone(), tx) {
            Ok(handle) => handles.push(handle),
            Err(err) => warn!("UDP discovery listener failed to start: {err}"),
        }
    }

    if handles.is_empty() {
        return Err(DiscoveryError::Mdns(
            "no discovery tasks could be started".into(),
        ));
    }

    Ok(DiscoveryRuntime {
        _handles: handles,
        _mdns_daemon: mdns_daemon,
    })
}

fn start_mdns_tasks(
    identity: &DeviceIdentity,
    service_port: u16,
    tx: mpsc::Sender<DiscoveredPeer>,
) -> Result<(mdns_sd::ServiceDaemon, thread::JoinHandle<()>), DiscoveryError> {
    let daemon =
        mdns_sd::ServiceDaemon::new().map_err(|err| DiscoveryError::Mdns(err.to_string()))?;
    let daemon_for_browser = daemon.clone();
    let local_device_id = identity.device_id.clone();
    mdns::register_mdns_service(&daemon, identity, service_port)?;
    let handle = mdns::start_mdns_browser(&daemon_for_browser, local_device_id, tx)?;
    Ok((daemon, handle))
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
