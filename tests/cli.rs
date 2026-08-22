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
    // MM-2: applied to opencode.json agent.<slot>.model without clobbering user keys.
    let config = read_json(&home.join(".config/opencode/opencode.json"));
    assert_eq!(
        config["agent"]["ce-brainstorm"]["model"],
        "opencode-go/kimi-k2.6"
    );
    assert!(
        config["agent"]["ce-brainstorm"].get("variant").is_none(),
        "ce-ai never writes variant; that is user customization"
    );
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
        .current_dir(tmp.path())
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
        .current_dir(tmp.path())
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
        .current_dir(tmp.path())
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
        .current_dir(tmp.path())
        .arg("doctor")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("state-inconsistent"));
}

#[test]
fn doctor_reports_git_hooks_misconfigured_finding() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // Initialize git repo in tmp dir with invalid core.hooksPath
    let repo = tmp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    let _ = std::process::Command::new("git")
        .args(["init"])
        .current_dir(&repo)
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "core.hooksPath", "invalid_hooks"])
        .current_dir(&repo)
        .output();

    let mut cmd = ceai(&config_dir, &home);
    cmd.current_dir(&repo)
        .arg("doctor")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains(
            "git-hooks: core.hooksPath set to 'invalid_hooks'",
        ));
}

#[test]
fn doctor_reports_sibling_worktree_info() {
    let tmp = TempDir::new().unwrap();
    let tmp_path = if cfg!(windows) {
        tmp.path().to_path_buf()
    } else {
        tmp.path()
            .canonicalize()
            .unwrap_or_else(|_| tmp.path().to_path_buf())
    };
    let (config_dir, home) = (tmp_path.join("ce-ai"), tmp_path.join("home"));
    let source = ce_source(&tmp_path);
    install(&config_dir, &home, &source);

    let repo = tmp_path.join("repo");
    let wt_dir = tmp_path.join("worktrees");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&wt_dir).unwrap();

    let clean_git = |cwd: &Path| {
        let mut cmd = std::process::Command::new("git");
        cmd.current_dir(cwd)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_PREFIX");
        cmd
    };

    let _ = clean_git(&repo).args(["init", "-b", "main"]).output();

    let _ = clean_git(&repo)
        .args(["config", "core.hooksPath", ".githooks"])
        .output();
    let _ = clean_git(&repo)
        .args(["config", "user.name", "Test"])
        .output();
    let _ = clean_git(&repo)
        .args(["config", "user.email", "test@test.com"])
        .output();

    fs::create_dir_all(repo.join(".githooks")).unwrap();
    let pre_commit_path = repo.join(".githooks/pre-commit");
    fs::write(&pre_commit_path, "#!/bin/sh\nexit 0").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&pre_commit_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&pre_commit_path, perms).unwrap();
    }

    let commit_out = clean_git(&repo)
        .args(["commit", "--allow-empty", "-m", "initial"])
        .output()
        .unwrap();
    assert!(commit_out.status.success());

    let wt_path = wt_dir.join("sibling");
    let wt_arg = wt_path.to_str().unwrap();
    let wt_out = clean_git(&repo)
        .args(["worktree", "add", "-b", "wt-branch", wt_arg])
        .output()
        .unwrap();
    if !wt_out.status.success() {
        panic!(
            "git worktree add failed: {}",
            String::from_utf8_lossy(&wt_out.stderr)
        );
    }

    let mut cmd = ceai(&config_dir, &home);
    cmd.current_dir(&repo)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_PREFIX")
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "doctor-info: active sibling worktree detected",
        ));
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

#[test]
fn sync_watch_flag_parsing() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    ceai(&config_dir, &home)
        .args(["sync", "--watch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("monitoring managed paths"));
}

#[test]
fn uninstall_harness_all_with_yes_flag_test() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "all", "--all", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Uninstalled all target harnesses cleanly",
        ));
}

#[test]
fn init_prj_and_deinit_prj_roundtrip_fresh_repo() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("my-project");
    fs::create_dir_all(&prj_dir).unwrap();

    // 1. Run init-prj
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Adopted project"));

    let agents_file = prj_dir.join("AGENTS.md");
    let claude_stub = prj_dir.join("CLAUDE.md");
    assert!(agents_file.exists());
    assert!(claude_stub.exists());

    let agents_text = fs::read_to_string(&agents_file).unwrap();
    assert!(agents_text.contains("<!-- ce-ai:block begin v=2 tier=full"));
    assert!(agents_text.contains("<!-- ce-ai:block end -->"));

    let claude_text = fs::read_to_string(&claude_stub).unwrap();
    assert_eq!(claude_text.trim(), "@AGENTS.md");

    // 2. Run init-prj second time (idempotency check)
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already adopted"));

    // 3. Run deinit-prj
    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed project adoption block"));

    // Fresh repo files should be removed since they were created by ce-ai and empty after deinit
    assert!(!agents_file.exists());
    assert!(!claude_stub.exists());
}

#[test]
fn init_prj_preserves_preexisting_content_and_crlf() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("existing-project");
    fs::create_dir_all(&prj_dir).unwrap();

    let agents_file = prj_dir.join("AGENTS.md");
    let initial_content = "# My Existing Project\r\n\r\nCustom developer notes.\r\n";
    fs::write(&agents_file, initial_content).unwrap();

    // 1. Run init-prj
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "minimal"])
        .assert()
        .success();

    let updated_text = fs::read_to_string(&agents_file).unwrap();
    assert!(updated_text.starts_with("# My Existing Project\r\n"));
    assert!(updated_text.contains("<!-- ce-ai:block begin v=2 tier=minimal"));
    assert!(updated_text.contains("\r\n"));

    // 2. Run deinit-prj
    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();

    let restored_text = fs::read_to_string(&agents_file).unwrap();
    assert_eq!(restored_text, initial_content);
}

#[test]
fn init_prj_full_tier_contains_ssot_rule() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("ssot-project");
    fs::create_dir_all(&prj_dir).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let agents_text = fs::read_to_string(prj_dir.join("AGENTS.md")).unwrap();
    assert!(agents_text.contains("### Single Source of Truth Rule"));
    assert!(agents_text.contains(
        "Skip ideation skills entirely when requirements and approach are already clear."
    ));
}

#[test]
fn init_prj_orchestrator_tier_contains_distillation_line_once() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("orchestrator-project");
    fs::create_dir_all(&prj_dir).unwrap();

    ceai(&config_dir, &home)
        .args([
            "init-prj",
            prj_dir.to_str().unwrap(),
            "--tier",
            "orchestrator",
        ])
        .assert()
        .success();

    let agents_text = fs::read_to_string(prj_dir.join("AGENTS.md")).unwrap();
    assert_eq!(
        agents_text
            .matches("never maintain them in parallel")
            .count(),
        1
    );
}

#[test]
fn init_prj_minimal_block_matches_v1_bytes() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("minimal-project");
    fs::create_dir_all(&prj_dir).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "minimal"])
        .assert()
        .success();

    let agents_text = fs::read_to_string(prj_dir.join("AGENTS.md")).unwrap();
    let start = agents_text
        .find("<!-- ce-ai:block begin")
        .expect("begin marker present");
    let body_start = agents_text[start..]
        .find('\n')
        .map(|i| start + i + 1)
        .unwrap();
    let end = agents_text
        .find("<!-- ce-ai:block end -->")
        .expect("end marker present");
    let inner_body = agents_text[..end].trim_end_matches(['\n', '\r']);
    let inner_body = &inner_body[body_start..];

    let expected_v1_body = "## 🔄 Compound Engineering Workflow Guidelines\n\nAI agents operating on this codebase should follow structured planning and verification:\n- Validate scope boundaries before making changes.\n- Ensure all unit, integration, and linter tests pass before committing.\n- Document key technical learnings and post-mortem fixes.";
    assert_eq!(inner_body, expected_v1_body);
}

#[test]
fn init_prj_replaces_v1_block_with_v2_preserving_content_and_crlf() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("v1-adopted-project");
    fs::create_dir_all(&prj_dir).unwrap();

    let agents_file = prj_dir.join("AGENTS.md");
    let user_head = "# My Existing Project\r\n\r\nCustom developer notes.\r\n\r\n";
    let user_tail = "\r\nTrailing custom section.\r\n";
    let v1_block = "<!-- ce-ai:block begin v=1 tier=full sha256=deadbeef -->\r\n## 🔄 Mandatory 7-Stage Development Cycle & OpenSpec Enforcement\r\n\r\nStale v1 content.\r\n<!-- ce-ai:block end -->";
    fs::write(
        &agents_file,
        format!("{}{}{}", user_head, v1_block, user_tail),
    )
    .unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let updated_text = fs::read_to_string(&agents_file).unwrap();
    assert!(updated_text.starts_with(user_head));
    assert!(updated_text.ends_with(user_tail));
    assert!(!updated_text.contains("Stale v1 content."));
    assert!(updated_text.contains("<!-- ce-ai:block begin v=2 tier=full"));
    assert!(updated_text.contains("### Single Source of Truth Rule"));

    let state_text = fs::read_to_string(config_dir.join("state.json")).unwrap();
    let state_val: serde_json::Value = serde_json::from_str(&state_text).unwrap();
    assert_eq!(state_val["projects"][0]["block_version"], 2);
}

#[test]
fn init_prj_second_run_is_byte_identical() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("idempotent-project");
    fs::create_dir_all(&prj_dir).unwrap();

    let agents_file = prj_dir.join("AGENTS.md");
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let before = fs::read_to_string(&agents_file).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already adopted"));

    let after = fs::read_to_string(&agents_file).unwrap();
    assert_eq!(before, after);
}

#[test]
fn init_prj_upgrade_rerun_preserves_created_file_flag() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("upgrade-project");
    fs::create_dir_all(&prj_dir).unwrap();

    // Fresh adoption: ce-ai creates AGENTS.md and CLAUDE.md (created_file=true).
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    // Simulate a stale managed block so the re-run takes the replacement path.
    let agents_file = prj_dir.join("AGENTS.md");
    let text = fs::read_to_string(&agents_file).unwrap();
    fs::write(
        &agents_file,
        text.replace("## 🔄 Mandatory", "## 🔄 STALE Mandatory"),
    )
    .unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let state_val: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(config_dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(state_val["projects"][0]["created_file"], true);

    // deinit-prj must clean up agent-created files, not leave orphans.
    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed project adoption block"));

    assert!(!agents_file.exists());
    assert!(!prj_dir.join("CLAUDE.md").exists());
}

#[test]
fn doctor_reports_model_assignment_drift_and_sync_reconciles() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // Manually inject an unrecorded model assignment into opencode.json
    let opencode_json = home.join(".config/opencode/opencode.json");
    let content = fs::read_to_string(&opencode_json).unwrap();
    let mut val: serde_json::Value = serde_json::from_str(&content).unwrap();
    val["agent"]["ce-brainstorm"] = serde_json::json!({
        "model": "anthropic/claude-3-5-sonnet",
        "variant": ""
    });
    fs::write(&opencode_json, serde_json::to_string_pretty(&val).unwrap()).unwrap();

    // 1. Doctor should report model assignment drift
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .arg("doctor")
        .assert()
        .failure()
        .stdout(predicate::str::contains("model-assignment-drift"));

    // 2. Sync should reconcile model assignments bidirectionally
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .arg("sync")
        .assert()
        .success();

    // 3. Doctor should now pass cleanly
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .arg("doctor")
        .assert()
        .success();
}

// ---- skills (R1..R6) ----

#[test]
fn skills_list_outputs_catalog_table_and_json() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // List in text table mode
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .args(["skills", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Skill Registry Catalog"))
        .stdout(predicate::str::contains("ce-brainstorm"));

    // List in JSON mode
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .args(["skills", "list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"ce-brainstorm\""));
}

#[test]
fn skills_resolve_emits_markdown_prompt_and_json() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // Resolve in default markdown mode
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .args([
            "skills",
            "resolve",
            "--harness",
            "opencode",
            "--query",
            "brainstorm",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "<!-- ce-ai:skill_resolution status=",
        ))
        .stdout(predicate::str::contains("## Skills to load before work:"));

    // Resolve in JSON mode
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .args([
            "skills",
            "resolve",
            "--harness",
            "opencode",
            "--query",
            "brainstorm",
            "--json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"resolution_status\": \"paths-injected\"",
        ));
}
