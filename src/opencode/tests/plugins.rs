use tempfile::tempdir;

use super::*;
use crate::state::diff::sha256_hex;

#[test]
fn copies_loader_into_managed_plugins_dir() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("ce-source");
    let loader_src = source.join(".opencode/plugins/compound-engineering.js");
    std::fs::create_dir_all(loader_src.parent().unwrap()).unwrap();
    let loader_bytes = b"export default function ceLoader() {}";
    std::fs::write(&loader_src, loader_bytes).unwrap();

    let config_dir = dir.path().join("opencode-config");
    let installed = install_loader(&source, &config_dir).unwrap();

    assert_eq!(installed.path, "plugins/compound-engineering.js");
    assert_eq!(installed.sha256, sha256_hex(loader_bytes));
    let dest = config_dir.join("compound-engineering/plugins/compound-engineering.js");
    assert_eq!(std::fs::read(&dest).unwrap(), loader_bytes);
}

#[test]
fn skills_path_points_at_managed_skills_dir() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("opencode-config");
    assert_eq!(
        skills_path(&config_dir),
        config_dir.join("compound-engineering/skills")
    );
}

#[test]
fn install_loader_falls_back_to_builtin_when_missing() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("empty-source");
    let config_dir = dir.path().join("opencode-config");

    let installed = install_loader(&source, &config_dir).unwrap();
    assert_eq!(installed.path, "plugins/compound-engineering.js");
    let dest = config_dir.join("compound-engineering/plugins/compound-engineering.js");
    let content = std::fs::read_to_string(&dest).unwrap();
    assert!(content.contains("session.created"));
    assert!(content.contains("ce-ai"));
}

#[test]
fn ensures_and_removes_session_start_plugin_lifecycle() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("opencode-config");

    assert!(!has_session_start_plugin(&config_dir));

    let changed = ensure_session_start_plugin(&config_dir).unwrap();
    assert!(changed);
    assert!(has_session_start_plugin(&config_dir));

    // Idempotent second call
    let changed_second = ensure_session_start_plugin(&config_dir).unwrap();
    assert!(!changed_second);
    assert!(has_session_start_plugin(&config_dir));

    // Remove
    let removed = remove_session_start_plugin(&config_dir).unwrap();
    assert!(removed);
    assert!(!has_session_start_plugin(&config_dir));

    let removed_second = remove_session_start_plugin(&config_dir).unwrap();
    assert!(!removed_second);
}

#[test]
fn preserves_user_plugins_in_opencode_json() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("opencode-config");
    std::fs::create_dir_all(&config_dir).unwrap();

    let initial = serde_json::json!({
        "plugin": ["custom-user-plugin", "@org/telemetry"],
        "model": "claude-3-5-sonnet"
    });
    std::fs::write(
        config_dir.join("opencode.json"),
        serde_json::to_string_pretty(&initial).unwrap(),
    )
    .unwrap();

    ensure_session_start_plugin(&config_dir).unwrap();
    let text = std::fs::read_to_string(config_dir.join("opencode.json")).unwrap();
    let val: serde_json::Value = serde_json::from_str(&text).unwrap();
    let plugins = val["plugin"].as_array().unwrap();
    assert_eq!(plugins.len(), 3);
    assert_eq!(plugins[0], "custom-user-plugin");
    assert_eq!(plugins[1], "@org/telemetry");
    assert_eq!(val["model"], "claude-3-5-sonnet");

    remove_session_start_plugin(&config_dir).unwrap();
    let text_after = std::fs::read_to_string(config_dir.join("opencode.json")).unwrap();
    let val_after: serde_json::Value = serde_json::from_str(&text_after).unwrap();
    let plugins_after = val_after["plugin"].as_array().unwrap();
    assert_eq!(plugins_after.len(), 2);
    assert_eq!(plugins_after[0], "custom-user-plugin");
    assert_eq!(plugins_after[1], "@org/telemetry");
    assert_eq!(val_after["model"], "claude-3-5-sonnet");
}
