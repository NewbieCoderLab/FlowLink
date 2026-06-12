use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use flowlink_lib::{
    discovery::udp::{
        announce_to_peer, broadcast_destinations, build_announce, decode_announce,
        peer_from_announce, should_ignore_announce,
    },
    identity::{DeviceIdentity, PrivateKeyRef},
    protocol::messages::{ArchType, DiscoverySource, OsType},
};

#[test]
fn udp_announce_json_contains_identity_and_port() {
    let identity = test_identity("device-a");

    let announce = build_announce(&identity, 42424);

    assert_eq!(announce.v, 1);
    assert_eq!(announce.id, "device-a");
    assert_eq!(announce.name, "FlowLink Test");
    assert_eq!(announce.os, "windows");
    assert_eq!(announce.arch, "x86_64");
    assert_eq!(announce.app_version, "0.1.0");
    assert_eq!(announce.protocol_version, 1);
    assert_eq!(announce.port, 42424);
    assert!(announce.pairing);
}

#[test]
fn udp_announce_can_be_sent_to_socket() {
    let identity = test_identity("device-a");
    let Ok(receiver) = UdpSocket::bind("127.0.0.1:0") else {
        eprintln!("skipping UDP socket smoke test because loopback bind is not permitted");
        return;
    };
    receiver
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .expect("timeout");
    let destination = receiver.local_addr().expect("receiver address");
    let Ok(sender) = UdpSocket::bind("127.0.0.1:0") else {
        eprintln!("skipping UDP socket smoke test because loopback sender bind is not permitted");
        return;
    };

    if let Err(err) = announce_to_peer(&sender, &identity, 42424, destination) {
        if err.to_string().contains("Permission denied")
            || err.to_string().contains("Operation not permitted")
        {
            eprintln!("skipping UDP socket smoke test because loopback send is not permitted");
            return;
        }
        panic!("send announce: {err}");
    }

    let mut buffer = [0u8; 1024];
    let (len, _source) = receiver.recv_from(&mut buffer).expect("receive announce");
    let announce = decode_announce(&buffer[..len]).expect("decode announce");
    assert_eq!(announce.id, "device-a");
}

#[test]
fn udp_announce_becomes_discovered_peer() {
    let identity = test_identity("peer-a");
    let announce = build_announce(&identity, 42424);
    let source = SocketAddr::from((Ipv4Addr::new(192, 168, 1, 33), 42425));

    let peer = peer_from_announce(&announce, source).expect("peer");

    assert_eq!(peer.device_id, "peer-a");
    assert_eq!(peer.device_name, "FlowLink Test");
    assert_eq!(peer.os, OsType::Windows);
    assert_eq!(peer.arch, ArchType::X86_64);
    assert_eq!(peer.app_version, "0.1.0");
    assert_eq!(peer.protocol_version, 1);
    assert_eq!(peer.addresses, vec!["192.168.1.33:42424"]);
    assert_eq!(peer.service_port, 42424);
    assert!(peer.pairing_available);
    assert_eq!(peer.source, DiscoverySource::UdpBroadcast);
}

#[test]
fn udp_self_filter_uses_device_id() {
    let identity = test_identity("local-device");
    let announce = build_announce(&identity, 42424);

    assert!(should_ignore_announce(&announce, "local-device"));
    assert!(!should_ignore_announce(&announce, "other-device"));
}

#[test]
fn udp_announce_rejects_unspecified_source_address() {
    let identity = test_identity("peer-a");
    let announce = build_announce(&identity, 42424);
    let source = SocketAddr::from((IpAddr::V4(Ipv4Addr::UNSPECIFIED), 42425));

    assert!(peer_from_announce(&announce, source).is_err());
}

#[test]
fn udp_broadcast_destinations_include_limited_broadcast() {
    assert!(broadcast_destinations(42425).contains(&SocketAddr::from((Ipv4Addr::BROADCAST, 42425))));
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
