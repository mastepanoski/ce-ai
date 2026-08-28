use super::*;
use tempfile::TempDir;

use crate::harness::tests::HARNESS_ENV_LOCK;

#[test]
fn fx_adapter_default_paths() {
    let _guard = HARNESS_ENV_LOCK.lock().unwrap();
    std::env::remove_var("FX_HOME");

    let adapter = FxAdapter;
    assert_eq!(adapter.kind(), HarnessKind::Fx);
    let home = PathBuf::from("/tmp/home");
    assert_eq!(
        adapter.kind().harness_dir(&home),
        PathBuf::from("/tmp/home/.fx")
    );
    assert_eq!(
        adapter.default_config_path(&home),
        PathBuf::from("/tmp/home/.fx/mcp.json")
    );
    assert_eq!(
        adapter.default_config_path(&PathBuf::from("/tmp/home/.fx")),
        PathBuf::from("/tmp/home/.fx/mcp.json")
    );
    assert_eq!(
        adapter.default_config_path(&PathBuf::from("/tmp/home/.fx/mcp.json")),
        PathBuf::from("/tmp/home/.fx/mcp.json")
    );
}

#[test]
fn fx_adapter_respects_fx_home_env() {
    let _guard = HARNESS_ENV_LOCK.lock().unwrap();
    std::env::set_var("FX_HOME", "/custom/fx/dir");

    let adapter = FxAdapter;
    let home = PathBuf::from("/tmp/home");
    assert_eq!(
        adapter.kind().harness_dir(&home),
        PathBuf::from("/custom/fx/dir")
    );
    assert_eq!(
        adapter.default_config_path(&home),
        PathBuf::from("/custom/fx/dir/mcp.json")
    );

    std::env::remove_var("FX_HOME");
}

#[test]
fn registers_and_unregisters_native_fx_mcp_server() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp.json");

    let mut env = BTreeMap::new();
    env.insert("KEY".to_string(), "VAL".to_string());

    register_fx_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("\"mcp\":"));
    assert!(content.contains("\"type\": \"local\""));
    assert!(content.contains("\"command\": ["));

    let config: FxMcpConfig = serde_json::from_str(&content).unwrap();

    assert!(config.mcp.contains_key("codegraph"));
    let server = &config.mcp["codegraph"];
    assert_eq!(server.r#type.as_deref(), Some("local"));
    assert_eq!(server.command, vec!["codegraph", "mcp"]);
    assert_eq!(
        server.environment.get("KEY").map(|s| s.as_str()),
        Some("VAL")
    );

    unregister_fx_mcp_server(&config_path, "codegraph").unwrap();
    assert!(!config_path.exists());
}

#[test]
fn preserves_existing_user_fx_keys_and_extra_fields() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp.json");

    let initial_json = r#"{
        "user_setting": "active",
        "mcp": {
            "user_remote": {
                "type": "http",
                "url": "https://mcp.example.com"
            }
        }
    }"#;
    std::fs::write(&config_path, initial_json).unwrap();

    let env = BTreeMap::new();
    register_fx_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: FxMcpConfig = serde_json::from_str(&content).unwrap();

    assert_eq!(config.extra.get("user_setting").unwrap(), "active");
    assert!(config.mcp.contains_key("user_remote"));
    assert!(config.mcp.contains_key("codegraph"));

    unregister_fx_mcp_server(&config_path, "codegraph").unwrap();

    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let config_after: FxMcpConfig = serde_json::from_str(&content_after).unwrap();
    assert!(config_after.mcp.contains_key("user_remote"));
    assert!(!config_after.mcp.contains_key("codegraph"));
}

#[test]
fn fx_adapter_default_paths_ignores_preexisting_mcp_json_in_home_dir() {
    let _guard = HARNESS_ENV_LOCK.lock().unwrap();
    std::env::remove_var("FX_HOME");

    let tmp = TempDir::new().unwrap();
    // Create an unrelated mcp.json in the home directory root
    let home_mcp_json = tmp.path().join("mcp.json");
    std::fs::write(&home_mcp_json, "{}").unwrap();

    let adapter = FxAdapter;
    // Even though home/mcp.json exists, default_config_path must deterministically return home/.fx/mcp.json
    assert_eq!(
        adapter.default_config_path(tmp.path()),
        tmp.path().join(".fx").join("mcp.json")
    );
}

#[test]
fn cleans_stale_type_from_extra_map_on_re_registration() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp.json");
    let initial_json = r#"{
        "mcp": {
            "codegraph": {
                "type": "custom_legacy",
                "custom_field": "keep_me"
            }
        }
    }"#;
    std::fs::write(&config_path, initial_json).unwrap();

    let env = BTreeMap::new();
    register_fx_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: FxMcpConfig = serde_json::from_str(&content).unwrap();
    let server = &config.mcp["codegraph"];
    assert_eq!(server.r#type.as_deref(), Some("local"));
    assert!(!server.extra.contains_key("type"));
    assert_eq!(server.extra.get("custom_field").unwrap(), "keep_me");
}

#[test]
#[cfg(unix)]
fn unregister_fx_mcp_server_propagates_io_errors() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    let dir_path = tmp.path().join("readonly_dir");
    std::fs::create_dir(&dir_path).unwrap();
    let config_path = dir_path.join("mcp.json");
    std::fs::write(&config_path, r#"{"mcp":{"codegraph":{"type":"local"}}}"#).unwrap();

    let mut perms = std::fs::metadata(&dir_path).unwrap().permissions();
    perms.set_mode(0o555);
    std::fs::set_permissions(&dir_path, perms).unwrap();

    let res = unregister_fx_mcp_server(&config_path, "codegraph");
    assert!(matches!(res, Err(CeError::Io(_))));

    let mut perms = std::fs::metadata(&dir_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&dir_path, perms).unwrap();
}
