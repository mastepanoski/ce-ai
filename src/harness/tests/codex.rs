use super::*;
use tempfile::TempDir;

use crate::harness::tests::HARNESS_ENV_LOCK;

#[test]
fn codex_adapter_default_paths() {
    let _guard = HARNESS_ENV_LOCK.lock().unwrap();
    std::env::remove_var("CODEX_HOME");
    let adapter = CodexAdapter;
    assert_eq!(adapter.kind(), HarnessKind::Codex);
    let home = PathBuf::from("/tmp/home");
    assert_eq!(
        adapter.default_config_path(&home),
        PathBuf::from("/tmp/home/.codex/config.toml")
    );
}

#[test]
fn codex_adapter_respects_codex_home_env() {
    let _guard = HARNESS_ENV_LOCK.lock().unwrap();
    let adapter = CodexAdapter;
    let home = PathBuf::from("/tmp/home");
    std::env::set_var("CODEX_HOME", "/custom/codex/dir");
    let path = adapter.default_config_path(&home);
    std::env::remove_var("CODEX_HOME");
    assert_eq!(path, PathBuf::from("/custom/codex/dir/config.toml"));
}

#[test]
fn registers_and_unregisters_native_codex_mcp_server() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut env = BTreeMap::new();
    env.insert("LOG_LEVEL".to_string(), "info".to_string());

    register_codex_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let root: toml::Table = content.parse().unwrap();
    assert!(!root.contains_key("plugin"));
    assert!(!root.contains_key("skills"));

    let mcp = root["mcp_servers"].as_table().unwrap();
    let codegraph = mcp["codegraph"].as_table().unwrap();
    assert_eq!(codegraph["command"].as_str(), Some("codegraph"));
    assert_eq!(codegraph["env"]["LOG_LEVEL"].as_str(), Some("info"));

    unregister_codex_mcp_server(&config_path, "codegraph").unwrap();
    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let root_after: toml::Table = content_after.parse().unwrap();
    if let Some(mcp_after) = root_after.get("mcp_servers").and_then(|v| v.as_table()) {
        assert!(!mcp_after.contains_key("codegraph"));
    }
}

#[test]
fn preserves_existing_user_codex_tables_and_extra_fields() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let initial_toml = r#"model = "gpt-4o"

[mcp_servers.engram]
command = "engram"
args = ["serve"]
enabled = true
"#;
    std::fs::write(&config_path, initial_toml).unwrap();

    let env = BTreeMap::new();
    register_codex_mcp_server(
        &config_path,
        "engram",
        "engram",
        &["serve", "--debug"],
        &env,
    )
    .unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let root: toml::Table = content.parse().unwrap();
    assert_eq!(root["model"].as_str().unwrap(), "gpt-4o");
    let mcp = root["mcp_servers"].as_table().unwrap();
    let engram_table = mcp["engram"].as_table().unwrap();
    assert!(engram_table["enabled"].as_bool().unwrap());
    assert_eq!(engram_table["command"].as_str().unwrap(), "engram");

    unregister_codex_mcp_server(&config_path, "engram").unwrap();

    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let root_after: toml::Table = content_after.parse().unwrap();
    assert_eq!(root_after["model"].as_str().unwrap(), "gpt-4o");
}

#[test]
fn updates_and_strips_codex_agents_md_managed_block() {
    let tmp = TempDir::new().unwrap();
    let md_path = tmp.path().join("AGENTS.md");

    let user_header = "# My Project\n";
    std::fs::write(&md_path, user_header).unwrap();

    update_codex_agents_md(&md_path, "Directives content").unwrap();

    let content = std::fs::read_to_string(&md_path).unwrap();
    assert!(content.starts_with("# My Project"));
    assert!(content.contains(CE_MANAGED_BEGIN));
    assert!(content.contains("Directives content"));
    assert!(content.contains(CE_MANAGED_END));

    let stripped = strip_managed_block(&content);
    assert!(!stripped.contains(CE_MANAGED_BEGIN));
    assert_eq!(stripped.trim(), "# My Project");
}

#[test]
fn replaces_env_map_cleanly_on_re_registration() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut env1 = BTreeMap::new();
    env1.insert("OLD_KEY".to_string(), "old_val".to_string());
    register_codex_mcp_server(&config_path, "engram", "engram", &["serve"], &env1).unwrap();

    let mut env2 = BTreeMap::new();
    env2.insert("NEW_KEY".to_string(), "new_val".to_string());
    register_codex_mcp_server(&config_path, "engram", "engram", &["serve"], &env2).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let root: toml::Table = content.parse().unwrap();
    let engram_env = root["mcp_servers"]["engram"]["env"].as_table().unwrap();
    assert!(!engram_env.contains_key("OLD_KEY"));
    assert_eq!(engram_env["NEW_KEY"].as_str().unwrap(), "new_val");

    // Re-register with empty env map -> removes env table
    let empty_env = BTreeMap::new();
    register_codex_mcp_server(&config_path, "engram", "engram", &["serve"], &empty_env).unwrap();
    let content_empty = std::fs::read_to_string(&config_path).unwrap();
    let root_empty: toml::Table = content_empty.parse().unwrap();
    let engram_table = root_empty["mcp_servers"]["engram"].as_table().unwrap();
    assert!(!engram_table.contains_key("env"));
}

#[test]
fn codex_session_start_hook_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join(".codex/config.toml");

    assert!(!has_session_start_hook(&config_path));

    // Ensure hook in non-existent file
    let changed = ensure_session_start_hook(&config_path).unwrap();
    assert!(changed);
    assert!(has_session_start_hook(&config_path));

    // Verify content
    let content = std::fs::read_to_string(&config_path).unwrap();
    let root: toml::Table = content.parse().unwrap();
    let session_start = root["hooks"]["SessionStart"].as_array().unwrap();
    assert_eq!(session_start.len(), 1);
    let matcher = session_start[0]["matcher"].as_str().unwrap();
    assert_eq!(matcher, "startup|resume|compact");
    let hooks = session_start[0]["hooks"].as_array().unwrap();
    assert_eq!(hooks.len(), 1);
    assert_eq!(hooks[0]["command"].as_str().unwrap(), CODEX_RESUME_COMMAND);

    let stop = root["hooks"]["Stop"].as_array().unwrap();
    assert_eq!(stop.len(), 1);
    assert_eq!(
        stop[0]["hooks"][0]["command"].as_str().unwrap(),
        CODEX_RESUME_COMMAND
    );

    let pre_compact = root["hooks"]["PreCompact"].as_array().unwrap();
    assert_eq!(pre_compact.len(), 1);
    assert_eq!(
        pre_compact[0]["hooks"][0]["command"].as_str().unwrap(),
        CODEX_RESUME_COMMAND
    );

    // Idempotent second call
    let changed_second = ensure_session_start_hook(&config_path).unwrap();
    assert!(!changed_second);
    assert!(has_session_start_hook(&config_path));

    // Remove hook
    let removed = remove_session_start_hook(&config_path).unwrap();
    assert!(removed);
    assert!(!has_session_start_hook(&config_path));
    assert!(!config_path.exists(), "File should be removed when empty");

    let removed_second = remove_session_start_hook(&config_path).unwrap();
    assert!(!removed_second);
}

#[test]
fn codex_session_start_hook_preserves_user_settings() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join(".codex/config.toml");
    std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();

    let initial = r#"model = "o3-mini"

[[hooks.PreToolUse]]
matcher = "Bash"
[[hooks.PreToolUse.hooks]]
type = "command"
command = "policy_check.sh"

[[hooks.SessionStart]]
matcher = "startup"
[[hooks.SessionStart.hooks]]
type = "command"
command = "echo user-start"
"#;
    std::fs::write(&config_path, initial).unwrap();

    let changed = ensure_session_start_hook(&config_path).unwrap();
    assert!(changed);
    assert!(has_session_start_hook(&config_path));

    let content = std::fs::read_to_string(&config_path).unwrap();
    let root: toml::Table = content.parse().unwrap();
    assert_eq!(root["model"].as_str().unwrap(), "o3-mini");
    assert!(root["hooks"]["PreToolUse"].as_array().is_some());

    // Remove our hook
    let removed = remove_session_start_hook(&config_path).unwrap();
    assert!(removed);
    assert!(!has_session_start_hook(&config_path));
    assert!(
        config_path.exists(),
        "File should remain because user settings exist"
    );

    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let root_after: toml::Table = content_after.parse().unwrap();
    assert_eq!(root_after["model"].as_str().unwrap(), "o3-mini");
    assert!(root_after["hooks"]["PreToolUse"].as_array().is_some());
    let session_start = root_after["hooks"]["SessionStart"].as_array().unwrap();
    assert_eq!(session_start.len(), 1);
    assert_eq!(
        session_start[0]["hooks"].as_array().unwrap()[0]["command"]
            .as_str()
            .unwrap(),
        "echo user-start"
    );
}
