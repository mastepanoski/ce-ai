//! CLI integration tests: install, status, uninstall (CC-1..CC-3, OI-1..OI-5, SU-4).
//! Every test pins ce-ai to hermetic temp dirs — never touches the real user config or HOME.

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn ceai(config_dir: &Path, home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ce-ai").unwrap();
    cmd.arg("--config-dir").arg(config_dir).env("HOME", home).env("CE_AI_OPENCODE_CONFIG", home.join(".config/opencode"));
    cmd
}

/// Local CE source-tree fixture: loader + one skill.
fn ce_source(dir: &Path) -> PathBuf {
    let loader = dir.join("ce-tree/.opencode/plugins/compound-engineering.js");
    fs::create_dir_all(loader.parent().unwrap()).unwrap();
    fs::write(&loader, "export default function ceLoader() {}\n").unwrap();
    let skill = dir.join("ce-tree/.opencode/skills/ce-brainstorm/SKILL.md");
    fs::create_dir_all(skill.parent().unwrap()).unwrap();
    fs::write(&skill, "# ce-brainstorm\n").unwrap();
    dir.join("ce-tree")
}

fn user_config(home: &Path, content: &str) {
    let path = home.join(".config/opencode/opencode.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn install(config_dir: &Path, home: &Path, source: &Path) {
    ceai(config_dir, home).args(["install", "--harness", "opencode", "--source"]).arg(source).assert().success();
}

#[test]
fn install_fresh_install_creates_backup_entry_loader_skills_and_manifest() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let user = r#"{"plugin":["user-plugin"],"skills":{"paths":["/home/user/skills"]}}"#;
    user_config(&home, user);
    install(&config_dir, &home, &source);

    // OI-1: pre-install config backed up (one backup dir).
    assert_eq!(fs::read_dir(config_dir.join("backups")).unwrap().count(), 1);
    // OI-2/OI-4: plugin entry + skills path merged without clobbering user config.
    let opencode_json = home.join(".config/opencode/opencode.json");
    let config = read_json(&opencode_json);
    let plugins = config["plugin"].as_array().unwrap();
    assert_eq!(plugins.len(), 2, "user plugin + CE loader");
    assert!(plugins.iter().any(|v| v == "user-plugin"));
    let loader_path = opencode_json.parent().unwrap().join("compound-engineering/plugins/compound-engineering.js");
    assert!(plugins.iter().any(|v| v.as_str() == Some(loader_path.to_str().unwrap())));
    assert_eq!(config["skills"]["paths"].as_array().unwrap().len(), 2, "user + CE skills path");
    // OI-3: loader copied into the managed plugins dir.
    assert_eq!(fs::read_to_string(&loader_path).unwrap(), "export default function ceLoader() {}\n");
    // OI-5: manifest lists every managed file and the config backup.
    let manifest = read_json(&opencode_json.parent().unwrap().join("compound-engineering/install-manifest.json"));
    let paths: Vec<&str> = manifest["files"].as_array().unwrap().iter().map(|f| f["path"].as_str().unwrap()).collect();
    assert_eq!(paths, vec!["plugins/compound-engineering.js", "skills/ce-brainstorm/SKILL.md"]);
    assert!(manifest["config_mutations"][0]["backup"].as_str().is_some());
}

#[test]
fn install_reinstall_is_idempotent_without_duplicates() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    user_config(&home, r#"{"plugin":["user-plugin"]}"#);

    let mut cmd = ceai(&config_dir, &home);
    cmd.args(["install", "--harness", "opencode", "--source"]).arg(&source);
    cmd.assert().success();
    cmd.assert().success();

    let config = read_json(&home.join(".config/opencode/opencode.json"));
    assert_eq!(config["plugin"].as_array().unwrap().len(), 2, "user plugin + one CE entry");
    assert_eq!(config["skills"]["paths"].as_array().unwrap().len(), 1, "no duplicate skills path");
    let state = read_json(&config_dir.join("state.json"));
    assert_eq!(state["installed_harnesses"].as_array().unwrap().len(), 1, "one state entry");
    assert_eq!(state["installed_harnesses"][0]["name"], "opencode");
}

#[test]
fn install_dry_run_writes_nothing() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let user = r#"{"plugin":["user-plugin"]}"#;
    user_config(&home, user);

    ceai(&config_dir, &home).args(["install", "--harness", "opencode", "--source"]).arg(&source).arg("--dry-run")
        .assert().success().stdout(predicate::str::contains("plan:"));

    // SU-4: dry-run plans but writes nothing — zero filesystem changes.
    assert!(!config_dir.join("state.json").exists(), "no state written");
    assert!(!config_dir.join("backups").exists(), "no backup written");
    assert_eq!(fs::read_to_string(home.join(".config/opencode/opencode.json")).unwrap(), user, "config untouched");
    assert!(!home.join(".config/opencode/compound-engineering").exists(), "no managed dir");
}

#[test]
fn install_unknown_harness_exits_usage_code() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home).args(["install", "--harness", "codex", "--source"]).arg(&source)
        .assert().failure().code(2);
}

#[test]
fn status_prints_installed_harness_version_and_drift() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    ceai(&config_dir, &home).arg("status").assert().success()
        .stdout(predicate::str::contains("installed: opencode"))
        .stdout(predicate::str::contains("(local"))
        .stdout(predicate::str::contains("drift: none"));
}

#[test]
fn uninstall_restores_newest_backup_and_removes_managed_files() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let user = r#"{"plugin":["user-plugin"],"agent":{"sdd-explore":{"model":"user-model"}}}"#;
    user_config(&home, user);
    install(&config_dir, &home, &source);

    ceai(&config_dir, &home).args(["uninstall", "--harness", "opencode"]).assert().success();

    // CC-3: pre-install config restored, managed files removed, state updated.
    assert_eq!(fs::read_to_string(home.join(".config/opencode/opencode.json")).unwrap(), user);
    assert!(!home.join(".config/opencode/compound-engineering").exists(), "managed dir removed");
    let state = read_json(&config_dir.join("state.json"));
    assert!(state["installed_harnesses"].as_array().unwrap().is_empty(), "state updated");
}

#[test]
fn uninstall_without_backup_removes_created_config_and_state() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);
    let opencode_json = home.join(".config/opencode/opencode.json");
    assert!(opencode_json.exists(), "install created config for a fresh user");

    ceai(&config_dir, &home).args(["uninstall", "--harness", "opencode"]).assert().success();

    assert!(!opencode_json.exists(), "config created by install is removed");
    assert!(!home.join(".config/opencode/compound-engineering").exists(), "managed dir removed");
    ceai(&config_dir, &home).arg("status").assert().success()
        .stdout(predicate::str::contains("installed: none"));
}
