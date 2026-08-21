//! CLI integration tests: install, status, uninstall (CC-1..CC-3, OI-1..OI-5, SU-4).
//! Every test pins ce-ai to hermetic temp dirs — never touches the real user config or HOME.

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn ceai(config_dir: &Path, home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ce-ai").unwrap();
    cmd.arg("--config-dir")
        .arg(config_dir)
        .env("HOME", home)
        .env("CE_AI_OPENCODE_CONFIG", home.join(".config/opencode"));
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
    ceai(config_dir, home)
        .args(["install", "--harness", "opencode", "--source"])
        .arg(source)
        .assert()
        .success();
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
    let loader_path = opencode_json
        .parent()
        .unwrap()
        .join("compound-engineering")
        .join("plugins")
        .join("compound-engineering.js");
    assert!(plugins
        .iter()
        .any(|v| v.as_str() == Some(loader_path.to_str().unwrap())));
    assert_eq!(
        config["skills"]["paths"].as_array().unwrap().len(),
        2,
        "user + CE skills path"
    );
    // OI-3: loader copied into the managed plugins dir.
    assert_eq!(
        fs::read_to_string(&loader_path).unwrap(),
        "export default function ceLoader() {}\n"
    );
    // OI-5: manifest lists every managed file and the config backup.
    let manifest = read_json(
        &opencode_json
            .parent()
            .unwrap()
            .join("compound-engineering/install-manifest.json"),
    );
    let paths: Vec<&str> = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        paths,
        vec![
            "plugins/compound-engineering.js",
            "skills/ce-brainstorm/SKILL.md"
        ]
    );
    assert!(manifest["config_mutations"][0]["backup"].as_str().is_some());
}

#[test]
fn install_reinstall_is_idempotent_without_duplicates() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    user_config(&home, r#"{"plugin":["user-plugin"]}"#);

    let mut cmd = ceai(&config_dir, &home);
    cmd.args(["install", "--harness", "opencode", "--source"])
        .arg(&source);
    cmd.assert().success();
    cmd.assert().success();

    let config = read_json(&home.join(".config/opencode/opencode.json"));
    assert_eq!(
        config["plugin"].as_array().unwrap().len(),
        2,
        "user plugin + one CE entry"
    );
    assert_eq!(
        config["skills"]["paths"].as_array().unwrap().len(),
        1,
        "no duplicate skills path"
    );
    let state = read_json(&config_dir.join("state.json"));
    assert_eq!(
        state["installed_harnesses"].as_array().unwrap().len(),
        1,
        "one state entry"
    );
    assert_eq!(state["installed_harnesses"][0]["name"], "opencode");
}

#[test]
fn install_dry_run_writes_nothing() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let user = r#"{"plugin":["user-plugin"]}"#;
    user_config(&home, user);

    ceai(&config_dir, &home)
        .args(["install", "--harness", "opencode", "--source"])
        .arg(&source)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("plan:"));

    // SU-4: dry-run plans but writes nothing — zero filesystem changes.
    assert!(!config_dir.join("state.json").exists(), "no state written");
    assert!(!config_dir.join("backups").exists(), "no backup written");
    assert_eq!(
        fs::read_to_string(home.join(".config/opencode/opencode.json")).unwrap(),
        user,
        "config untouched"
    );
    assert!(
        !home.join(".config/opencode/compound-engineering").exists(),
        "no managed dir"
    );
}

#[test]
fn install_unknown_harness_exits_usage_code() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args(["install", "--harness", "unknown_foo_harness", "--source"])
        .arg(&source)
        .assert()
        .failure()
        .code(2);
}

#[test]
fn status_prints_installed_harness_version_and_drift() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    ceai(&config_dir, &home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("installed: opencode"))
        .stdout(predicate::str::contains("(local"))
        .stdout(predicate::str::contains("drift: none"));
}

#[test]
fn uninstall_restores_newest_backup_and_removes_managed_files() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let user = r#"{"plugin":["user-plugin"],"agent":{"ce-brainstorm":{"model":"user-model"}}}"#;
    user_config(&home, user);
    install(&config_dir, &home, &source);

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "opencode"])
        .assert()
        .success();

    // CC-3: pre-install config restored, managed files removed, state updated.
    assert_eq!(
        fs::read_to_string(home.join(".config/opencode/opencode.json")).unwrap(),
        user
    );
    assert!(
        !home.join(".config/opencode/compound-engineering").exists(),
        "managed dir removed"
    );
    let state = read_json(&config_dir.join("state.json"));
    assert!(
        state["installed_harnesses"].as_array().unwrap().is_empty(),
        "state updated"
    );
}

#[test]
fn uninstall_without_backup_removes_created_config_and_state() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);
    let opencode_json = home.join(".config/opencode/opencode.json");
    assert!(
        opencode_json.exists(),
        "install created config for a fresh user"
    );

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "opencode"])
        .assert()
        .success();

    assert!(
        !opencode_json.exists(),
        "config created by install is removed"
    );
    assert!(
        !home.join(".config/opencode/compound-engineering").exists(),
        "managed dir removed"
    );
    ceai(&config_dir, &home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("installed: none"));
}

// ---- helpers for the PR 5 command suites ----

fn managed_dir(home: &Path) -> PathBuf {
    home.join(".config/opencode/compound-engineering")
}

fn manifest_path(home: &Path) -> PathBuf {
    managed_dir(home).join("install-manifest.json")
}

fn loader_path(home: &Path) -> PathBuf {
    managed_dir(home)
        .join("plugins")
        .join("compound-engineering.js")
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

/// Builds a v9 CE tarball (gzip) with a top-level dir, a changed loader, and an
/// extra skill; used to seed the upgrade cache (SU-5).
fn ce_tarball_v9(dir: &Path) -> PathBuf {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use tar::Builder;

    fn add_entry(builder: &mut Builder<Vec<u8>>, path: &str, content: &str) {
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, path, content.as_bytes())
            .unwrap();
    }

    let mut builder = Builder::new(Vec::new());
    add_entry(
        &mut builder,
        "ce-v9/.opencode/plugins/compound-engineering.js",
        "export default function ceLoaderV9() {}\n",
    );
    add_entry(
        &mut builder,
        "ce-v9/.opencode/skills/ce-brainstorm/SKILL.md",
        "# ce-brainstorm\n",
    );
    add_entry(
        &mut builder,
        "ce-v9/.opencode/skills/ce-foo/SKILL.md",
        "# ce-foo\n",
    );
    builder.finish().unwrap();
    let raw = builder.into_inner().unwrap();

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&raw).unwrap();
    let path = dir.join("ce-v9.tar.gz");
    fs::write(&path, encoder.finish().unwrap()).unwrap();
    path
}

// ---- sync (SU-1..SU-4) ----

#[test]
fn sync_restores_deleted_managed_file_and_updates_manifest() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // SU-1: a managed file goes missing on disk.
    let skill = managed_dir(&home).join("skills/ce-brainstorm/SKILL.md");
    fs::remove_file(&skill).unwrap();

    ceai(&config_dir, &home).arg("sync").assert().success();

    // SU-2: the file is restored from the source tree and the manifest updated.
    assert_eq!(fs::read_to_string(&skill).unwrap(), "# ce-brainstorm\n");
    let manifest = read_json(&manifest_path(&home));
    let files: Vec<&str> = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert_eq!(
        files,
        vec![
            "plugins/compound-engineering.js",
            "skills/ce-brainstorm/SKILL.md"
        ]
    );
    for file in manifest["files"].as_array().unwrap() {
        let disk = fs::read(managed_dir(&home).join(file["path"].as_str().unwrap())).unwrap();
        assert_eq!(
            file["sha256"].as_str().unwrap(),
            sha256_hex(&disk),
            "manifest hash matches disk"
        );
    }
}

#[test]
fn sync_dry_run_lists_changes_without_writing() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // SU-3: drift — tamper with a managed file.
    fs::write(loader_path(&home), "tampered").unwrap();
    let manifest_before = fs::read(manifest_path(&home)).unwrap();
    let state_before = fs::read(config_dir.join("state.json")).unwrap();

    // SU-4: dry-run lists the change and writes nothing.
    ceai(&config_dir, &home)
        .arg("sync")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "plan: restore plugins/compound-engineering.js",
        ));

    assert_eq!(
        fs::read(loader_path(&home)).unwrap(),
        b"tampered",
        "managed file untouched"
    );
    assert_eq!(
        fs::read(manifest_path(&home)).unwrap(),
        manifest_before,
        "manifest untouched"
    );
    assert_eq!(
        fs::read(config_dir.join("state.json")).unwrap(),
        state_before,
        "state untouched"
    );
}

// ---- models (MM-1..MM-4) ----

#[test]
fn models_set_reflects_in_state_and_opencode_config() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let user = r#"{"plugin":["user-plugin"],"agent":{"ce-brainstorm":{"model":"user-model","temperature":0.7}}}"#;
    user_config(&home, user);
    install(&config_dir, &home, &source);

    ceai(&config_dir, &home)
        .args(["models", "set", "ce-brainstorm", "opencode-go/kimi-k2.6"])
        .assert()
        .success();

    // MM-1: persisted in state.json.
    let state = read_json(&config_dir.join("state.json"));
    assert_eq!(
        state["model_assignments"]["ce-brainstorm"]["provider_id"],
        "opencode-go"
    );
    assert_eq!(
        state["model_assignments"]["ce-brainstorm"]["model_id"],
        "kimi-k2.6"
    );
    // MM-2: applied to opencode.json agent.<slot>.model/variant without clobbering user keys.
    let config = read_json(&home.join(".config/opencode/opencode.json"));
    assert_eq!(
        config["agent"]["ce-brainstorm"]["model"],
        "opencode-go/kimi-k2.6"
    );
    assert_eq!(config["agent"]["ce-brainstorm"]["variant"], "");
    assert_eq!(
        config["agent"]["ce-brainstorm"]["temperature"], 0.7,
        "user agent keys preserved"
    );
    assert_eq!(
        config["plugin"].as_array().unwrap().len(),
        2,
        "plugin merge intact"
    );
}

#[test]
fn models_set_unknown_slot_persists_with_warning() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    ceai(&config_dir, &home)
        .args([
            "models",
            "set",
            "definitely-unknown",
            "opencode-go/kimi-k2.6",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("warning"));

    let state = read_json(&config_dir.join("state.json"));
    assert_eq!(
        state["model_assignments"]["definitely-unknown"]["model_id"], "kimi-k2.6",
        "assignment persisted"
    );
    let config = read_json(&home.join(".config/opencode/opencode.json"));
    assert_eq!(
        config["agent"]["definitely-unknown"]["model"],
        "opencode-go/kimi-k2.6"
    );
}

#[test]
fn models_profile_save_load_round_trip_restores_snapshot() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    ceai(&config_dir, &home)
        .args(["models", "set", "ce-brainstorm", "opencode-go/kimi-k2.6"])
        .assert()
        .success();
    ceai(&config_dir, &home)
        .args(["models", "set", "ce-plan", "opencode-go/claude-sonnet-4"])
        .assert()
        .success();

    // MM-3/MM-4: named profile + append-only snapshot.
    ceai(&config_dir, &home)
        .args(["models", "profile", "save", "fast"])
        .assert()
        .success();
    assert!(
        config_dir.join("profiles/fast.json").exists(),
        "profile file written"
    );
    let versions = fs::read_dir(config_dir.join("profiles/versions")).unwrap();
    assert!(
        versions.into_iter().any(|e| e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("fast-")),
        "snapshot written"
    );

    // Change the assignment, then load the profile back.
    ceai(&config_dir, &home)
        .args(["models", "set", "ce-brainstorm", "opencode-go/gemini-3-pro"])
        .assert()
        .success();
    let opencode_json = home.join(".config/opencode/opencode.json");
    assert_eq!(
        read_json(&opencode_json)["agent"]["ce-brainstorm"]["model"],
        "opencode-go/gemini-3-pro"
    );

    ceai(&config_dir, &home)
        .args(["models", "profile", "load", "fast"])
        .assert()
        .success();

    let config = read_json(&opencode_json);
    assert_eq!(
        config["agent"]["ce-brainstorm"]["model"], "opencode-go/kimi-k2.6",
        "opencode.json matches snapshot"
    );
    assert_eq!(
        config["agent"]["ce-plan"]["model"],
        "opencode-go/claude-sonnet-4"
    );
    let state = read_json(&config_dir.join("state.json"));
    assert_eq!(
        state["model_assignments"]["ce-brainstorm"]["provider_id"],
        "opencode-go"
    );
    assert_eq!(
        state["model_assignments"]["ce-plan"]["model_id"],
        "claude-sonnet-4"
    );
}

// ---- upgrade (SU-5) ----

#[test]
fn upgrade_to_tag_resolves_from_cache_and_runs_sync() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // Seed the cache with a v9 tarball and record its digest in state.json.
    let tarball = ce_tarball_v9(tmp.path());
    let bytes = fs::read(&tarball).unwrap();
    let hex = sha256_hex(&bytes);
    let cache_dir = config_dir.join("cache");
    fs::create_dir_all(&cache_dir).unwrap();
    fs::write(cache_dir.join(format!("ce-{hex}.tar.gz")), &bytes).unwrap();
    let mut state = read_json(&config_dir.join("state.json"));
    state["managed_asset_digest"]["tarball"] = serde_json::json!(format!("sha256:{hex}"));
    fs::write(
        config_dir.join("state.json"),
        serde_json::to_vec_pretty(&state).unwrap(),
    )
    .unwrap();

    // SU-5 asserted via the dry-run plan; zero writes on the managed surface.
    let manifest_before = fs::read(manifest_path(&home)).unwrap();
    let state_before = fs::read(config_dir.join("state.json")).unwrap();
    ceai(&config_dir, &home)
        .args([
            "upgrade",
            "--to",
            "compound-engineering-v9.9.9",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "plan: restore plugins/compound-engineering.js",
        ))
        .stdout(predicate::str::contains(
            "plan: copy skills/ce-foo/SKILL.md",
        ));
    assert_eq!(
        fs::read_to_string(loader_path(&home)).unwrap(),
        "export default function ceLoader() {}\n",
        "managed file untouched"
    );
    assert_eq!(
        fs::read(manifest_path(&home)).unwrap(),
        manifest_before,
        "manifest untouched"
    );
    assert_eq!(
        fs::read(config_dir.join("state.json")).unwrap(),
        state_before,
        "state untouched"
    );

    // Real run: applies the sync and records the new tag as the version.
    ceai(&config_dir, &home)
        .args(["upgrade", "--to", "compound-engineering-v9.9.9"])
        .assert()
        .success();
    assert_eq!(
        fs::read_to_string(loader_path(&home)).unwrap(),
        "export default function ceLoaderV9() {}\n"
    );
    assert_eq!(
        fs::read_to_string(managed_dir(&home).join("skills/ce-foo/SKILL.md")).unwrap(),
        "# ce-foo\n"
    );
    let manifest = read_json(&manifest_path(&home));
    assert_eq!(manifest["version"], "compound-engineering-v9.9.9");
    assert_eq!(manifest["source"]["kind"], "github-release");
    assert_eq!(manifest["source"]["tag"], "compound-engineering-v9.9.9");
    let state = read_json(&config_dir.join("state.json"));
    assert_eq!(
        state["installed_harnesses"][0]["version"],
        "compound-engineering-v9.9.9"
    );
}

// ---- doctor ----

#[test]
fn doctor_clean_install_reports_ok() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("doctor: ok"));
}

#[test]
fn doctor_reports_diff_finding_with_non_zero_exit() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);
    fs::write(loader_path(&home), "tampered").unwrap();

    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains(
            "diff: modified plugins/compound-engineering.js",
        ));
}

#[test]
fn doctor_reports_config_invalid_finding() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);
    fs::write(
        home.join(".config/opencode/opencode.json"),
        "{ this is not json",
    )
    .unwrap();

    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("config-invalid"));
}

#[test]
fn doctor_reports_state_inconsistency_finding() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);
    fs::remove_file(manifest_path(&home)).unwrap();

    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("state-inconsistent"));
}

// ---- CLI completion (CC-1) ----

#[test]
fn cli_without_subcommand_exits_usage_code_2() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    ceai(&config_dir, &home).assert().failure().code(2);
}

#[test]
fn backups_list_and_restore_subcommands() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let user_v1 = r#"{"plugin":["user-plugin-v1"]}"#;
    user_config(&home, user_v1);
    install(&config_dir, &home, &source);

    // List backups
    ceai(&config_dir, &home)
        .args(["backups", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("BACKUP ID"))
        .stdout(predicate::str::contains("opencode"));

    // Mutate config
    let user_v2 = r#"{"plugin":["user-plugin-v2"]}"#;
    user_config(&home, user_v2);

    // Restore latest backup
    ceai(&config_dir, &home)
        .args(["backups", "restore", "latest", "--harness", "opencode"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully restored latest backup",
        ));

    // Verify restored content matches v1
    let restored_content = fs::read_to_string(home.join(".config/opencode/opencode.json")).unwrap();
    assert!(restored_content.contains("user-plugin-v1"));
}

#[test]
fn backups_restore_by_explicit_id() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let user_v1 = r#"{"plugin":["user-plugin-v1"]}"#;
    user_config(&home, user_v1);
    install(&config_dir, &home, &source);

    let backup_dirs: Vec<_> = fs::read_dir(config_dir.join("backups"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(backup_dirs.len(), 1);
    let backup_id = &backup_dirs[0];

    // Mutate config
    let user_v2 = r#"{"plugin":["user-plugin-v2"]}"#;
    user_config(&home, user_v2);

    // Restore by explicit snapshot ID
    ceai(&config_dir, &home)
        .args(["backups", "restore", backup_id, "-t", "opencode"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully restored backup"));

    let restored_content = fs::read_to_string(home.join(".config/opencode/opencode.json")).unwrap();
    assert!(restored_content.contains("user-plugin-v1"));
}

#[test]
fn status_validates_multi_harness_probing() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // Create host-installed claude config
    fs::write(home.join(".claude.json"), "{}").unwrap();

    ceai(&config_dir, &home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("installed: opencode"))
        .stdout(predicate::str::contains("installed: claude"));
}

#[test]
fn upgrade_local_source_protection_and_force() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // Upgrade when source is local -> notice printed and upgrade proceeds to local/release
    ceai(&config_dir, &home)
        .args(["upgrade", "--source", source.to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn companion_tools_status_and_install_subcommands() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));

    ceai(&config_dir, &home)
        .args(["tools", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Companion Tools"));

    ceai(&config_dir, &home)
        .args(["tools", "install", "engram"])
        .assert()
        .success()
        .stdout(predicate::str::contains("engram"));
}

#[test]
fn workflow_status_checkpoint_and_resume_subcommands() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));

    ceai(&config_dir, &home)
        .args(["workflow", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Workflow FSM"));

    ceai(&config_dir, &home)
        .args([
            "workflow",
            "checkpoint",
            "--task",
            "4.2 TDD",
            "--phase",
            "Stage 4: TDD",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("checkpoint saved"));

    ceai(&config_dir, &home)
        .args(["workflow", "resume"])
        .assert()
        .success()
        .stdout(predicate::str::contains("resuming execution"));
}
