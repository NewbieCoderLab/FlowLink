use flowlink_lib::{
    app::context::AppContext,
    discovery::DiscoveredPeer,
    protocol::messages::{ArchType, DiscoverySource, OsType},
    storage::files::now_ms,
};
use tempfile::tempdir;

#[test]
fn app_context_reports_device_discovered_only_for_new_peers() {
    let dir = tempdir().expect("tempdir");
    let mut app = AppContext::load_or_default(dir.path().to_path_buf()).expect("context");

    let first = app.handle_discovered_peer(discovered_peer("peer-a", 10_000));
    let second = app.handle_discovered_peer(discovered_peer("peer-a", 11_000));

    assert_eq!(first.map(|peer| peer.device_id), Some("peer-a".into()));
    assert!(second.is_none());
}

#[test]
fn app_context_reports_stale_devices_after_evicting_them() {
    let dir = tempdir().expect("tempdir");
    let mut app = AppContext::load_or_default(dir.path().to_path_buf()).expect("context");
    let now = now_ms();
    app.handle_discovered_peer(discovered_peer("peer-a", now - 11_000));
    app.handle_discovered_peer(discovered_peer("peer-b", now));

    let stale = app.evict_stale_discovered_peers(now);

    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].device_id, "peer-a");
    assert_eq!(app.list_discovered_devices().len(), 1);
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
