use serde::{Deserialize, Serialize};

use crate::{input::types::Point, protocol::messages::TimestampMs, storage::files::now_ms};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Disconnected,
    Discovered,
    Pairing,
    Paired,
    Connecting,
    ConnectedIdle,
    ControllingRemote,
    ControlledByRemote,
    Reconnecting,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlOwner {
    Local,
    Remote,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: Option<String>,
    pub peer_id: Option<String>,
    pub peer_name: Option<String>,
    pub state: SessionState,
    pub control_owner: ControlOwner,
    pub local_pointer: Option<Point>,
    pub remote_pointer: Option<Point>,
    pub last_heartbeat_rtt_ms: Option<u32>,
    pub connected_since_ms: Option<TimestampMs>,
    pub updated_at_ms: TimestampMs,
}

impl Default for SessionSnapshot {
    fn default() -> Self {
        Self {
            session_id: None,
            peer_id: None,
            peer_name: None,
            state: SessionState::Disconnected,
            control_owner: ControlOwner::Local,
            local_pointer: None,
            remote_pointer: None,
            last_heartbeat_rtt_ms: None,
            connected_since_ms: None,
            updated_at_ms: now_ms(),
        }
    }
}
