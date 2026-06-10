use tempfile::tempdir;

use flowlink::{
    config::LayoutConfig,
    protocol::messages::LayoutDirection,
    storage::files::{LayoutStore, StorageManager},
};

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

