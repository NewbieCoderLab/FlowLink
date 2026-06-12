use std::net::{IpAddr, Ipv4Addr};

use flowlink_lib::{
    discovery::mdns::{
        build_service_info, service_info_to_peer, should_ignore_peer, FLOWLINK_MDNS_SERVICE_TYPE,
    },
    identity::{DeviceIdentity, PrivateKeyRef},
    protocol::messages::{ArchType, DiscoverySource, OsType},
};

#[test]
fn mdns_service_info_contains_flowlink_txt_fields() {
    let identity = test_identity("device-a");
    let service = build_service_info(
        &identity,
        42424,
        &[IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20))],
    )
    .expect("service info");

    assert_eq!(service.get_type(), FLOWLINK_MDNS_SERVICE_TYPE);
    assert_eq!(service.get_port(), 42424);
    assert_eq!(service.get_property_val_str("device_id"), Some("device-a"));
    assert_eq!(
        service.get_property_val_str("device_name"),
        Some("FlowLink Test")
    );
    assert_eq!(service.get_property_val_str("os"), Some("windows"));
    assert_eq!(service.get_property_val_str("arch"), Some("x86_64"));
    assert_eq!(service.get_property_val_str("app_version"), Some("0.1.0"));
    assert_eq!(service.get_property_val_str("protocol_version"), Some("1"));
    assert_eq!(service.get_property_val_str("pairing"), Some("true"));
}

#[test]
fn resolved_mdns_service_becomes_discovered_peer() {
    let identity = test_identity("peer-a");
    let service = build_service_info(
        &identity,
        42424,
        &[IpAddr::V4(Ipv4Addr::new(192, 168, 1, 21))],
    )
    .expect("service info");

    let peer = service_info_to_peer(&service).expect("peer");

    assert_eq!(peer.device_id, "peer-a");
    assert_eq!(peer.device_name, "FlowLink Test");
    assert_eq!(peer.os, OsType::Windows);
    assert_eq!(peer.arch, ArchType::X86_64);
    assert_eq!(peer.app_version, "0.1.0");
    assert_eq!(peer.protocol_version, 1);
    assert_eq!(peer.service_port, 42424);
    assert!(peer.pairing_available);
    assert_eq!(peer.source, DiscoverySource::Mdns);
    assert_eq!(peer.addresses, vec!["192.168.1.21:42424"]);
}

#[test]
fn mdns_self_filter_uses_device_id() {
    let peer_identity = test_identity("local-device");
    let service = build_service_info(
        &peer_identity,
        42424,
        &[IpAddr::V4(Ipv4Addr::new(192, 168, 1, 22))],
    )
    .expect("service info");
    let peer = service_info_to_peer(&service).expect("peer");

    assert!(should_ignore_peer(&peer, "local-device"));
    assert!(!should_ignore_peer(&peer, "other-device"));
}

fn test_identity(device_id: &str) -> DeviceIdentity {
    DeviceIdentity {
        schema_version: 1,
        device_id: device_id.into(),
        device_name: "FlowLink Test".into(),
        os: OsType::Windows,
        arch: ArchType::X86_64,
        app_version: "0.1.0".into(),
        protocol_version: 1,
        public_key: vec![1, 2, 3],
        private_key_ref: PrivateKeyRef::FileEncrypted {
            path: "identity.key".into(),
        },
        created_at_ms: 1,
    }
}
