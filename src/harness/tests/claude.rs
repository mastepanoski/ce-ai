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

#[test]
fn ensures_and_removes_session_start_hook_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let settings_path = tmp.path().join(".claude").join("settings.json");

    assert!(!has_session_start_hook(&settings_path));

    // Fresh injection
    let changed = ensure_session_start_hook(&settings_path).unwrap();
    assert!(changed);
    assert!(has_session_start_hook(&settings_path));

    // Idempotent re-run
    let changed_again = ensure_session_start_hook(&settings_path).unwrap();
    assert!(!changed_again);

    let content = std::fs::read_to_string(&settings_path).unwrap();
    let val: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(
        val["hooks"]["SessionStart"][0]["hooks"][0]["command"],
        "ce-ai workflow resume"
    );

    // Surgical removal cleans up empty file
    let removed = remove_session_start_hook(&settings_path).unwrap();
    assert!(removed);
    assert!(!settings_path.exists());
    assert!(!has_session_start_hook(&settings_path));
}

#[test]
fn preserves_user_hooks_and_settings_in_claude_settings_json() {
    let tmp = TempDir::new().unwrap();
    let settings_path = tmp.path().join("settings.json");

    let initial = serde_json::json!({
        "mcpServers": {
            "custom-tool": {
                "command": "node"
            }
        },
        "hooks": {
            "PreToolUse": [
                {
                    "matcher": ".*",
                    "hooks": [{"type": "command", "command": "echo pre"}]
                }
            ],
            "SessionStart": [
                {
                    "matcher": "git.*",
                    "hooks": [{"type": "command", "command": "echo user-hook"}]
                }
            ]
        }
    });

    std::fs::write(
        &settings_path,
        serde_json::to_string_pretty(&initial).unwrap(),
    )
    .unwrap();

    let changed = ensure_session_start_hook(&settings_path).unwrap();
    assert!(changed);
    assert!(has_session_start_hook(&settings_path));

    let content = std::fs::read_to_string(&settings_path).unwrap();
    let val: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(val.get("mcpServers").is_some());
    assert_eq!(
        val["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
        "echo pre"
    );

    // SessionStart now contains user hook + our hook
    let s_start = val["hooks"]["SessionStart"].as_array().unwrap();
    assert_eq!(s_start.len(), 2);

    // Remove our hook: user hook and mcpServers must be preserved
    let removed = remove_session_start_hook(&settings_path).unwrap();
    assert!(removed);
    assert!(!has_session_start_hook(&settings_path));
    assert!(settings_path.exists());

    let content_after = std::fs::read_to_string(&settings_path).unwrap();
    let val_after: serde_json::Value = serde_json::from_str(&content_after).unwrap();
    assert!(val_after.get("mcpServers").is_some());
    assert_eq!(
        val_after["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
        "echo pre"
    );
    assert_eq!(
        val_after["hooks"]["SessionStart"][0]["hooks"][0]["command"],
        "echo user-hook"
    );
}
