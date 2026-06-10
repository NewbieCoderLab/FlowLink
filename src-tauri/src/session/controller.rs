use crate::session::state::{ControlOwner, SessionSnapshot, SessionState};

pub fn emergency_disconnect(snapshot: &mut SessionSnapshot) {
    snapshot.state = SessionState::Disconnected;
    snapshot.control_owner = ControlOwner::Local;
    snapshot.peer_id = None;
    snapshot.peer_name = None;
    snapshot.connected_since_ms = None;
    snapshot.updated_at_ms = crate::storage::files::now_ms();
}

