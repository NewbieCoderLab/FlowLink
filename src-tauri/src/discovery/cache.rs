use std::collections::HashMap;

use crate::{discovery::DiscoveredPeer, storage::files::now_ms};

pub struct DiscoveryCache {
    stale_after_ms: u64,
    peers: HashMap<String, DiscoveredPeer>,
}

impl DiscoveryCache {
    pub fn new(stale_after_ms: u64) -> Self {
        Self {
            stale_after_ms,
            peers: HashMap::new(),
        }
    }

    pub fn upsert(&mut self, peer: DiscoveredPeer) {
        self.peers.insert(peer.device_id.clone(), peer);
    }

    pub fn list(&self) -> Vec<DiscoveredPeer> {
        let cutoff = now_ms().saturating_sub(self.stale_after_ms);
        let mut peers = self
            .peers
            .values()
            .filter(|peer| peer.last_seen_ms >= cutoff)
            .cloned()
            .collect::<Vec<_>>();
        peers.sort_by(|left, right| right.last_seen_ms.cmp(&left.last_seen_ms));
        peers
    }
}

