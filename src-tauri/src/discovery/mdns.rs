use std::{collections::HashMap, net::IpAddr, thread};

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::{
    discovery::{DiscoveredPeer, DiscoveryError},
    identity::DeviceIdentity,
    protocol::messages::{ArchType, DiscoverySource, OsType},
    storage::files::now_ms,
};

pub const FLOWLINK_MDNS_SERVICE_TYPE: &str = "_mac22win._tcp.local.";

pub fn start_mdns() {
    info!("mDNS discovery placeholder initialized");
}

pub fn register_mdns_service(
    daemon: &ServiceDaemon,
    identity: &DeviceIdentity,
    service_port: u16,
) -> Result<String, DiscoveryError> {
    let addresses = local_ip_addresses()?;
    let service = build_service_info(identity, service_port, &addresses)?;
    let fullname = service.get_fullname().to_string();
    daemon
        .register(service)
        .map_err(|err| DiscoveryError::Mdns(err.to_string()))?;
    Ok(fullname)
}

pub fn start_mdns_browser(
    daemon: &ServiceDaemon,
    local_device_id: String,
    tx: mpsc::Sender<DiscoveredPeer>,
) -> Result<thread::JoinHandle<()>, DiscoveryError> {
    let receiver = daemon
        .browse(FLOWLINK_MDNS_SERVICE_TYPE)
        .map_err(|err| DiscoveryError::Mdns(err.to_string()))?;

    let handle = thread::Builder::new()
        .name("flowlink-mdns-browser".into())
        .spawn(move || {
            while let Ok(event) = receiver.recv() {
                match event {
                    ServiceEvent::ServiceResolved(service) => {
                        match service_info_to_peer(&service) {
                            Ok(peer) if should_ignore_peer(&peer, &local_device_id) => {
                                debug!("ignoring local mDNS service {}", peer.device_id);
                            }
                            Ok(peer) => {
                                if tx.blocking_send(peer).is_err() {
                                    break;
                                }
                            }
                            Err(err) => warn!("ignoring invalid mDNS service: {err}"),
                        }
                    }
                    ServiceEvent::SearchStopped(_) => break,
                    _ => {}
                }
            }
        })
        .map_err(|err| DiscoveryError::Mdns(err.to_string()))?;

    Ok(handle)
}

pub fn build_service_info(
    identity: &DeviceIdentity,
    service_port: u16,
    addresses: &[IpAddr],
) -> Result<ServiceInfo, DiscoveryError> {
    let properties = txt_properties(identity);
    let host_name = format!("{}.local.", sanitize_dns_label(&identity.device_name));
    ServiceInfo::new(
        FLOWLINK_MDNS_SERVICE_TYPE,
        &identity.device_name,
        &host_name,
        addresses,
        service_port,
        Some(properties),
    )
    .map_err(|err| DiscoveryError::Mdns(err.to_string()))
}

pub fn service_info_to_peer(service: &ServiceInfo) -> Result<DiscoveredPeer, DiscoveryError> {
    if service.get_type() != FLOWLINK_MDNS_SERVICE_TYPE {
        return Err(DiscoveryError::InvalidRecord(format!(
            "unexpected service type {}",
            service.get_type()
        )));
    }

    let service_port = service.get_port();
    let device_id = required_txt(service, "device_id")?.to_string();
    let device_name = required_txt(service, "device_name")?.to_string();
    let os = parse_os(required_txt(service, "os")?);
    let arch = parse_arch(required_txt(service, "arch")?);
    let app_version = required_txt(service, "app_version")?.to_string();
    let protocol_version = required_txt(service, "protocol_version")?
        .parse::<u16>()
        .map_err(|_| DiscoveryError::InvalidRecord("invalid protocol_version".into()))?;
    let pairing_available = parse_bool(required_txt(service, "pairing")?);
    let mut addresses = service
        .get_addresses()
        .iter()
        .map(|address| format!("{address}:{service_port}"))
        .collect::<Vec<_>>();
    addresses.sort();

    if addresses.is_empty() {
        return Err(DiscoveryError::InvalidRecord(
            "resolved service has no addresses".into(),
        ));
    }

    Ok(DiscoveredPeer {
        device_id,
        device_name,
        os,
        arch,
        app_version,
        protocol_version,
        addresses,
        service_port,
        pairing_available,
        last_seen_ms: now_ms(),
        source: DiscoverySource::Mdns,
    })
}

pub fn should_ignore_peer(peer: &DiscoveredPeer, local_device_id: &str) -> bool {
    peer.device_id == local_device_id
}

fn local_ip_addresses() -> Result<Vec<IpAddr>, DiscoveryError> {
    let addresses = if_addrs::get_if_addrs()
        .map_err(|err| DiscoveryError::Mdns(err.to_string()))?
        .into_iter()
        .filter(|interface| !interface.is_loopback())
        .map(|interface| interface.ip())
        .collect::<Vec<_>>();

    if addresses.is_empty() {
        return Err(DiscoveryError::Mdns(
            "no non-loopback interface addresses available".into(),
        ));
    }

    Ok(addresses)
}

fn txt_properties(identity: &DeviceIdentity) -> HashMap<String, String> {
    HashMap::from([
        ("device_id".into(), identity.device_id.clone()),
        ("device_name".into(), identity.device_name.clone()),
        ("os".into(), os_label(identity.os).into()),
        ("arch".into(), arch_label(identity.arch).into()),
        ("app_version".into(), identity.app_version.clone()),
        (
            "protocol_version".into(),
            identity.protocol_version.to_string(),
        ),
        ("pairing".into(), "true".into()),
    ])
}

fn required_txt<'a>(service: &'a ServiceInfo, key: &str) -> Result<&'a str, DiscoveryError> {
    service
        .get_property_val_str(key)
        .ok_or_else(|| DiscoveryError::InvalidRecord(format!("missing TXT field {key}")))
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

fn parse_bool(value: &str) -> bool {
    matches!(value, "true" | "1" | "yes")
}

fn sanitize_dns_label(value: &str) -> String {
    let label = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if label.is_empty() {
        "flowlink".into()
    } else {
        label
    }
}
