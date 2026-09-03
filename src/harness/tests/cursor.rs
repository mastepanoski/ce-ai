use super::*;
use tempfile::TempDir;

#[test]
fn registers_and_unregisters_native_cursor_mcp_server() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp.json");

    let mut env = BTreeMap::new();
    env.insert("LOG_LEVEL".to_string(), "info".to_string());

    register_cursor_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("\"mcpServers\""));
    assert!(content.contains("\"codegraph\""));
    assert!(content.contains("\"stdio\""));
    assert!(!content.contains("\"plugin\""));
    assert!(!content.contains("\"skills\""));

    let config: CursorMcpConfig = serde_json::from_str(&content).unwrap();
    assert_eq!(config.mcp_servers.len(), 1);
    assert_eq!(
        config.mcp_servers["codegraph"].r#type.as_deref(),
        Some("stdio")
    );

    unregister_cursor_mcp_server(&config_path, "codegraph").unwrap();
    assert!(!config_path.exists());
}

#[test]
fn preserves_existing_user_cursor_mcp_servers_and_per_server_extra_fields() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp.json");

    let initial_json = r#"{
  "mcpServers": {
    "user-tool": {
      "type": "stdio",
      "command": "node",
      "args": ["server.js"],
      "disabled": false,
      "timeout": 300
    },
    "sse-tool": {
      "type": "sse",
      "url": "https://mcp.example.com/sse"
    }
  },
  "userSetting": true
}"#;
    std::fs::write(&config_path, initial_json).unwrap();

    let env = BTreeMap::new();
    register_cursor_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: CursorMcpConfig = serde_json::from_str(&content).unwrap();

    assert_eq!(config.mcp_servers.len(), 3);
    assert!(config.mcp_servers.contains_key("user-tool"));
    assert!(config.mcp_servers.contains_key("sse-tool"));
    assert!(config.mcp_servers.contains_key("engram"));
    assert_eq!(
        config.mcp_servers["user-tool"].extra.get("disabled"),
        Some(&serde_json::Value::Bool(false))
    );
    assert_eq!(
        config.mcp_servers["sse-tool"].url,
        Some("https://mcp.example.com/sse".to_string())
    );

    unregister_cursor_mcp_server(&config_path, "engram").unwrap();

    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let config_after: CursorMcpConfig = serde_json::from_str(&content_after).unwrap();
    assert_eq!(config_after.mcp_servers.len(), 2);
    assert!(config_after.mcp_servers.contains_key("user-tool"));
    assert!(config_after.mcp_servers.contains_key("sse-tool"));
}

#[test]
fn updates_cursor_rule_mdc_with_frontmatter() {
    let tmp = TempDir::new().unwrap();
    let rule_path = tmp.path().join("compound-engineering.mdc");
    let frontmatter = CursorRuleFrontmatter::default();

    update_cursor_rule_mdc(&rule_path, &frontmatter, "Directives content").unwrap();

    let content = std::fs::read_to_string(&rule_path).unwrap();
    assert!(content.starts_with("---\n"));
    assert!(content.contains("description: Compound Engineering Agent Directives"));
    assert!(content.contains("globs: *"));
    assert!(content.contains("alwaysApply: true"));
    assert!(content.contains(CE_MANAGED_BEGIN));
    assert!(content.contains("Directives content"));
    assert!(content.contains(CE_MANAGED_END));
}

#[test]
fn handles_unbalanced_managed_markers_gracefully() {
    let unbalanced_start = format!("User content\n\n{}\nPartial block", CE_MANAGED_BEGIN);
    let updated = update_managed_block(&unbalanced_start, "Fresh block");
    assert!(updated.contains("User content"));
    assert!(updated.contains(CE_MANAGED_BEGIN));
    assert!(updated.contains("Fresh block"));
    assert!(updated.contains(CE_MANAGED_END));
}

#[test]
fn preserves_non_stdio_transport_type_on_re_registration() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("mcp.json");
    let initial_json = r#"{
  "mcpServers": {
    "engram": {
      "type": "sse",
      "command": "old",
      "url": "https://example.com/sse"
    }
  }
}"#;
    std::fs::write(&config_path, initial_json).unwrap();
    let env = BTreeMap::new();
    register_cursor_mcp_server(&config_path, "engram", "engram", &["serve"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let config: CursorMcpConfig = serde_json::from_str(&content).unwrap();
    assert_eq!(config.mcp_servers["engram"].r#type.as_deref(), Some("sse"));
    assert_eq!(config.mcp_servers["engram"].command, "engram");
}

#[test]
fn updates_existing_cursor_rule_mdc_without_duplicating_frontmatter() {
    let tmp = TempDir::new().unwrap();
    let rule_path = tmp.path().join("compound-engineering.mdc");
    let frontmatter = CursorRuleFrontmatter::default();

    let initial_text = r#"---
description: Custom Rule
globs: *.rs
alwaysApply: false
---
# User Custom Instructions
Keep types strict.
"#;
    std::fs::write(&rule_path, initial_text).unwrap();

    update_cursor_rule_mdc(&rule_path, &frontmatter, "Updated managed text").unwrap();

    let content = std::fs::read_to_string(&rule_path).unwrap();
    let frontmatter_count = content.matches("---").count();
    assert_eq!(
        frontmatter_count, 2,
        "Frontmatter delimiters must not be duplicated"
    );
    assert!(content.contains("# User Custom Instructions"));
    assert!(content.contains("Keep types strict."));
    assert!(content.contains("Updated managed text"));
}

#[test]
fn cursor_session_start_hook_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let hooks_path = tmp.path().join("hooks.json");

    assert!(!has_session_start_hook(&hooks_path));

    // First ensure -> creates file and returns Ok(true)
    let created = ensure_session_start_hook(&hooks_path).unwrap();
    assert!(created);
    assert!(has_session_start_hook(&hooks_path));

    let content = std::fs::read_to_string(&hooks_path).unwrap();
    let val: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(val["version"], 1);
    let session_start = val["hooks"]["sessionStart"].as_array().unwrap();
    assert_eq!(session_start.len(), 1);
    assert_eq!(session_start[0]["command"], CURSOR_RESUME_COMMAND);

    // Second ensure -> idempotent, returns Ok(false)
    let re_ensure = ensure_session_start_hook(&hooks_path).unwrap();
    assert!(!re_ensure);
    assert!(has_session_start_hook(&hooks_path));

    // Remove -> returns Ok(true) and prunes file
    let removed = remove_session_start_hook(&hooks_path).unwrap();
    assert!(removed);
    assert!(!has_session_start_hook(&hooks_path));
    assert!(!hooks_path.exists());

    // Second remove -> returns Ok(false)
    let re_remove = remove_session_start_hook(&hooks_path).unwrap();
    assert!(!re_remove);
}

#[test]
fn cursor_session_start_hook_preserves_user_hooks() {
    let tmp = TempDir::new().unwrap();
    let hooks_path = tmp.path().join("hooks.json");

    let initial = serde_json::json!({
        "version": 1,
        "hooks": {
            "preToolUse": [
                {
                    "command": "./validate.sh",
                    "matcher": "Shell|Write"
                }
            ],
            "sessionStart": [
                {
                    "command": "./custom-init.sh"
                }
            ]
        },
        "user_custom_setting": true
    });
    std::fs::write(&hooks_path, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

    let created = ensure_session_start_hook(&hooks_path).unwrap();
    assert!(created);
    assert!(has_session_start_hook(&hooks_path));

    let content = std::fs::read_to_string(&hooks_path).unwrap();
    let val: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(val["user_custom_setting"], true);
    assert_eq!(val["hooks"]["preToolUse"].as_array().unwrap().len(), 1);
    let session_start = val["hooks"]["sessionStart"].as_array().unwrap();
    assert_eq!(session_start.len(), 2);
    assert_eq!(session_start[0]["command"], "./custom-init.sh");
    assert_eq!(session_start[1]["command"], CURSOR_RESUME_COMMAND);

    let removed = remove_session_start_hook(&hooks_path).unwrap();
    assert!(removed);
    assert!(!has_session_start_hook(&hooks_path));
    assert!(hooks_path.exists());

    let content_after = std::fs::read_to_string(&hooks_path).unwrap();
    let val_after: serde_json::Value = serde_json::from_str(&content_after).unwrap();
    assert_eq!(val_after["user_custom_setting"], true);
    assert_eq!(
        val_after["hooks"]["preToolUse"].as_array().unwrap().len(),
        1
    );
    let session_start_after = val_after["hooks"]["sessionStart"].as_array().unwrap();
    assert_eq!(session_start_after.len(), 1);
    assert_eq!(session_start_after[0]["command"], "./custom-init.sh");
}
