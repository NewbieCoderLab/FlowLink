use flowlink_lib::{
    discovery::{cache::DiscoveryCache, DiscoveredPeer},
    protocol::messages::{ArchType, DiscoverySource, OsType},
};

#[test]
fn discovery_cache_evicts_stale_peers_and_returns_them() {
    let mut cache = DiscoveryCache::new(10_000);
    cache.upsert(discovered_peer("stale-peer", 1_000));
    cache.upsert(discovered_peer("fresh-peer", 11_000));

    let evicted = cache.evict_stale(12_000);

    assert_eq!(evicted.len(), 1);
    assert_eq!(evicted[0].device_id, "stale-peer");
    let remaining = cache.list_at(12_000);
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].device_id, "fresh-peer");
}

#[test]
fn discovery_cache_keeps_peer_at_stale_boundary() {
    let mut cache = DiscoveryCache::new(10_000);
    cache.upsert(discovered_peer("boundary-peer", 2_000));

    let evicted = cache.evict_stale(12_000);

    assert!(evicted.is_empty());
    assert_eq!(cache.list_at(12_000).len(), 1);
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
