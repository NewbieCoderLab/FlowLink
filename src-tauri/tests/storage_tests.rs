use tempfile::tempdir;

use flowlink_lib::{
    config::{AppConfig, LayoutConfig},
    identity::PrivateKeyRef,
    protocol::messages::LayoutDirection,
    storage::{
        files::{write_json_atomic, LayoutStore, StorageError, StorageManager},
        secret::{FileSecretStore, SecretStore},
    },
};
use std::fs;

#[test]
fn storage_manager_writes_layouts_atomically() {
    let dir = tempdir().expect("tempdir");
    let storage = StorageManager::new(dir.path().to_path_buf());
    let layouts = LayoutStore {
        schema_version: 1,
        layouts: vec![LayoutConfig {
            peer_id: "peer-1".into(),
            direction: LayoutDirection::Right,
            edge_thickness_px: 1,
            corner_guard_px: 32,
            enabled: true,
            updated_at_ms: 1,
        }],
    };

    storage.save_layouts(&layouts).expect("save");
    let loaded = storage.load_layouts().expect("load");
    assert_eq!(loaded.layouts.len(), 1);
    assert_eq!(loaded.layouts[0].peer_id, "peer-1");
}

#[test]
fn write_json_atomic_writes_readable_json() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("config.json");
    let config = AppConfig::default();

    write_json_atomic(&path, &config).expect("write");
    let content = fs::read_to_string(&path).expect("read");
    let loaded: AppConfig = serde_json::from_str(&content).expect("parse");

    assert_eq!(loaded.schema_version, config.schema_version);
}

#[test]
fn write_json_atomic_replaces_existing_file() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("config.json");
    let mut config = AppConfig::default();

    write_json_atomic(&path, &config).expect("first write");
    config.schema_version += 1;
    write_json_atomic(&path, &config).expect("replace");
    let content = fs::read_to_string(&path).expect("read");
    let loaded: AppConfig = serde_json::from_str(&content).expect("parse");

    assert_eq!(loaded.schema_version, config.schema_version);
}

#[test]
fn missing_config_file_is_created_with_default_value() {
    let dir = tempdir().expect("tempdir");
    let storage = StorageManager::new(dir.path().to_path_buf());
    let path = dir.path().join("config.json");

    let config = storage.load_config().expect("load default");

    assert!(path.exists());
    assert_eq!(config.schema_version, AppConfig::default().schema_version);
}

#[test]
fn corrupt_config_is_backed_up_and_replaced_with_default() {
    let dir = tempdir().expect("tempdir");
    let storage = StorageManager::new(dir.path().to_path_buf());
    let path = dir.path().join("config.json");
    let corrupt_content = "{not valid json";
    fs::write(&path, corrupt_content).expect("write corrupt");

    let config = storage.load_config().expect("recover default");
    let backups = fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(Result::ok)
        .filter(|entry| is_corrupt_backup(entry.file_name().to_string_lossy().as_ref()))
        .collect::<Vec<_>>();
    let backup_name = backups[0].file_name().to_string_lossy().to_string();
    let timestamp = backup_name
        .strip_prefix("config.json.corrupt.")
        .expect("backup prefix");
    let replaced: AppConfig =
        serde_json::from_str(&fs::read_to_string(&path).expect("read replacement"))
            .expect("replacement is valid json");

    assert_eq!(config.schema_version, AppConfig::default().schema_version);
    assert_eq!(replaced.schema_version, AppConfig::default().schema_version);
    assert_eq!(backups.len(), 1);
    assert!(timestamp.parse::<u64>().is_ok());
    assert_eq!(
        fs::read_to_string(backups[0].path()).expect("read backup"),
        corrupt_content
    );
    assert!(path.exists());
}

#[test]
fn config_read_io_failure_returns_io_error_without_backup() {
    let dir = tempdir().expect("tempdir");
    let storage = StorageManager::new(dir.path().to_path_buf());
    let path = dir.path().join("config.json");
    fs::create_dir(&path).expect("create directory where config file should be");

    let result = storage.load_config();

    assert!(matches!(result, Err(StorageError::Io(_))));
    assert!(path.is_dir());
    let backup_count = fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(Result::ok)
        .filter(|entry| is_corrupt_backup(entry.file_name().to_string_lossy().as_ref()))
        .count();
    assert_eq!(backup_count, 0);
}

#[test]
fn schema_mismatch_returns_error_without_replacing_file() {
    let dir = tempdir().expect("tempdir");
    let storage = StorageManager::new(dir.path().to_path_buf());
    let path = dir.path().join("config.json");
    fs::write(&path, r#"{"schema_version":"wrong"}"#).expect("write invalid schema");

    let result = storage.load_config();

    assert!(matches!(result, Err(StorageError::Serialization(_))));
    assert!(path.exists());
    let backup_count = fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(Result::ok)
        .filter(|entry| is_corrupt_backup(entry.file_name().to_string_lossy().as_ref()))
        .count();
    assert_eq!(backup_count, 0);
}

#[test]
fn file_secret_store_saves_and_loads_private_key() {
    let dir = tempdir().expect("tempdir");
    let store = FileSecretStore::new(dir.path().to_path_buf());
    let key = [7_u8; 32];

    let key_ref = store.save_private_key(&key).expect("save private key");
    let loaded = store.load_private_key().expect("load private key");

    assert_eq!(loaded, key);
    assert_eq!(
        fs::read(dir.path().join("identity.key")).expect("read key file"),
        key
    );
    assert!(matches!(
        key_ref,
        PrivateKeyRef::FileEncrypted { ref path } if path == "identity.key"
    ));
}

#[cfg(unix)]
#[test]
fn file_secret_store_writes_private_key_with_owner_only_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().expect("tempdir");
    let store = FileSecretStore::new(dir.path().to_path_buf());

    store
        .save_private_key(&[9_u8; 32])
        .expect("save private key");
    let mode = fs::metadata(dir.path().join("identity.key"))
        .expect("metadata")
        .permissions()
        .mode()
        & 0o777;

    assert_eq!(mode, 0o600);
}

fn is_corrupt_backup(file_name: &str) -> bool {
    file_name.starts_with("config.json.corrupt.")
}
