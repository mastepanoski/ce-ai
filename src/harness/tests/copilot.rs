use super::*;
use tempfile::TempDir;

#[test]
fn copilot_adapter_default_paths() {
    let adapter = CopilotAdapter;
    assert_eq!(adapter.kind(), HarnessKind::Copilot);
    let home = PathBuf::from("/tmp/home");
    assert_eq!(
        adapter.default_config_path(&home),
        PathBuf::from("/tmp/home/.copilot/mcp-config.json")
    );
}

#[test]
fn registers_and_unregisters_native_copilot_mcp_server() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp-config.json");

    let mut env = BTreeMap::new();
    env.insert("LOG_LEVEL".to_string(), "info".to_string());

    register_copilot_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: CopilotMcpConfig = serde_json::from_str(&content).unwrap();
    assert!(config.mcp_servers.contains_key("codegraph"));
    assert_eq!(config.mcp_servers["codegraph"].command, "codegraph");
    assert_eq!(config.mcp_servers["codegraph"].args, vec!["mcp"]);
    assert!(config.extra.is_empty(), "Zero OpenCode key leaks");

    unregister_copilot_mcp_server(&config_path, "codegraph").unwrap();
    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let config_after: CopilotMcpConfig = serde_json::from_str(&content_after).unwrap();
    assert!(!config_after.mcp_servers.contains_key("codegraph"));
}

#[test]
fn preserves_existing_user_copilot_keys_and_extra_fields() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp-config.json");

    let initial_json = r#"{
  "telemetry": false,
  "mcpServers": {
    "user-tool": {
      "command": "node",
      "args": ["server.js"]
    }
  }
}"#;
    std::fs::write(&config_path, initial_json).unwrap();

    let env = BTreeMap::new();
    register_copilot_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: CopilotMcpConfig = serde_json::from_str(&content).unwrap();
    assert_eq!(
        config.extra.get("telemetry").unwrap(),
        &serde_json::Value::Bool(false)
    );
    assert!(config.mcp_servers.contains_key("user-tool"));
    assert!(config.mcp_servers.contains_key("engram"));

    unregister_copilot_mcp_server(&config_path, "engram").unwrap();

    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let config_after: CopilotMcpConfig = serde_json::from_str(&content_after).unwrap();
    assert_eq!(
        config_after.extra.get("telemetry").unwrap(),
        &serde_json::Value::Bool(false)
    );
    assert!(config_after.mcp_servers.contains_key("user-tool"));
    assert!(!config_after.mcp_servers.contains_key("engram"));
}

#[test]
fn updates_and_strips_copilot_instructions_md_managed_block() {
    let tmp = TempDir::new().unwrap();
    let md_path = tmp.path().join("copilot-instructions.md");

    let user_header = "# My Project Notes\n";
    std::fs::write(&md_path, user_header).unwrap();

    update_copilot_instructions_md(&md_path, "Directives content").unwrap();

    let content = std::fs::read_to_string(&md_path).unwrap();
    assert!(content.starts_with("# My Project Notes"));
    assert!(content.contains(CE_MANAGED_BEGIN));
    assert!(content.contains("Directives content"));
    assert!(content.contains(CE_MANAGED_END));

    let stripped = strip_managed_block(&content);
    assert!(!stripped.contains(CE_MANAGED_BEGIN));
    assert_eq!(stripped.trim(), "# My Project Notes");
}

#[test]
fn register_copilot_mcp_server_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp-config.json");

    let env = BTreeMap::new();
    register_copilot_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();
    register_copilot_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: CopilotMcpConfig = serde_json::from_str(&content).unwrap();
    assert_eq!(config.mcp_servers.len(), 1);
    assert!(config.mcp_servers.contains_key("engram"));
}

#[test]
fn replaces_env_map_cleanly_on_re_registration() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp-config.json");

    let mut env1 = BTreeMap::new();
    env1.insert("OLD_KEY".to_string(), "old_val".to_string());
    register_copilot_mcp_server(&config_path, "engram", "engram", &["serve"], &env1).unwrap();

    let mut env2 = BTreeMap::new();
    env2.insert("NEW_KEY".to_string(), "new_val".to_string());
    register_copilot_mcp_server(&config_path, "engram", "engram", &["serve"], &env2).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: CopilotMcpConfig = serde_json::from_str(&content).unwrap();
    let engram = config.mcp_servers.get("engram").unwrap();
    assert!(!engram.env.contains_key("OLD_KEY"));
    assert_eq!(engram.env.get("NEW_KEY").unwrap(), "new_val");

    // Re-register with empty env map -> removes env key from JSON
    let empty_env = BTreeMap::new();
    register_copilot_mcp_server(&config_path, "engram", "engram", &["serve"], &empty_env).unwrap();
    let content_empty = std::fs::read_to_string(&config_path).unwrap();
    assert!(!content_empty.contains("\"env\""));

    // Confirm struct-level deserialization: env map is empty and command/args are preserved
    let config_empty: CopilotMcpConfig = serde_json::from_str(&content_empty).unwrap();
    let engram_server = config_empty.mcp_servers.get("engram").unwrap();
    assert!(engram_server.env.is_empty());
    assert_eq!(engram_server.command, "engram");
    assert_eq!(engram_server.args, vec!["serve"]);
}

#[test]
fn copilot_session_start_hook_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let hooks_path = tmp.path().join(".github/hooks/hooks.json");

    assert!(!has_session_start_hook(&hooks_path));

    // Ensure hook in non-existent file
    let changed = ensure_session_start_hook(&hooks_path).unwrap();
    assert!(changed);
    assert!(has_session_start_hook(&hooks_path));

    // Content check
    let content = std::fs::read_to_string(&hooks_path).unwrap();
    let val: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(val["version"], 1);
    let session_start = val["hooks"]["sessionStart"].as_array().unwrap();
    assert_eq!(session_start.len(), 1);
    assert_eq!(session_start[0]["type"], "command");
    assert_eq!(session_start[0]["bash"], COPILOT_RESUME_COMMAND);
    assert_eq!(session_start[0]["powershell"], COPILOT_RESUME_COMMAND);

    // Idempotent second call
    let changed_second = ensure_session_start_hook(&hooks_path).unwrap();
    assert!(!changed_second);
    assert!(has_session_start_hook(&hooks_path));

    // Remove hook
    let removed = remove_session_start_hook(&hooks_path).unwrap();
    assert!(removed);
    assert!(!has_session_start_hook(&hooks_path));
    assert!(!hooks_path.exists(), "File should be removed when empty");

    let removed_second = remove_session_start_hook(&hooks_path).unwrap();
    assert!(!removed_second);
}

#[test]
fn copilot_session_start_hook_preserves_user_settings() {
    let tmp = TempDir::new().unwrap();
    let hooks_path = tmp.path().join(".github/hooks/hooks.json");
    std::fs::create_dir_all(hooks_path.parent().unwrap()).unwrap();

    let initial = serde_json::json!({
        "version": 1,
        "hooks": {
            "preToolUse": [
                {
                    "type": "command",
                    "bash": "echo 'pre-tool'"
                }
            ],
            "sessionStart": [
                {
                    "type": "command",
                    "bash": "echo 'user-start'"
                }
            ]
        }
    });
    std::fs::write(&hooks_path, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

    let changed = ensure_session_start_hook(&hooks_path).unwrap();
    assert!(changed);
    assert!(has_session_start_hook(&hooks_path));

    let content = std::fs::read_to_string(&hooks_path).unwrap();
    let val: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(val["hooks"]["sessionStart"].as_array().unwrap().len(), 2);
    assert_eq!(val["hooks"]["preToolUse"].as_array().unwrap().len(), 1);

    // Remove our hook
    let removed = remove_session_start_hook(&hooks_path).unwrap();
    assert!(removed);
    assert!(!has_session_start_hook(&hooks_path));
    assert!(
        hooks_path.exists(),
        "File should remain because user hooks exist"
    );

    let content_after = std::fs::read_to_string(&hooks_path).unwrap();
    let val_after: serde_json::Value = serde_json::from_str(&content_after).unwrap();
    assert_eq!(
        val_after["hooks"]["sessionStart"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        val_after["hooks"]["sessionStart"][0]["bash"],
        "echo 'user-start'"
    );
    assert_eq!(
        val_after["hooks"]["preToolUse"].as_array().unwrap().len(),
        1
    );
}
