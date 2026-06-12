use std::{cmp::Reverse, collections::HashMap};

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

    pub fn upsert(&mut self, peer: DiscoveredPeer) -> CacheUpsertOutcome {
        let is_new = !self.peers.contains_key(&peer.device_id);
        self.peers.insert(peer.device_id.clone(), peer);
        CacheUpsertOutcome { is_new }
    }

    pub fn list(&self) -> Vec<DiscoveredPeer> {
        self.list_at(now_ms())
    }

    pub fn list_at(&self, now_ms: u64) -> Vec<DiscoveredPeer> {
        let cutoff = now_ms.saturating_sub(self.stale_after_ms);
        let mut peers = self
            .peers
            .values()
            .filter(|peer| peer.last_seen_ms >= cutoff)
            .cloned()
            .collect::<Vec<_>>();
        peers.sort_by_key(|peer| Reverse(peer.last_seen_ms));
        peers
    }

    pub fn evict_stale(&mut self, now_ms: u64) -> Vec<DiscoveredPeer> {
        let cutoff = now_ms.saturating_sub(self.stale_after_ms);
        let stale_ids = self
            .peers
            .iter()
            .filter_map(|(device_id, peer)| (peer.last_seen_ms < cutoff).then(|| device_id.clone()))
            .collect::<Vec<_>>();

        stale_ids
            .into_iter()
            .filter_map(|device_id| self.peers.remove(&device_id))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheUpsertOutcome {
    pub is_new: bool,
}
