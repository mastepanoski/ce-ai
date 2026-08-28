use super::*;
use tempfile::TempDir;

#[test]
fn grok_adapter_default_paths() {
    let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
    std::env::remove_var("GROK_HOME");
    let adapter = GrokAdapter;
    assert_eq!(adapter.kind(), HarnessKind::Grok);
    let home = PathBuf::from("/tmp/home");
    assert_eq!(
        adapter.default_config_path(&home),
        PathBuf::from("/tmp/home/.grok/config.toml")
    );
}

#[test]
fn grok_adapter_respects_grok_home_env() {
    let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
    let adapter = GrokAdapter;
    let home = PathBuf::from("/tmp/home");
    std::env::set_var("GROK_HOME", "/custom/grok/dir");
    let path = adapter.default_config_path(&home);
    std::env::remove_var("GROK_HOME");
    assert_eq!(path, PathBuf::from("/custom/grok/dir/config.toml"));
}

#[test]
fn grok_adapter_config_path_edge_cases() {
    let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
    std::env::remove_var("GROK_HOME");
    let adapter = GrokAdapter;

    let config_direct = PathBuf::from("/tmp/home/config.toml");
    assert_eq!(adapter.default_config_path(&config_direct), config_direct);

    let grok_dir_direct = PathBuf::from("/tmp/home/.grok");
    assert_eq!(
        adapter.default_config_path(&grok_dir_direct),
        PathBuf::from("/tmp/home/.grok/config.toml")
    );
}

#[test]
fn register_grok_mcp_server_handles_invalid_toml() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");
    std::fs::write(&config_path, "invalid_toml = [unclosed").unwrap();

    let env = BTreeMap::new();
    let res = register_grok_mcp_server(&config_path, "engram", "engram", &["serve"], &env);
    assert!(res.is_err());
}

#[test]
fn registers_and_unregisters_native_grok_mcp_server() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut env = BTreeMap::new();
    env.insert("LOG_LEVEL".to_string(), "info".to_string());

    register_grok_mcp_server(&config_path, "codegraph", "codegraph", &["mcp"], &env).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let root: toml::Table = content.parse().unwrap();
    let mcp = root["mcp_servers"].as_table().unwrap();
    assert!(mcp.contains_key("codegraph"));

    let codegraph = mcp["codegraph"].as_table().unwrap();
    assert_eq!(codegraph["command"].as_str().unwrap(), "codegraph");
    assert_eq!(
        codegraph["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["mcp"]
    );
    assert!(!root.contains_key("plugin"), "Zero OpenCode key leaks");
    assert!(!root.contains_key("skills"), "Zero OpenCode key leaks");

    unregister_grok_mcp_server(&config_path, "codegraph").unwrap();
    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let root_after: toml::Table = content_after.parse().unwrap();
    let mcp_after = root_after["mcp_servers"].as_table().unwrap();
    assert!(!mcp_after.contains_key("codegraph"));
}

#[test]
fn preserves_existing_user_grok_tables_and_extra_fields() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let initial_toml = r#"
model = "grok-beta"

[mcp_servers.engram]
command = "engram"
args = ["serve"]
enabled = true
"#;
    std::fs::write(&config_path, initial_toml).unwrap();

    let env = BTreeMap::new();
    register_grok_mcp_server(
        &config_path,
        "engram",
        "engram",
        &["serve", "--debug"],
        &env,
    )
    .unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let root: toml::Table = content.parse().unwrap();
    assert_eq!(root["model"].as_str().unwrap(), "grok-beta");
    let mcp = root["mcp_servers"].as_table().unwrap();
    let engram_table = mcp["engram"].as_table().unwrap();
    assert!(engram_table["enabled"].as_bool().unwrap());
    assert_eq!(engram_table["command"].as_str().unwrap(), "engram");

    unregister_grok_mcp_server(&config_path, "engram").unwrap();

    let content_after = std::fs::read_to_string(&config_path).unwrap();
    let root_after: toml::Table = content_after.parse().unwrap();
    assert_eq!(root_after["model"].as_str().unwrap(), "grok-beta");
}

#[test]
fn updates_and_strips_grok_rule_md_managed_block() {
    let tmp = TempDir::new().unwrap();
    let md_path = tmp.path().join("compound-engineering.md");

    let user_header = "# My Grok Rules\n";
    std::fs::write(&md_path, user_header).unwrap();

    update_grok_rule_md(&md_path, "Directives content").unwrap();

    let content = std::fs::read_to_string(&md_path).unwrap();
    assert!(content.starts_with("# My Grok Rules"));
    assert!(content.contains(CE_MANAGED_BEGIN));
    assert!(content.contains("Directives content"));
    assert!(content.contains(CE_MANAGED_END));

    let stripped = strip_managed_block(&content);
    assert!(!stripped.contains(CE_MANAGED_BEGIN));
    assert_eq!(stripped.trim(), "# My Grok Rules");
}

#[test]
fn replaces_env_map_cleanly_on_re_registration() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let mut env1 = BTreeMap::new();
    env1.insert("OLD_KEY".to_string(), "old_val".to_string());
    register_grok_mcp_server(&config_path, "engram", "engram", &["serve"], &env1).unwrap();

    let mut env2 = BTreeMap::new();
    env2.insert("NEW_KEY".to_string(), "new_val".to_string());
    register_grok_mcp_server(&config_path, "engram", "engram", &["serve"], &env2).unwrap();

    let content = std::fs::read_to_string(&config_path).unwrap();
    let root: toml::Table = content.parse().unwrap();
    let engram_env = root["mcp_servers"]["engram"]["env"].as_table().unwrap();
    assert!(!engram_env.contains_key("OLD_KEY"));
    assert_eq!(engram_env["NEW_KEY"].as_str().unwrap(), "new_val");

    // Re-register with empty env map -> removes env table
    let empty_env = BTreeMap::new();
    register_grok_mcp_server(&config_path, "engram", "engram", &["serve"], &empty_env).unwrap();
    let content_empty = std::fs::read_to_string(&config_path).unwrap();
    let root_empty: toml::Table = content_empty.parse().unwrap();
    let engram_table = root_empty["mcp_servers"]["engram"].as_table().unwrap();
    assert!(!engram_table.contains_key("env"));
}
