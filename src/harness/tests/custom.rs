use super::*;
use tempfile::TempDir;

#[test]
fn custom_adapter_default_paths_use_single_contract() {
    let home = PathBuf::from("/tmp/home");
    let adapter = CustomAdapter::new(None);
    assert_eq!(adapter.kind(), HarnessKind::Custom);
    assert_eq!(
        adapter.default_config_path(&home),
        home.join(".ce-ai").join(CONFIG_FILE_NAME)
    );
    assert!(adapter.config().is_none());
}

#[test]
fn resolve_prefers_flags_over_config_file() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let cfg_path = CustomHarnessConfig::config_path(home);
    std::fs::create_dir_all(cfg_path.parent().unwrap()).unwrap();
    std::fs::write(
        &cfg_path,
        r#"{"plugins_dir": "~/file-plugins", "skills_dir": "/abs/file-skills"}"#,
    )
    .unwrap();

    let cfg = CustomHarnessConfig::resolve(
        home,
        &CustomConfigFlags {
            plugins_dir: Some(PathBuf::from("~/flag-plugins")),
            ..Default::default()
        },
    )
    .unwrap();

    assert_eq!(cfg.plugins_dir, home.join("flag-plugins"));
    assert_eq!(cfg.skills_dir, PathBuf::from("/abs/file-skills"));
    assert_eq!(cfg.rules_file, None);
}

#[test]
fn resolve_falls_back_to_config_file_and_expands_tilde() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let cfg_path = CustomHarnessConfig::config_path(home);
    std::fs::create_dir_all(cfg_path.parent().unwrap()).unwrap();
    std::fs::write(
        &cfg_path,
        r#"{"plugins_dir": "~/p", "skills_dir": "rel-skills", "rules_file": "~/r.md"}"#,
    )
    .unwrap();

    let cfg = CustomHarnessConfig::resolve(home, &CustomConfigFlags::default()).unwrap();
    assert_eq!(cfg.plugins_dir, home.join("p"));
    assert_eq!(cfg.rules_file, Some(home.join("r.md")));
    assert!(cfg.skills_dir.is_absolute());
}

#[test]
fn resolve_without_any_configuration_is_a_usage_error() {
    let tmp = TempDir::new().unwrap();
    let err = CustomHarnessConfig::resolve(tmp.path(), &CustomConfigFlags::default()).unwrap_err();
    assert!(matches!(err, CeError::Usage(_)));
    assert_eq!(err.exit_code(), 2);
}

#[test]
fn resolve_preserves_rooted_paths_without_cwd_joining() {
    let tmp = TempDir::new().unwrap();
    let cfg = CustomHarnessConfig::resolve(
        tmp.path(),
        &CustomConfigFlags {
            plugins_dir: Some(PathBuf::from("/rooted/plugins")),
            skills_dir: Some(PathBuf::from("/rooted/skills")),
            rules_file: None,
            mcp_file: None,
        },
    )
    .unwrap();
    // A rooted path without a drive letter (Windows) must never be
    // joined onto the CWD.
    assert_eq!(cfg.plugins_dir, PathBuf::from("/rooted/plugins"));
    assert_eq!(cfg.skills_dir, PathBuf::from("/rooted/skills"));
}

#[test]
fn load_from_home_rejects_malformed_json_as_runtime_error() {
    let tmp = TempDir::new().unwrap();
    let cfg_path = CustomHarnessConfig::config_path(tmp.path());
    std::fs::create_dir_all(cfg_path.parent().unwrap()).unwrap();
    std::fs::write(&cfg_path, "{not json").unwrap();

    let err = CustomHarnessConfig::load_from_home(tmp.path()).unwrap_err();
    assert!(matches!(err, CeError::Runtime(_)));
}

#[test]
fn managed_rel_mappers_split_by_prefix() {
    assert_eq!(plugin_rel("plugins/loader.js"), Some("loader.js"));
    assert_eq!(
        skill_rel("skills/ce-work/SKILL.md"),
        Some("ce-work/SKILL.md")
    );
    assert_eq!(plugin_rel("skills/ce-work/SKILL.md"), None);
    assert_eq!(skill_rel("README.md"), None);
}

#[test]
fn state_json_round_trips_resolved_config() {
    let cfg = CustomHarnessConfig {
        plugins_dir: PathBuf::from("/p"),
        skills_dir: PathBuf::from("/s"),
        rules_file: Some(PathBuf::from("/r.md")),
        mcp_file: Some(PathBuf::from("/mcp.json")),
    };
    let parsed = CustomHarnessConfig::from_state_json(&cfg.to_state_json()).unwrap();
    assert_eq!(parsed, cfg);
}

#[test]
fn from_state_json_requires_both_directories() {
    assert!(CustomHarnessConfig::from_state_json(&serde_json::json!({
        "plugins_dir": "/p"
    }))
    .is_none());
}

#[test]
fn ensure_rules_block_creates_appends_and_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let rules = tmp.path().join("nested").join("rules.md");

    assert!(ensure_rules_block(&rules).unwrap());
    let first = std::fs::read_to_string(&rules).unwrap();
    assert!(first.contains(BLOCK_BEGIN_MARKER));
    assert!(first.contains(BLOCK_END_MARKER));

    assert!(!ensure_rules_block(&rules).unwrap());
    assert_eq!(std::fs::read_to_string(&rules).unwrap(), first);
}

#[test]
fn ensure_rules_block_preserves_user_content_around_the_block() {
    let tmp = TempDir::new().unwrap();
    let rules = tmp.path().join("rules.md");
    std::fs::write(&rules, "# my rules\nbe excellent\n").unwrap();

    ensure_rules_block(&rules).unwrap();
    let with_block = std::fs::read_to_string(&rules).unwrap();
    assert!(with_block.starts_with("# my rules\nbe excellent\n"));

    // Re-running replaces the block in place, keeping surrounding bytes.
    ensure_rules_block(&rules).unwrap();
    let again = std::fs::read_to_string(&rules).unwrap();
    assert!(again.starts_with("# my rules\nbe excellent\n"));
}

#[test]
fn strip_rules_block_removes_only_the_block() {
    let tmp = TempDir::new().unwrap();
    let rules = tmp.path().join("rules.md");
    std::fs::write(&rules, "# my rules\nbe excellent\n").unwrap();

    ensure_rules_block(&rules).unwrap();
    assert!(strip_rules_block(&rules).unwrap());
    assert_eq!(
        std::fs::read_to_string(&rules).unwrap(),
        "# my rules\nbe excellent\n"
    );
    assert!(!strip_rules_block(&rules).unwrap());

    let bare = tmp.path().join("bare.md");
    std::fs::write(&bare, "only block incoming\n").unwrap();
    ensure_rules_block(&bare).unwrap();
    assert!(strip_rules_block(&bare).unwrap());
    // User bytes survive verbatim; only the managed block disappears.
    assert_eq!(
        std::fs::read_to_string(&bare).unwrap(),
        "only block incoming\n"
    );
}

#[test]
fn strip_rules_block_errors_on_malformed_block() {
    let tmp = TempDir::new().unwrap();
    let rules = tmp.path().join("broken.md");
    std::fs::write(&rules, "<!-- ce-ai:block begin v=1 tier=full -->\nno end").unwrap();

    let err = strip_rules_block(&rules).unwrap_err();
    assert!(matches!(err, CeError::Runtime(_)));
}

#[test]
fn prune_empty_dirs_stops_at_boundaries() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().join("root");
    let deep = root.join("a").join("b");
    std::fs::create_dir_all(&deep).unwrap();

    prune_empty_dirs(&deep, &[&root]);
    assert!(root.exists());
    assert!(!root.join("a").exists());
}

#[test]
fn resolve_supports_mcp_file_via_flag_and_config() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let cfg_path = CustomHarnessConfig::config_path(home);
    std::fs::create_dir_all(cfg_path.parent().unwrap()).unwrap();
    std::fs::write(
        &cfg_path,
        r#"{"plugins_dir": "~/p", "skills_dir": "~/s", "mcp_file": "~/config.json"}"#,
    )
    .unwrap();

    let cfg = CustomHarnessConfig::resolve(home, &CustomConfigFlags::default()).unwrap();
    assert_eq!(cfg.mcp_file, Some(home.join("config.json")));

    let cfg_override = CustomHarnessConfig::resolve(
        home,
        &CustomConfigFlags {
            plugins_dir: Some(home.join("p")),
            skills_dir: Some(home.join("s")),
            mcp_file: Some(PathBuf::from("/custom/mcp.json")),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        cfg_override.mcp_file,
        Some(PathBuf::from("/custom/mcp.json"))
    );
}

#[test]
fn register_and_unregister_custom_mcp_server() {
    let tmp = TempDir::new().unwrap();
    let mcp_path = tmp.path().join("mcp.json");

    let env = std::collections::BTreeMap::new();
    register_custom_mcp_server(&mcp_path, "tool1", "tool1-cmd", &["arg1"], &env).unwrap();

    let content = std::fs::read_to_string(&mcp_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(
        json["mcpServers"]["tool1"],
        serde_json::json!({
            "command": "tool1-cmd",
            "args": ["arg1"],
            "env": {}
        })
    );

    register_companions(&mcp_path).unwrap();
    let content2 = std::fs::read_to_string(&mcp_path).unwrap();
    let json2: serde_json::Value = serde_json::from_str(&content2).unwrap();
    assert_eq!(
        json2["mcpServers"]["codegraph"],
        serde_json::json!({
            "command": "codegraph",
            "args": ["mcp"],
            "env": {}
        })
    );
    assert_eq!(
        json2["mcpServers"]["engram"],
        serde_json::json!({
            "command": "engram",
            "args": ["serve"],
            "env": {}
        })
    );
    assert_eq!(
        json2["mcpServers"]["tool1"],
        serde_json::json!({
            "command": "tool1-cmd",
            "args": ["arg1"],
            "env": {}
        })
    );

    assert!(unregister_custom_mcp_server(&mcp_path, "codegraph").unwrap());
    let content3 = std::fs::read_to_string(&mcp_path).unwrap();
    let json3: serde_json::Value = serde_json::from_str(&content3).unwrap();
    assert!(json3["mcpServers"].get("codegraph").is_none());
    assert!(json3["mcpServers"].get("engram").is_some());
    assert!(json3["mcpServers"].get("tool1").is_some());

    assert!(!unregister_custom_mcp_server(&mcp_path, "codegraph").unwrap());
}

#[test]
fn register_and_unregister_custom_mcp_server_fails_on_malformed_json() {
    let tmp = TempDir::new().unwrap();
    let mcp_path = tmp.path().join("corrupt.json");
    std::fs::write(&mcp_path, "{ not valid json").unwrap();

    let env = std::collections::BTreeMap::new();
    let reg_err = register_custom_mcp_server(&mcp_path, "tool", "cmd", &[], &env).unwrap_err();
    assert!(matches!(reg_err, CeError::Runtime(_)));

    let unreg_err = unregister_custom_mcp_server(&mcp_path, "tool").unwrap_err();
    assert!(matches!(unreg_err, CeError::Runtime(_)));
}

#[test]
fn register_and_unregister_custom_mcp_server_fails_when_not_an_object() {
    let tmp = TempDir::new().unwrap();
    let mcp_path = tmp.path().join("not_obj.json");
    std::fs::write(&mcp_path, "[1, 2, 3]").unwrap();

    let env = std::collections::BTreeMap::new();
    let reg_err = register_custom_mcp_server(&mcp_path, "tool", "cmd", &[], &env).unwrap_err();
    assert!(matches!(reg_err, CeError::Runtime(_)));

    let unreg_err = unregister_custom_mcp_server(&mcp_path, "tool").unwrap_err();
    assert!(matches!(unreg_err, CeError::Runtime(_)));
}

#[test]
fn unregister_companions_removes_both_and_preserves_other_servers() {
    let tmp = TempDir::new().unwrap();
    let mcp_path = tmp.path().join("mcp.json");

    let env = std::collections::BTreeMap::new();
    register_custom_mcp_server(&mcp_path, "user-tool", "cmd", &[], &env).unwrap();
    register_companions(&mcp_path).unwrap();

    unregister_companions(&mcp_path).unwrap();

    let content = std::fs::read_to_string(&mcp_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json["mcpServers"].get("codegraph").is_none());
    assert!(json["mcpServers"].get("engram").is_none());
    assert!(json["mcpServers"].get("user-tool").is_some());
}

#[test]
fn save_and_load_from_home_round_trips_custom_config_including_mcp_file() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path();
    let cfg = CustomHarnessConfig {
        plugins_dir: home.join("plugins"),
        skills_dir: home.join("skills"),
        rules_file: Some(home.join("rules.md")),
        mcp_file: Some(home.join("mcp.json")),
    };

    cfg.save(home).unwrap();

    let loaded = CustomHarnessConfig::load_from_home(home)
        .unwrap()
        .expect("config should exist");
    assert_eq!(loaded, cfg);

    // Re-saving with updated mcp_file updates custom_harness.json cleanly
    let mut cfg2 = cfg.clone();
    cfg2.mcp_file = Some(home.join("mcp_updated.json"));
    cfg2.save(home).unwrap();

    let loaded2 = CustomHarnessConfig::load_from_home(home)
        .unwrap()
        .expect("config should exist");
    assert_eq!(loaded2, cfg2);
}
