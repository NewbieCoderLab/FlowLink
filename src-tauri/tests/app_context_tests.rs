use flowlink_lib::app::context::AppContext;
use tempfile::tempdir;

#[test]
fn app_context_starts_without_demo_discovered_peer() {
    let dir = tempdir().expect("tempdir");
    let app = AppContext::load_or_default(dir.path().to_path_buf()).expect("load app context");

    assert!(app.list_discovered_devices().is_empty());
    assert_eq!(app.diagnostics().discovered_peer_count, 0);
}
