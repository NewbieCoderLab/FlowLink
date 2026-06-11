use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    config::{AppConfig, LayoutConfig},
    discovery::{cache::DiscoveryCache, DiscoveredPeer},
    identity::DeviceIdentity,
    input::{platform_input, InputPlatform},
    platform::PermissionStatus,
    session::state::SessionSnapshot,
    storage::files::StorageManager,
    telemetry::metrics::DiagnosticsSnapshot,
};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedPeerStore {
    pub schema_version: u16,
    pub peers: Vec<crate::config::TrustedPeer>,
}

pub struct AppContext {
    pub storage: StorageManager,
    pub local_identity: DeviceIdentity,
    pub config: AppConfig,
    pub trusted_peers: TrustedPeerStore,
    pub discovery: DiscoveryCache,
    pub input: Box<dyn InputPlatform>,
    pub permissions: PermissionStatus,
    pub session: SessionSnapshot,
}

impl AppContext {
    pub fn load_or_default(config_dir: PathBuf) -> Result<Self, AppError> {
        let storage = StorageManager::new(config_dir);
        let local_identity = storage
            .load_identity()
            .map_err(|err| AppError::Storage(err.to_string()))?;
        let config = storage
            .load_config()
            .map_err(|err| AppError::Storage(err.to_string()))?;
        let trusted_peers = storage
            .load_trusted_peers()
            .map_err(|err| AppError::Storage(err.to_string()))?;
        let layouts = storage
            .load_layouts()
            .map_err(|err| AppError::Storage(err.to_string()))?;
        let mut merged_config = config;
        merged_config.layouts = layouts.layouts;
        crate::config::validate::validate_app_config(&merged_config)
            .map_err(|err| AppError::Validation(err.to_string()))?;

        let discovery = DiscoveryCache::new(merged_config.discovery.stale_after_ms);

        let input = platform_input();
        let permissions = input.permissions();

        Ok(Self {
            storage,
            permissions,
            input,
            local_identity,
            config: merged_config,
            trusted_peers,
            discovery,
            session: SessionSnapshot::default(),
        })
    }

    pub fn save_layout(&mut self, layout: LayoutConfig) -> Result<(), AppError> {
        self.config.upsert_layout(layout.clone());
        self.storage
            .save_layouts(&crate::storage::files::LayoutStore {
                schema_version: self.config.schema_version,
                layouts: self.config.layouts.clone(),
            })
            .map_err(|err| AppError::Storage(err.to_string()))
    }

    pub fn list_discovered_devices(&self) -> Vec<DiscoveredPeer> {
        self.discovery.list()
    }

    pub fn diagnostics(&self) -> DiagnosticsSnapshot {
        DiagnosticsSnapshot {
            discovered_peer_count: self.discovery.list().len(),
            trusted_peer_count: self.trusted_peers.peers.len(),
            layout_count: self.config.layouts.len(),
            config_path: self.storage.base_dir.display().to_string(),
        }
    }

    pub fn refresh_permissions(&mut self) -> PermissionStatus {
        self.permissions = self.input.permissions();
        self.permissions.clone()
    }
}

#[derive(Clone)]
pub struct SharedAppState(pub Arc<RwLock<AppContext>>);

impl SharedAppState {
    pub fn new(context: AppContext) -> Self {
        Self(Arc::new(RwLock::new(context)))
    }
}
