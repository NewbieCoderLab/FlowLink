use flowlink_lib::{
    app::context::AppContext,
    discovery::DiscoveredPeer,
    protocol::messages::{ArchType, DiscoverySource, OsType},
    storage::files::now_ms,
};
use tempfile::tempdir;

#[test]
fn app_context_upserts_discovered_peer_from_discovery_task() {
    let dir = tempdir().expect("tempdir");
    let mut app = AppContext::load_or_default(dir.path().to_path_buf()).expect("context");
    let peer = discovered_peer("peer-a", now_ms());

    let outcome = app.upsert_discovered_peer(peer);

    assert!(outcome.is_new);
    assert_eq!(app.list_discovered_devices().len(), 1);
    assert_eq!(app.diagnostics().discovered_peer_count, 1);
}

#[test]
fn app_context_updates_existing_discovered_peer_without_marking_new() {
    let dir = tempdir().expect("tempdir");
    let mut app = AppContext::load_or_default(dir.path().to_path_buf()).expect("context");
    app.upsert_discovered_peer(discovered_peer("peer-a", now_ms()));

    let updated_at = now_ms() + 1;
    let outcome = app.upsert_discovered_peer(discovered_peer("peer-a", updated_at));

    assert!(!outcome.is_new);
    let peers = app.list_discovered_devices();
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].last_seen_ms, updated_at);
}

fn discovered_peer(device_id: &str, last_seen_ms: u64) -> DiscoveredPeer {
    DiscoveredPeer {
        device_id: device_id.into(),
        device_name: "Office PC".into(),
        os: OsType::Windows,
        arch: ArchType::X86_64,
        app_version: "0.1.0".into(),
        protocol_version: 1,
        addresses: vec!["192.168.1.42:42424".into()],
        service_port: 42424,
        pairing_available: true,
        last_seen_ms,
        source: DiscoverySource::UdpBroadcast,
    }
}
