use thiserror::Error;

use super::AppConfig;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("listen_port must be non-zero")]
    InvalidPort,
    #[error("heartbeat timeout must be greater than interval")]
    InvalidHeartbeatWindow,
}

pub fn validate_app_config(config: &AppConfig) -> Result<(), ValidationError> {
    if config.network.listen_port == 0 {
        return Err(ValidationError::InvalidPort);
    }
    if config.network.heartbeat_timeout_ms <= config.network.heartbeat_interval_ms {
        return Err(ValidationError::InvalidHeartbeatWindow);
    }
    Ok(())
}

