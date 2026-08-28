use super::*;
use tempfile::TempDir;

#[test]
fn claude_adapter_default_paths() {
    let adapter = ClaudeAdapter;
    assert_eq!(adapter.kind(), HarnessKind::Claude);
    let home = PathBuf::from("/tmp/home");
    assert_eq!(
        adapter.default_config_path(&home),
        PathBuf::from("/tmp/home/.claude.json")
    );
}

#[test]
fn registers_and_unregisters_native_claude_mcp_server() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join(".claude.json");

    let mut env = BTreeMap::new();
    env.insert("LOG_LEVEL".to_string(), "info".to_string());

    register_claude_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("\"mcpServers\""));
    assert!(content.contains("\"codegraph\""));
    assert!(!content.contains("\"plugin\""));
    assert!(!content.contains("\"skills\""));

    let config: ClaudeMcpConfig = serde_json::from_str(&content).unwrap();
    assert_eq!(config.mcp_servers.len(), 1);

    unregister_claude_mcp_server(&config_path, "codegraph").unwrap();
    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let config_after: ClaudeMcpConfig = serde_json::from_str(&content_after).unwrap();
    assert!(config_after.mcp_servers.is_empty());
}

#[test]
fn preserves_existing_user_claude_mcp_servers_and_extra_fields() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join(".claude.json");

    let initial_json = r#"{
  "mcpServers": {
    "user-tool": {
      "command": "node",
      "args": ["server.js"],
      "disabled": false
    }
  },
  "numStartups": 42
}"#;
    std::fs::write(&config_path, initial_json).unwrap();

    let env = BTreeMap::new();
    register_claude_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: ClaudeMcpConfig = serde_json::from_str(&content).unwrap();

    assert_eq!(config.mcp_servers.len(), 2);
    assert!(config.mcp_servers.contains_key("user-tool"));
    assert!(config.mcp_servers.contains_key("engram"));
    assert_eq!(config.mcp_servers["user-tool"].r#type, None);
    assert_eq!(
        config.extra.get("numStartups"),
        Some(&serde_json::Value::Number(42.into()))
    );

    unregister_claude_mcp_server(&config_path, "engram").unwrap();

    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let config_after: ClaudeMcpConfig = serde_json::from_str(&content_after).unwrap();
    assert_eq!(config_after.mcp_servers.len(), 1);
    assert!(config_after.mcp_servers.contains_key("user-tool"));
}

#[test]
fn updates_and_strips_claude_md_managed_block() {
    let tmp = TempDir::new().unwrap();
    let md_path = tmp.path().join("CLAUDE.md");

    update_claude_md(&md_path, "Directives content").unwrap();

    let content = std::fs::read_to_string(&md_path).unwrap();
    assert!(content.contains(CE_MANAGED_BEGIN));
    assert!(content.contains("Directives content"));
    assert!(content.contains(CE_MANAGED_END));

    let stripped = strip_managed_block(&content);
    assert!(!stripped.contains(CE_MANAGED_BEGIN));
}
