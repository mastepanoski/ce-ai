use super::*;
use crate::harness::tests::HARNESS_ENV_LOCK;
use tempfile::TempDir;

#[test]
fn agy_adapter_default_paths() {
    let _guard = HARNESS_ENV_LOCK.lock().unwrap();
    std::env::remove_var("ANTIGRAVITY_CONFIG_DIR");
    std::env::remove_var("GEMINI_HOME");

    let adapter = AgyAdapter;
    let home = Path::new("/tmp/home");

    assert_eq!(adapter.kind(), HarnessKind::Agy);
    assert_eq!(adapter.harness_dir(home), home.join(".gemini"));
    assert_eq!(
        adapter.default_config_path(home),
        home.join(".gemini/config/mcp_config.json")
    );
}

#[test]
fn agy_adapter_respects_env_overrides() {
    let _guard = HARNESS_ENV_LOCK.lock().unwrap();
    std::env::set_var("ANTIGRAVITY_CONFIG_DIR", "/custom/agy/dir");

    let adapter = AgyAdapter;
    let home = Path::new("/tmp/home");

    assert_eq!(adapter.harness_dir(home), PathBuf::from("/custom/agy/dir"));
    assert_eq!(
        adapter.default_config_path(home),
        PathBuf::from("/custom/agy/dir/config/mcp_config.json")
    );

    std::env::set_var("GEMINI_HOME", "/custom/gemini/dir");
    assert_eq!(
        adapter.harness_dir(home),
        PathBuf::from("/custom/agy/dir"),
        "ANTIGRAVITY_CONFIG_DIR takes precedence over GEMINI_HOME"
    );

    std::env::remove_var("ANTIGRAVITY_CONFIG_DIR");

    assert_eq!(
        adapter.harness_dir(home),
        PathBuf::from("/custom/gemini/dir")
    );
    assert_eq!(
        adapter.default_config_path(home),
        PathBuf::from("/custom/gemini/dir/config/mcp_config.json")
    );
    std::env::remove_var("GEMINI_HOME");
}

#[test]
fn register_and_unregister_agy_mcp_server_preserves_remote_server_url() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp_config.json");

    // Seed with a remote server using serverUrl
    let initial_json = r#"{
      "custom_root_key": "active",
      "mcpServers": {
        "remote_server": {
          "serverUrl": "https://mcp.example.com/sse",
          "headers": { "Auth": "Bearer token" }
        }
      }
    }"#;
    std::fs::write(&config_path, initial_json).unwrap();

    let mut env = BTreeMap::new();
    env.insert("KEY".to_string(), "VAL".to_string());

    register_agy_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: AgyMcpConfig = serde_json::from_str(&content).unwrap();

    assert_eq!(
        config
            .extra
            .get("custom_root_key")
            .unwrap()
            .as_str()
            .unwrap(),
        "active"
    );

    assert!(config.mcp_servers.contains_key("codegraph"));
    let codegraph = &config.mcp_servers["codegraph"];
    assert_eq!(codegraph.command.as_deref(), Some("codegraph"));
    assert_eq!(codegraph.args, vec!["mcp"]);
    assert_eq!(codegraph.env.get("KEY").unwrap(), "VAL");

    assert!(config.mcp_servers.contains_key("remote_server"));
    let remote = &config.mcp_servers["remote_server"];
    assert_eq!(
        remote.server_url.as_deref(),
        Some("https://mcp.example.com/sse")
    );

    unregister_agy_mcp_server(&config_path, "codegraph").unwrap();

    let after_unreg = std::fs::read_to_string(&config_path).unwrap();
    let config_after: AgyMcpConfig = serde_json::from_str(&after_unreg).unwrap();
    assert!(!config_after.mcp_servers.contains_key("codegraph"));
    assert!(config_after.mcp_servers.contains_key("remote_server"));
}

#[test]
fn register_agy_mcp_server_resets_server_url_on_name_collision() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp_config.json");

    // Seed with codegraph entry containing serverUrl and url alias
    let initial_json = r#"{
      "mcpServers": {
        "codegraph": {
          "url": "https://mcp.example.com/codegraph",
          "headers": { "Auth": "Bearer token" }
        },
        "other_remote": {
          "serverUrl": "https://mcp.example.com/other"
        }
      }
    }"#;
    std::fs::write(&config_path, initial_json).unwrap();

    let env = BTreeMap::new();
    register_agy_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(json_val
        .pointer("/mcpServers/codegraph/serverUrl")
        .is_none());
    assert!(json_val.pointer("/mcpServers/codegraph/url").is_none());
    assert!(json_val.pointer("/mcpServers/codegraph/headers").is_none());

    let config: AgyMcpConfig = serde_json::from_str(&content).unwrap();

    let codegraph = &config.mcp_servers["codegraph"];
    assert_eq!(codegraph.server_url, None);
    assert_eq!(codegraph.command.as_deref(), Some("codegraph"));

    let other = &config.mcp_servers["other_remote"];
    assert_eq!(
        other.server_url.as_deref(),
        Some("https://mcp.example.com/other")
    );
}

#[test]
fn register_agy_mcp_server_excludes_opencode_keys() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp_config.json");
    let env = BTreeMap::new();

    register_agy_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(json_val.get("plugin").is_none());
    assert!(json_val.get("skills").is_none());
    assert!(!content.contains("plugin"));
    assert!(!content.contains("skills"));
}
