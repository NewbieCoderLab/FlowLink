use std::time::Duration;

pub fn default_heartbeat_interval() -> Duration {
    Duration::from_millis(1000)
}
