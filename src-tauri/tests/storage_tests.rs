use tempfile::tempdir;

use flowlink::{
    config::{AppConfig, LayoutConfig},
    protocol::messages::LayoutDirection,
    storage::files::{write_json_atomic, LayoutStore, StorageError, StorageManager},
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
    fs::write(&path, "{not valid json").expect("write corrupt");

    let config = storage.load_config().expect("recover default");
    let backup_count = fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("config.json.corrupt.")
        })
        .count();

    assert_eq!(config.schema_version, AppConfig::default().schema_version);
    assert_eq!(backup_count, 1);
    assert!(path.exists());
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
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("config.json.corrupt.")
        })
        .count();
    assert_eq!(backup_count, 0);
}
