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

// ---------------------------------------------------------------------------
// Native plugin manager divergence & orphan managed tree probes.
// All fixtures are hermetic (tempfile); the real ~/.kimi-code is never read.
// ---------------------------------------------------------------------------

fn write_native_installed(kimi_dir: &Path, plugins: serde_json::Value) -> PathBuf {
    let plugins_dir = kimi_dir.join("plugins");
    std::fs::create_dir_all(&plugins_dir).unwrap();
    let file = plugins_dir.join("installed.json");
    std::fs::write(
        &file,
        serde_json::json!({ "version": 1, "plugins": plugins }).to_string(),
    )
    .unwrap();
    file
}

fn state_with_kimi(version: &str) -> crate::state::state::State {
    let mut state = crate::state::state::State::default();
    state.installed_harnesses.push(serde_json::json!({
        "name": "kimi",
        "scope": "global",
        "version": version,
    }));
    state
}

#[test]
fn test_kimi_marketplace_divergence_missing_or_malformed_file() {
    let tmp = TempDir::new().unwrap();
    let kimi_dir = tmp.path().join(".kimi-code");
    let cwd = tmp.path().join("cwd");
    let state = state_with_kimi("compound-engineering-v3.24.0");

    // 1. Missing plugins directory -> empty
    let divs = check_kimi_marketplace_divergence(&state, &cwd, &kimi_dir);
    assert!(divs.is_empty());

    // 2. Malformed JSON -> empty
    let file = write_native_installed(&kimi_dir, serde_json::json!([]));
    std::fs::write(&file, "{ corrupted json...").unwrap();
    let divs = check_kimi_marketplace_divergence(&state, &cwd, &kimi_dir);
    assert!(divs.is_empty());
}

#[test]
fn test_kimi_marketplace_divergence_harness_not_installed() {
    let tmp = TempDir::new().unwrap();
    let kimi_dir = tmp.path().join(".kimi-code");
    let cwd = tmp.path().join("cwd");

    let root = kimi_dir.join("plugins/managed/compound-engineering");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"compound-engineering","version":"3.14.3"}"#,
    )
    .unwrap();
    write_native_installed(
        &kimi_dir,
        serde_json::json!([
            { "id": "compound-engineering", "root": root, "enabled": true }
        ]),
    );

    // State has opencode, but NOT kimi -> no divergence
    let mut state = crate::state::state::State::default();
    state.installed_harnesses.push(serde_json::json!({
        "name": "opencode",
        "scope": "global",
        "version": "compound-engineering-v3.24.0",
    }));
    let divs = check_kimi_marketplace_divergence(&state, &cwd, &kimi_dir);
    assert!(divs.is_empty());
}

#[test]
fn test_kimi_marketplace_divergence_enabled_plugin_detected() {
    let tmp = TempDir::new().unwrap();
    let kimi_dir = tmp.path().join(".kimi-code");
    let cwd = tmp.path().join("cwd");

    let root = kimi_dir.join("plugins/managed/compound-engineering");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"compound-engineering","version":"3.14.3"}"#,
    )
    .unwrap();
    write_native_installed(
        &kimi_dir,
        serde_json::json!([
            { "id": "kimi-datasource", "root": "/other/root", "enabled": true },
            { "id": "compound-engineering", "root": root, "enabled": true }
        ]),
    );

    let state = state_with_kimi("compound-engineering-v3.24.0");
    let divs = check_kimi_marketplace_divergence(&state, &cwd, &kimi_dir);
    assert_eq!(divs.len(), 1);
    assert_eq!(divs[0].plugin_id, "compound-engineering");
    assert_eq!(divs[0].native_version, "3.14.3");
    assert_eq!(divs[0].ce_version, "compound-engineering-v3.24.0");
}

#[test]
fn test_kimi_marketplace_divergence_disabled_and_equal_versions_ignored() {
    let tmp = TempDir::new().unwrap();
    let kimi_dir = tmp.path().join(".kimi-code");
    let cwd = tmp.path().join("cwd");

    let disabled_root = kimi_dir.join("plugins/managed/compound-engineering");
    std::fs::create_dir_all(&disabled_root).unwrap();
    std::fs::write(
        disabled_root.join("package.json"),
        r#"{"name":"compound-engineering","version":"3.14.3"}"#,
    )
    .unwrap();

    let equal_root = tmp.path().join("equal-root");
    std::fs::create_dir_all(&equal_root).unwrap();
    // Version fallback: only plugin.json present, equal after normalization.
    std::fs::write(equal_root.join("plugin.json"), r#"{"version":"v3.24.0"}"#).unwrap();

    write_native_installed(
        &kimi_dir,
        serde_json::json!([
            { "id": "compound-engineering", "root": disabled_root, "enabled": false },
            { "id": "compound-engineering", "root": equal_root, "enabled": true }
        ]),
    );

    let state = state_with_kimi("compound-engineering-v3.24.0");
    let divs = check_kimi_marketplace_divergence(&state, &cwd, &kimi_dir);
    assert!(
        divs.is_empty(),
        "disabled entry and normalized-equal entry must not diverge: {divs:?}"
    );
}

#[test]
fn test_kimi_marketplace_divergence_unresolvable_native_version_skipped() {
    let tmp = TempDir::new().unwrap();
    let kimi_dir = tmp.path().join(".kimi-code");
    let cwd = tmp.path().join("cwd");

    let root = kimi_dir.join("plugins/managed/compound-engineering");
    std::fs::create_dir_all(&root).unwrap(); // no package.json / plugin.json
    write_native_installed(
        &kimi_dir,
        serde_json::json!([
            { "id": "compound-engineering", "root": root, "enabled": true }
        ]),
    );

    let state = state_with_kimi("compound-engineering-v3.24.0");
    let divs = check_kimi_marketplace_divergence(&state, &cwd, &kimi_dir);
    assert!(divs.is_empty());
}

#[test]
fn test_kimi_marketplace_divergence_workspace_scope_guard() {
    let tmp = TempDir::new().unwrap();
    let kimi_dir = tmp.path().join(".kimi-code");
    let other_project = tmp.path().join("other-project");
    let cwd = tmp.path().join("cwd");

    let root = kimi_dir.join("plugins/managed/compound-engineering");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("package.json"), r#"{"version":"3.14.3"}"#).unwrap();
    write_native_installed(
        &kimi_dir,
        serde_json::json!([
            { "id": "compound-engineering", "root": root, "enabled": true }
        ]),
    );

    // workspace-scoped kimi entry targeting another project must not apply
    let mut state = crate::state::state::State::default();
    state.installed_harnesses.push(serde_json::json!({
        "name": "kimi",
        "scope": "workspace",
        "target_dir": other_project,
        "version": "compound-engineering-v3.24.0",
    }));
    let divs = check_kimi_marketplace_divergence(&state, &cwd, &kimi_dir);
    assert!(divs.is_empty());

    // workspace-scoped entry encompassing cwd applies
    let workspace = tmp.path().join("workspace");
    let cwd_in_ws = workspace.join("src");
    std::fs::create_dir_all(&cwd_in_ws).unwrap();
    state.installed_harnesses[0]["target_dir"] = serde_json::json!(workspace);
    let divs = check_kimi_marketplace_divergence(&state, &cwd_in_ws, &kimi_dir);
    assert_eq!(divs.len(), 1);
}

#[test]
fn test_kimi_orphan_managed_tree_matrix() {
    let tmp = TempDir::new().unwrap();
    let kimi_dir = tmp.path().join(".kimi-code");
    let managed_dir = kimi_dir.join("compound-engineering");

    // 1. No managed tree -> None
    assert!(check_kimi_orphan_managed_tree(&kimi_dir).is_none());

    // 2. Managed tree without the ce-ai manifest marker -> None (not a ce-ai tree)
    std::fs::create_dir_all(&managed_dir).unwrap();
    assert!(check_kimi_orphan_managed_tree(&kimi_dir).is_none());

    // 3. ce-ai managed tree + missing config.toml -> orphan
    std::fs::write(managed_dir.join("install-manifest.json"), "{}").unwrap();
    let orphan = check_kimi_orphan_managed_tree(&kimi_dir).unwrap();
    assert_eq!(orphan.managed_dir, managed_dir);

    // 4. config.toml with empty extra_skill_dirs -> orphan
    std::fs::write(kimi_dir.join("config.toml"), "extra_skill_dirs = []\n").unwrap();
    assert!(check_kimi_orphan_managed_tree(&kimi_dir).is_some());

    // 5. config.toml referencing the managed dir -> not orphan
    std::fs::write(
        kimi_dir.join("config.toml"),
        format!("extra_skill_dirs = [\"{}\"]\n", managed_dir.display()),
    )
    .unwrap();
    assert!(check_kimi_orphan_managed_tree(&kimi_dir).is_none());

    // 6. Malformed config.toml degrades to orphan warning (not referenced)
    std::fs::write(kimi_dir.join("config.toml"), "not = [valid toml").unwrap();
    assert!(check_kimi_orphan_managed_tree(&kimi_dir).is_some());
}
