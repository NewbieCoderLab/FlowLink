use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

use crate::{
    app::context::TrustedPeerStore,
    config::{AppConfig, LayoutConfig},
    identity::DeviceIdentity,
};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutStore {
    pub schema_version: u16,
    pub layouts: Vec<LayoutConfig>,
}

pub struct StorageManager {
    pub base_dir: PathBuf,
}

impl StorageManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn ensure_base_dir(&self) -> Result<(), StorageError> {
        fs::create_dir_all(&self.base_dir).map_err(|err| StorageError::Io(err.to_string()))
    }

    pub fn load_identity(&self) -> Result<DeviceIdentity, StorageError> {
        self.ensure_base_dir()?;
        let path = self.base_dir.join("identity.json");
        read_or_default(&path, DeviceIdentity::generate)
    }

    pub fn load_config(&self) -> Result<AppConfig, StorageError> {
        self.ensure_base_dir()?;
        let path = self.base_dir.join("config.json");
        read_or_default(&path, AppConfig::default)
    }

    pub fn load_trusted_peers(&self) -> Result<TrustedPeerStore, StorageError> {
        self.ensure_base_dir()?;
        let path = self.base_dir.join("trusted_peers.json");
        read_or_default(&path, || TrustedPeerStore {
            schema_version: 1,
            peers: Vec::new(),
        })
    }

    pub fn load_layouts(&self) -> Result<LayoutStore, StorageError> {
        self.ensure_base_dir()?;
        let path = self.base_dir.join("layouts.json");
        read_or_default(&path, || LayoutStore {
            schema_version: 1,
            layouts: Vec::new(),
        })
    }

    pub fn save_layouts(&self, layouts: &LayoutStore) -> Result<(), StorageError> {
        self.ensure_base_dir()?;
        write_json_atomic(&self.base_dir.join("layouts.json"), layouts)
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis() as u64)
        .unwrap_or_default()
}

fn read_or_default<T, F>(path: &Path, create_default: F) -> Result<T, StorageError>
where
    T: Serialize + DeserializeOwned,
    F: FnOnce() -> T,
{
    if path.exists() {
        let content = fs::read_to_string(path).map_err(|err| StorageError::Io(err.to_string()))?;
        serde_json::from_str(&content).map_err(|err| StorageError::Serialization(err.to_string()))
    } else {
        let default_value = create_default();
        write_json_atomic(path, &default_value)?;
        Ok(default_value)
    }
}

pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), StorageError> {
    let tmp_path = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| StorageError::Serialization(err.to_string()))?;
    fs::write(&tmp_path, bytes).map_err(|err| StorageError::Io(err.to_string()))?;
    fs::rename(&tmp_path, path).map_err(|err| StorageError::Io(err.to_string()))
}

