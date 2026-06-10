pub mod hostname;
pub mod keys;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::protocol::messages::{ArchType, OsType, TimestampMs};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub schema_version: u16,
    pub device_id: String,
    pub device_name: String,
    pub os: OsType,
    pub arch: ArchType,
    pub app_version: String,
    pub protocol_version: u16,
    pub public_key: Vec<u8>,
    pub private_key_ref: PrivateKeyRef,
    pub created_at_ms: TimestampMs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PrivateKeyRef {
    FileEncrypted { path: String },
}

pub trait OsArchExt {
    fn os_name(&self) -> &'static str;
}

impl OsArchExt for DeviceIdentity {
    fn os_name(&self) -> &'static str {
        match self.os {
            OsType::Macos => "macos",
            OsType::Windows => "windows",
            OsType::Unknown => "unknown",
        }
    }
}

impl DeviceIdentity {
    pub fn generate() -> Self {
        let key_material = keys::generate_public_key_stub();
        Self {
            schema_version: 1,
            device_id: Uuid::new_v4().to_string(),
            device_name: hostname::default_device_name(),
            os: current_os(),
            arch: current_arch(),
            app_version: "0.1.0".into(),
            protocol_version: 1,
            public_key: key_material,
            private_key_ref: PrivateKeyRef::FileEncrypted {
                path: "identity.key".into(),
            },
            created_at_ms: crate::storage::files::now_ms(),
        }
    }
}

fn current_os() -> OsType {
    match std::env::consts::OS {
        "macos" => OsType::Macos,
        "windows" => OsType::Windows,
        _ => OsType::Unknown,
    }
}

fn current_arch() -> ArchType {
    match std::env::consts::ARCH {
        "x86_64" => ArchType::X86_64,
        "aarch64" => ArchType::Aarch64,
        _ => ArchType::Unknown,
    }
}

