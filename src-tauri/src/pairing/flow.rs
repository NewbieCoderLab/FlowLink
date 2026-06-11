use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::storage::files::now_ms;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingFlow {
    pub pairing_id: String,
    pub requested_at_ms: u64,
    pub expires_at_ms: u64,
}

impl PairingFlow {
    pub fn new() -> Self {
        let requested_at_ms = now_ms();
        Self {
            pairing_id: Uuid::new_v4().to_string(),
            requested_at_ms,
            expires_at_ms: requested_at_ms + 120_000,
        }
    }

    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms > self.expires_at_ms
    }
}

impl Default for PairingFlow {
    fn default() -> Self {
        Self::new()
    }
}
