use super::*;
use tempfile::TempDir;

#[test]
fn kimi_adapter_default_paths() {
    let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
    std::env::remove_var("KIMI_CODE_HOME");
    let adapter = KimiAdapter;
    assert_eq!(adapter.kind(), HarnessKind::Kimi);
    let home = PathBuf::from("/tmp/home");
    assert_eq!(
        adapter.default_config_path(&home),
        PathBuf::from("/tmp/home/.kimi-code/mcp.json")
    );
}

#[test]
fn kimi_adapter_respects_kimi_code_home_env() {
    let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
    let adapter = KimiAdapter;
    let home = PathBuf::from("/tmp/home");
    std::env::set_var("KIMI_CODE_HOME", "/custom/kimi/dir");
    let path = adapter.default_config_path(&home);
    std::env::remove_var("KIMI_CODE_HOME");
    assert_eq!(path, PathBuf::from("/custom/kimi/dir/mcp.json"));
}

#[test]
fn registers_and_unregisters_native_kimi_mcp_server() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp.json");

    let mut env = BTreeMap::new();
    env.insert("LOG_LEVEL".to_string(), "info".to_string());

    register_kimi_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: KimiMcpConfig = serde_json::from_str(&content).unwrap();
    assert!(config.mcp_servers.contains_key("codegraph"));

    let codegraph = &config.mcp_servers["codegraph"];
    assert_eq!(codegraph.command, "codegraph");
    assert_eq!(codegraph.args, vec!["mcp"]);
    assert_eq!(codegraph.env.get("LOG_LEVEL").unwrap(), "info");

    // Verify zero OpenCode keys leak into JSON
    assert!(!content.contains("plugin"));
    assert!(!content.contains("skills.paths"));

    // Unregister
    unregister_kimi_mcp_server(&config_path, "codegraph").unwrap();
    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let config_after: KimiMcpConfig = serde_json::from_str(&content_after).unwrap();
    assert!(!config_after.mcp_servers.contains_key("codegraph"));
}

#[test]
fn replaces_env_map_cleanly_on_re_registration() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp.json");

    let mut env1 = BTreeMap::new();
    env1.insert("OLD_KEY".to_string(), "old_val".to_string());
    register_kimi_mcp_server(&config_path, "engram", "engram", &["serve"], &env1).unwrap();

    let mut env2 = BTreeMap::new();
    env2.insert("NEW_KEY".to_string(), "new_val".to_string());
    register_kimi_mcp_server(&config_path, "engram", "engram", &["serve"], &env2).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: KimiMcpConfig = serde_json::from_str(&content).unwrap();
    let engram_env = &config.mcp_servers["engram"].env;
    assert!(!engram_env.contains_key("OLD_KEY"));
    assert_eq!(engram_env.get("NEW_KEY").unwrap(), "new_val");

    // Re-register with empty env map -> removes env key from JSON
    let empty_env = BTreeMap::new();
    register_kimi_mcp_server(&config_path, "engram", "engram", &["serve"], &empty_env).unwrap();
    let content_empty = std::fs::read_to_string(&config_path).unwrap();
    assert!(!content_empty.contains("\"env\""));

    let config_empty: KimiMcpConfig = serde_json::from_str(&content_empty).unwrap();
    assert!(config_empty.mcp_servers["engram"].env.is_empty());
}
