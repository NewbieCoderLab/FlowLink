use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::identity::PrivateKeyRef;

const DEFAULT_PRIVATE_KEY_FILE: &str = "identity.key";

#[derive(Debug, Error)]
pub enum SecretStoreError {
    #[error("io error: {0}")]
    Io(String),
    #[error("invalid private key path")]
    InvalidPath,
}

pub trait SecretStore {
    fn load_private_key(&self) -> Result<Vec<u8>, SecretStoreError>;
    fn save_private_key(&self, key: &[u8]) -> Result<PrivateKeyRef, SecretStoreError>;
}

#[derive(Debug, Clone)]
pub struct FileSecretStore {
    base_dir: PathBuf,
    key_file_name: String,
}

impl FileSecretStore {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            key_file_name: DEFAULT_PRIVATE_KEY_FILE.into(),
        }
    }

    pub fn with_file_name(base_dir: PathBuf, key_file_name: impl Into<String>) -> Self {
        Self {
            base_dir,
            key_file_name: key_file_name.into(),
        }
    }

    pub fn key_path(&self) -> PathBuf {
        self.base_dir.join(&self.key_file_name)
    }

    fn key_ref(&self) -> PrivateKeyRef {
        PrivateKeyRef::FileEncrypted {
            path: self.key_file_name.clone(),
        }
    }
}

impl SecretStore for FileSecretStore {
    fn load_private_key(&self) -> Result<Vec<u8>, SecretStoreError> {
        fs::read(self.key_path()).map_err(|err| SecretStoreError::Io(err.to_string()))
    }

    fn save_private_key(&self, key: &[u8]) -> Result<PrivateKeyRef, SecretStoreError> {
        fs::create_dir_all(&self.base_dir).map_err(|err| SecretStoreError::Io(err.to_string()))?;
        write_secret_atomic(&self.key_path(), key)?;
        Ok(self.key_ref())
    }
}

fn write_secret_atomic(path: &Path, key: &[u8]) -> Result<(), SecretStoreError> {
    let parent = path.parent().ok_or(SecretStoreError::InvalidPath)?;
    fs::create_dir_all(parent).map_err(|err| SecretStoreError::Io(err.to_string()))?;

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(SecretStoreError::InvalidPath)?;
    let tmp_path = path.with_file_name(format!("{file_name}.tmp"));
    match fs::remove_file(&tmp_path) {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(SecretStoreError::Io(err.to_string())),
    }

    fs::write(&tmp_path, key).map_err(|err| SecretStoreError::Io(err.to_string()))?;
    fs::rename(&tmp_path, path).map_err(|err| SecretStoreError::Io(err.to_string()))
}
