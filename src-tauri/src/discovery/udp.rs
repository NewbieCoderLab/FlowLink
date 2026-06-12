use std::{
    collections::BTreeSet,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    thread,
    time::Duration,
};

use if_addrs::IfAddr;
use serde::{Deserialize, Serialize};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::{
    discovery::{DiscoveredPeer, DiscoveryError},
    identity::DeviceIdentity,
    protocol::messages::{ArchType, DiscoverySource, OsType},
    storage::files::now_ms,
};

pub const UDP_ANNOUNCE_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UdpAnnounce {
    pub v: u16,
    pub id: String,
    pub name: String,
    pub os: String,
    pub arch: String,
    pub app_version: String,
    pub protocol_version: u16,
    pub port: u16,
    pub pairing: bool,
}

pub fn start_udp_fallback() {
    info!("UDP discovery fallback placeholder initialized");
}

pub fn start_udp_announcer(
    identity: DeviceIdentity,
    service_port: u16,
    udp_port: u16,
    interval_ms: u64,
) -> Result<thread::JoinHandle<()>, DiscoveryError> {
    let socket = broadcast_socket(0)?;
    let destinations = broadcast_destinations(udp_port);
    let interval = Duration::from_millis(interval_ms.max(100));

    thread::Builder::new()
        .name("flowlink-udp-announcer".into())
        .spawn(move || loop {
            for destination in &destinations {
                if let Err(err) = announce_to_peer(&socket, &identity, service_port, *destination) {
                    warn!("UDP announce to {destination} failed: {err}");
                }
            }
            thread::sleep(interval);
        })
        .map_err(|err| DiscoveryError::Mdns(err.to_string()))
}

pub fn start_udp_listener(
    udp_port: u16,
    local_device_id: String,
    tx: mpsc::Sender<DiscoveredPeer>,
) -> Result<thread::JoinHandle<()>, DiscoveryError> {
    let socket = broadcast_socket(udp_port)?;

    thread::Builder::new()
        .name("flowlink-udp-listener".into())
        .spawn(move || {
            let mut buffer = [0u8; 2048];
            loop {
                match socket.recv_from(&mut buffer) {
                    Ok((len, source)) => match decode_announce(&buffer[..len]) {
                        Ok(announce) if should_ignore_announce(&announce, &local_device_id) => {}
                        Ok(announce) => match peer_from_announce(&announce, source) {
                            Ok(peer) => {
                                if tx.blocking_send(peer).is_err() {
                                    break;
                                }
                            }
                            Err(err) => warn!("ignoring invalid UDP announce: {err}"),
                        },
                        Err(err) => warn!("ignoring malformed UDP announce: {err}"),
                    },
                    Err(err) => warn!("UDP discovery recv failed: {err}"),
                }
            }
        })
        .map_err(|err| DiscoveryError::Mdns(err.to_string()))
}

pub fn announce_to_peer(
    socket: &UdpSocket,
    identity: &DeviceIdentity,
    service_port: u16,
    destination: SocketAddr,
) -> Result<usize, DiscoveryError> {
    let announce = build_announce(identity, service_port);
    let payload = serde_json::to_vec(&announce)
        .map_err(|err| DiscoveryError::InvalidRecord(err.to_string()))?;
    socket
        .send_to(&payload, destination)
        .map_err(|err| DiscoveryError::Mdns(err.to_string()))
}

pub fn build_announce(identity: &DeviceIdentity, service_port: u16) -> UdpAnnounce {
    UdpAnnounce {
        v: UDP_ANNOUNCE_VERSION,
        id: identity.device_id.clone(),
        name: identity.device_name.clone(),
        os: os_label(identity.os).into(),
        arch: arch_label(identity.arch).into(),
        app_version: identity.app_version.clone(),
        protocol_version: identity.protocol_version,
        port: service_port,
        pairing: true,
    }
}

pub fn decode_announce(payload: &[u8]) -> Result<UdpAnnounce, DiscoveryError> {
    let announce = serde_json::from_slice::<UdpAnnounce>(payload)
        .map_err(|err| DiscoveryError::InvalidRecord(err.to_string()))?;
    if announce.v != UDP_ANNOUNCE_VERSION {
        return Err(DiscoveryError::InvalidRecord(format!(
            "unsupported UDP announce version {}",
            announce.v
        )));
    }
    Ok(announce)
}

pub fn peer_from_announce(
    announce: &UdpAnnounce,
    source: SocketAddr,
) -> Result<DiscoveredPeer, DiscoveryError> {
    if source.ip().is_unspecified() {
        return Err(DiscoveryError::InvalidRecord(
            "announce source address is unspecified".into(),
        ));
    }

    Ok(DiscoveredPeer {
        device_id: announce.id.clone(),
        device_name: announce.name.clone(),
        os: parse_os(&announce.os),
        arch: parse_arch(&announce.arch),
        app_version: announce.app_version.clone(),
        protocol_version: announce.protocol_version,
        addresses: vec![format!("{}:{}", source.ip(), announce.port)],
        service_port: announce.port,
        pairing_available: announce.pairing,
        last_seen_ms: now_ms(),
        source: DiscoverySource::UdpBroadcast,
    })
}

pub fn should_ignore_announce(announce: &UdpAnnounce, local_device_id: &str) -> bool {
    announce.id == local_device_id
}

pub fn broadcast_destinations(port: u16) -> Vec<SocketAddr> {
    let mut destinations = BTreeSet::from([SocketAddr::from((Ipv4Addr::BROADCAST, port))]);

    if let Ok(interfaces) = if_addrs::get_if_addrs() {
        for interface in interfaces {
            if interface.is_loopback() {
                continue;
            }
            if let IfAddr::V4(addr) = interface.addr {
                if let Some(broadcast) = addr.broadcast {
                    destinations.insert(SocketAddr::from((broadcast, port)));
                }
            }
        }
    }

    destinations.into_iter().collect()
}

fn broadcast_socket(port: u16) -> Result<UdpSocket, DiscoveryError> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .map_err(|err| DiscoveryError::Mdns(err.to_string()))?;
    socket
        .set_reuse_address(true)
        .map_err(|err| DiscoveryError::Mdns(err.to_string()))?;
    socket
        .set_broadcast(true)
        .map_err(|err| DiscoveryError::Mdns(err.to_string()))?;
    socket
        .bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)).into())
        .map_err(|err| DiscoveryError::Mdns(err.to_string()))?;
    Ok(socket.into())
}

fn os_label(os: OsType) -> &'static str {
    match os {
        OsType::Macos => "macos",
        OsType::Windows => "windows",
        OsType::Unknown => "unknown",
    }
}

fn parse_os(value: &str) -> OsType {
    match value {
        "macos" => OsType::Macos,
        "windows" => OsType::Windows,
        _ => OsType::Unknown,
    }
}

fn arch_label(arch: ArchType) -> &'static str {
    match arch {
        ArchType::X86_64 => "x86_64",
        ArchType::Aarch64 => "aarch64",
        ArchType::Unknown => "unknown",
    }
}

fn parse_arch(value: &str) -> ArchType {
    match value {
        "x86_64" => ArchType::X86_64,
        "aarch64" => ArchType::Aarch64,
        _ => ArchType::Unknown,
    }
}
