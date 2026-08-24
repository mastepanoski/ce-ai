//! CLI integration tests: install, status, uninstall (CC-1..CC-3, OI-1..OI-5, SU-4).
//! Every test pins ce-ai to hermetic temp dirs — never touches the real user config or HOME.

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn ceai(config_dir: &Path, home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ce-ai").unwrap();
    cmd.arg("--config-dir")
        .arg(config_dir)
        .env("HOME", home)
        .env("CE_AI_OPENCODE_CONFIG", home.join(".config/opencode"));
    // Hermetic git resolution: under the pre-commit hook GIT_DIR points at the
    // real checkout, which would make doctor's repo probes leave the fixture.
    for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        cmd.env_remove(var);
    }
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
fn install_deepseek_harness_exits_usage_code() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args(["install", "--harness", "deepseek", "--source"])
        .arg(&source)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "deepseek harness is unsupported during developer preview",
        ));
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

    // Initialize git repo in tmp dir with invalid core.hooksPath.
    // Strip GIT_* env so this never resolves outside the fixture repo —
    // under the pre-commit hook GIT_DIR points at the real checkout.
    let repo = tmp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();
    let isolated_git = || {
        let mut cmd = std::process::Command::new("git");
        cmd.current_dir(&repo)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_PREFIX");
        cmd
    };
    let _ = isolated_git().args(["init"]).output();
    let _ = isolated_git()
        .args(["config", "core.hooksPath", "invalid_hooks"])
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

    // Status derives phase/task from the saved checkpoint (single source of truth).
    ceai(&config_dir, &home)
        .args(["workflow", "status"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("current phase: Stage 4: TDD")
                .and(predicate::str::contains("active subtask: 4.2 TDD")),
        );

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
fn init_prj_replaces_lf_only_v1_block_preserving_content() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("v1-lf-project");
    fs::create_dir_all(&prj_dir).unwrap();

    let agents_file = prj_dir.join("AGENTS.md");
    let user_head = "# LF Project\n\nCustom notes.\n\n";
    let user_tail = "\nTrailing section.\n";
    let v1_block = "<!-- ce-ai:block begin v=1 tier=full sha256=deadbeef -->\n## 🔄 Mandatory 7-Stage Development Cycle & OpenSpec Enforcement\n\nStale v1 content.\n<!-- ce-ai:block end -->";
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
    assert!(!updated_text.contains("\r\n"));
    assert!(updated_text.contains("### Single Source of Truth Rule"));
}

#[test]
fn init_prj_malformed_block_fails_closed() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("malformed-project");
    fs::create_dir_all(&prj_dir).unwrap();

    let agents_file = prj_dir.join("AGENTS.md");
    let initial =
        "# My Project\n\n<!-- ce-ai:block begin v=1 tier=full sha256=x -->\nNo end marker.\n";
    fs::write(&agents_file, initial).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .failure();

    // Fail closed: file must be byte-for-byte untouched.
    assert_eq!(fs::read_to_string(&agents_file).unwrap(), initial);
}

#[test]
fn init_prj_block_header_sha_matches_body_and_state() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("sha-project");
    fs::create_dir_all(&prj_dir).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let text = fs::read_to_string(prj_dir.join("AGENTS.md")).unwrap();
    let begin = text.find("<!-- ce-ai:block begin").unwrap();
    let line_end = begin + text[begin..].find('\n').unwrap();
    let header = &text[begin..line_end];
    let header_sha = header
        .split("sha256=")
        .nth(1)
        .unwrap()
        .split(' ')
        .next()
        .unwrap();
    let body_start = line_end + 1;
    let body_end = text.find("<!-- ce-ai:block end -->").unwrap();
    let body = text[body_start..body_end].trim_end_matches(['\n', '\r']);

    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    let computed = format!("{:x}", hasher.finalize());

    assert_eq!(header_sha, computed);

    let state_val: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(config_dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(
        state_val["projects"][0]["block_sha256"].as_str().unwrap(),
        header_sha
    );
}

#[test]
fn doctor_reports_stale_block_version_with_upgrade_hint() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("stale-project");
    fs::create_dir_all(&prj_dir).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    // Simulate a real stale v1 block: hand-written old-format block (old body,
    // v=1 header, non-matching sha) replacing the adopted content.
    let agents_file = prj_dir.join("AGENTS.md");
    let v1_block = "<!-- ce-ai:block begin v=1 tier=full sha256=deadbeef -->\n## 🔄 Mandatory 7-Stage Development Cycle & OpenSpec Enforcement\n\nStale v1 content.\n<!-- ce-ai:block end -->";
    fs::write(&agents_file, format!("# Notes\n\n{}\n", v1_block)).unwrap();

    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("stale block version v=1"))
        .stdout(predicate::str::contains(
            "re-run ce-ai init-prj --tier full to upgrade",
        ));

    ceai(&config_dir, &home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "status: STALE BLOCK v=1 — re-run ce-ai init-prj --tier full to upgrade",
        ));
}

#[test]
fn doctor_reports_generic_drift_for_tampered_v2_body() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("tampered-project");
    fs::create_dir_all(&prj_dir).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    // Tamper: keep the declared v2 version but corrupt the header sha so the
    // current-template hash no longer appears anywhere in the file.
    let agents_file = prj_dir.join("AGENTS.md");
    let text = fs::read_to_string(&agents_file).unwrap();
    let begin = text.find("<!-- ce-ai:block begin").unwrap();
    let line_end = begin + text[begin..].find('\n').unwrap();
    let mut tampered = text.clone();
    tampered.replace_range(
        begin..line_end,
        "<!-- ce-ai:block begin v=2 tier=full sha256=fedcba -->",
    );
    drop(text);
    fs::write(&agents_file, tampered).unwrap();

    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("block SHA drift detected"))
        .stdout(predicate::str::contains("stale block version").not());
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

#[test]
fn audit_subcommand_runs_advisory_and_json_mode() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // Advisory default console output
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .arg("audit")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "== [ce-ai Agent Environment Audit] ==",
        ))
        .stdout(predicate::str::contains("configuration coverage:"));

    // JSON mode
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .args(["audit", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"score_percentage\":"));

    // Fail under threshold
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .args(["audit", "--fail-under", "101"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("is below required threshold"));
}

#[test]
fn install_cursor_harness_writes_to_native_dir_and_leaves_opencode_pristine() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "cursor",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let cursor_dir = home.join(".cursor");
    let mcp_json = cursor_dir.join("mcp.json");
    assert!(mcp_json.exists());
    assert!(cursor_dir.join("compound-engineering").exists());

    let content = fs::read_to_string(&mcp_json).unwrap();
    let config: ce_ai::harness::cursor::CursorMcpConfig = serde_json::from_str(&content).unwrap();
    assert!(config.mcp_servers.contains_key("codegraph"));
    assert!(config.mcp_servers.contains_key("engram"));
    assert_eq!(
        config.mcp_servers["codegraph"].r#type.as_deref(),
        Some("stdio")
    );
    assert_eq!(
        config.mcp_servers["engram"].r#type.as_deref(),
        Some("stdio")
    );
    assert!(config.extra.is_empty(), "Zero OpenCode key leaks");

    // opencode directory must remain pristine / non-existent
    assert!(!home.join(".config/opencode").exists());
}

#[test]
fn init_prj_cursor_writes_rule_mdc() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("my-project");
    fs::create_dir_all(prj_dir.join(".cursor")).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let mdc_path = prj_dir.join(".cursor/rules/compound-engineering.mdc");
    assert!(mdc_path.exists());
    let content = fs::read_to_string(&mdc_path).unwrap();
    assert!(content.starts_with("---\n"));
    assert!(content.contains("description: Compound Engineering Agent Directives"));
    assert!(content.contains("globs: *"));
    assert!(content.contains("alwaysApply: true"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK BEGIN -->"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK END -->"));
}

#[test]
fn uninstall_cursor_harness_cleans_native_dir_artifacts() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "cursor",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "cursor"])
        .assert()
        .success();

    let cursor_dir = home.join(".cursor");
    assert!(!cursor_dir.join("compound-engineering").exists());
    assert!(!cursor_dir.join("mcp.json").exists());

    let state_file = config_dir.join("state.json");
    let state_text = fs::read_to_string(&state_file).unwrap();
    assert!(!state_text.contains("\"cursor\""));
}

#[test]
fn uninstall_cursor_harness_preserves_user_mcp_servers_in_mcp_json() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    // Pre-create user mcp.json with a custom server
    let cursor_dir = home.join(".cursor");
    fs::create_dir_all(&cursor_dir).unwrap();
    let initial_json = r#"{
  "mcpServers": {
    "user-tool": {
      "type": "stdio",
      "command": "my-tool"
    }
  }
}"#;
    fs::write(cursor_dir.join("mcp.json"), initial_json).unwrap();

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "cursor",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "cursor"])
        .assert()
        .success();

    let mcp_json = cursor_dir.join("mcp.json");
    assert!(mcp_json.exists());
    let content = fs::read_to_string(&mcp_json).unwrap();
    let config: ce_ai::harness::cursor::CursorMcpConfig = serde_json::from_str(&content).unwrap();
    assert!(config.mcp_servers.contains_key("user-tool"));
    assert!(!config.mcp_servers.contains_key("codegraph"));
    assert!(!config.mcp_servers.contains_key("engram"));
}

#[test]
fn install_claude_harness_writes_to_native_dir_and_leaves_opencode_pristine() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "claude",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let claude_json = home.join(".claude.json");
    assert!(claude_json.exists());
    assert!(home.join(".claude/skills").exists());

    let content = fs::read_to_string(&claude_json).unwrap();
    let config: ce_ai::harness::claude::ClaudeMcpConfig = serde_json::from_str(&content).unwrap();
    assert!(config.mcp_servers.contains_key("codegraph"));
    assert!(config.mcp_servers.contains_key("engram"));
    assert_eq!(
        config.mcp_servers["codegraph"].r#type.as_deref(),
        Some("stdio")
    );
    assert!(config.extra.is_empty(), "Zero OpenCode key leaks");

    // opencode directory must remain pristine / non-existent
    assert!(!home.join(".config/opencode").exists());
}

#[test]
fn init_prj_claude_writes_claude_md() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("my-project");
    fs::create_dir_all(prj_dir.join(".claude")).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let md_path = prj_dir.join("CLAUDE.md");
    assert!(md_path.exists());
    let content = fs::read_to_string(&md_path).unwrap();
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK BEGIN -->"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK END -->"));
}

#[test]
fn uninstall_claude_harness_cleans_native_dir_artifacts() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "claude",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "claude"])
        .assert()
        .success();

    let claude_json = home.join(".claude.json");
    assert!(claude_json.exists());
    let content = fs::read_to_string(&claude_json).unwrap();
    let config: ce_ai::harness::claude::ClaudeMcpConfig = serde_json::from_str(&content).unwrap();
    assert!(config.mcp_servers.is_empty());

    let state_file = config_dir.join("state.json");
    let state_text = fs::read_to_string(&state_file).unwrap();
    assert!(!state_text.contains("\"claude\""));
}

#[test]
fn install_codex_harness_writes_to_native_dir_and_leaves_opencode_pristine() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "codex",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let config_toml = home.join(".codex/config.toml");
    assert!(config_toml.exists());
    assert!(home.join(".codex/skills").exists());

    let content = fs::read_to_string(&config_toml).unwrap();
    let root: toml::Table = content.parse().unwrap();
    assert!(!root.contains_key("plugin"));
    assert!(!root.contains_key("skills"));

    let mcp = root["mcp_servers"].as_table().unwrap();
    let codegraph: ce_ai::harness::codex::CodexMcpServer =
        mcp["codegraph"].clone().try_into().unwrap();
    assert_eq!(codegraph.command, "codegraph");
    assert_eq!(codegraph.args, vec!["mcp"]);

    let engram: ce_ai::harness::codex::CodexMcpServer = mcp["engram"].clone().try_into().unwrap();
    assert_eq!(engram.command, "engram");
    assert_eq!(engram.args, vec!["serve"]);

    // opencode directory must remain pristine / non-existent
    assert!(!home.join(".config/opencode").exists());
}

#[test]
fn init_prj_codex_writes_and_deinits_agents_md() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("my-project");
    fs::create_dir_all(prj_dir.join(".codex")).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let md_path = prj_dir.join(".codex/AGENTS.md");
    assert!(md_path.exists());
    let content = fs::read_to_string(&md_path).unwrap();
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK BEGIN -->"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK END -->"));

    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();

    assert!(!md_path.exists());
}

#[test]
fn uninstall_codex_harness_cleans_native_dir_artifacts_and_preserves_user_configs() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    // Pre-populate user config
    let initial_toml = r#"model = "gpt-4o"
"#;
    let codex_dir = home.join(".codex");
    fs::create_dir_all(&codex_dir).unwrap();
    fs::write(codex_dir.join("config.toml"), initial_toml).unwrap();

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "codex",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "codex"])
        .assert()
        .success();

    let config_toml = home.join(".codex/config.toml");
    assert!(config_toml.exists());
    let content = fs::read_to_string(&config_toml).unwrap();
    let root: toml::Table = content.parse().unwrap();
    assert_eq!(root["model"].as_str().unwrap(), "gpt-4o");

    if let Some(mcp) = root.get("mcp_servers").and_then(|v| v.as_table()) {
        assert!(!mcp.contains_key("codegraph"));
        assert!(!mcp.contains_key("engram"));
    }

    assert!(!home.join(".codex/skills").exists());
    assert!(!home.join(".config/opencode").exists());

    let state_file = config_dir.join("state.json");
    let state_text = fs::read_to_string(&state_file).unwrap();
    assert!(!state_text.contains("\"codex\""));
}

#[test]
fn install_copilot_harness_writes_to_native_dir_and_leaves_opencode_pristine() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "copilot",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let mcp_config = home.join(".copilot/mcp-config.json");
    assert!(mcp_config.exists());
    assert!(home.join(".copilot/skills").exists());

    let content = fs::read_to_string(&mcp_config).unwrap();
    let json_val: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json_val.get("mcpServers").is_some());
    assert!(json_val.get("plugin").is_none());
    assert!(json_val.get("plugins").is_none());
    assert!(json_val.get("skills").is_none());

    let config: ce_ai::harness::copilot::CopilotMcpConfig = serde_json::from_str(&content).unwrap();
    assert!(config.mcp_servers.contains_key("codegraph"));
    assert!(config.mcp_servers.contains_key("engram"));
    assert_eq!(config.mcp_servers["codegraph"].command, "codegraph");
    assert_eq!(config.mcp_servers["codegraph"].args, vec!["mcp"]);
    assert!(config.extra.is_empty(), "Zero OpenCode key leaks");

    // opencode directory must remain pristine / non-existent
    assert!(!home.join(".config/opencode").exists());
}

#[test]
fn init_prj_copilot_preserves_preexisting_instructions_md_on_deinit() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("my-project");
    let github_dir = prj_dir.join(".github");
    fs::create_dir_all(&github_dir).unwrap();
    let md_path = github_dir.join("copilot-instructions.md");
    fs::write(&md_path, "# User Copilot Rules\n").unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let content = fs::read_to_string(&md_path).unwrap();
    assert!(content.starts_with("# User Copilot Rules"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK BEGIN -->"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK END -->"));

    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();

    assert!(md_path.exists());
    let stripped = fs::read_to_string(&md_path).unwrap();
    assert_eq!(stripped.trim(), "# User Copilot Rules");
}

#[test]
fn uninstall_copilot_harness_clean_install_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "copilot",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(home.join(".copilot/mcp-config.json").exists());
    assert!(home.join(".copilot/skills").exists());

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "copilot"])
        .assert()
        .success();

    let content = fs::read_to_string(home.join(".copilot/mcp-config.json")).unwrap();
    let config: ce_ai::harness::copilot::CopilotMcpConfig = serde_json::from_str(&content).unwrap();
    assert!(config.mcp_servers.is_empty());
    assert!(!home.join(".copilot/skills").exists());
}

#[test]
fn uninstall_copilot_harness_cleans_native_dir_artifacts_and_preserves_user_configs() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    // Pre-populate user config
    let initial_json = r#"{
  "telemetry": false
}"#;
    let copilot_dir = home.join(".copilot");
    fs::create_dir_all(&copilot_dir).unwrap();
    fs::write(copilot_dir.join("mcp-config.json"), initial_json).unwrap();

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "copilot",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "copilot"])
        .assert()
        .success();

    let mcp_config = home.join(".copilot/mcp-config.json");
    assert!(mcp_config.exists());
    let content = fs::read_to_string(&mcp_config).unwrap();
    let config: ce_ai::harness::copilot::CopilotMcpConfig = serde_json::from_str(&content).unwrap();
    assert_eq!(
        config.extra.get("telemetry").unwrap(),
        &serde_json::Value::Bool(false)
    );
    assert!(!config.mcp_servers.contains_key("codegraph"));
    assert!(!config.mcp_servers.contains_key("engram"));

    assert!(!home.join(".copilot/skills").exists());
    assert!(!home.join(".config/opencode").exists());

    let state_file = config_dir.join("state.json");
    let state_text = fs::read_to_string(&state_file).unwrap();
    assert!(!state_text.contains("\"copilot\""));
}

#[test]
fn install_grok_harness_writes_to_native_dir_and_leaves_opencode_pristine() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "grok",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let grok_config = home.join(".grok/config.toml");
    assert!(grok_config.exists());
    assert!(home.join(".grok/skills").exists());

    let content = fs::read_to_string(&grok_config).unwrap();
    let root: toml::Table = content.parse().unwrap();
    let mcp = root["mcp_servers"].as_table().unwrap();
    assert!(mcp.contains_key("codegraph"));
    assert!(mcp.contains_key("engram"));
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

    // opencode directory must remain pristine / non-existent
    assert!(!home.join(".config/opencode").exists());
}

#[test]
fn init_prj_grok_preserves_preexisting_rules_md_on_deinit() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("my-project");
    let rules_dir = prj_dir.join(".grok").join("rules");
    fs::create_dir_all(&rules_dir).unwrap();
    let md_path = rules_dir.join("compound-engineering.md");
    fs::write(&md_path, "# User Grok Rules\n").unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let content = fs::read_to_string(&md_path).unwrap();
    assert!(content.starts_with("# User Grok Rules"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK BEGIN -->"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK END -->"));

    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();

    assert!(md_path.exists());
    let stripped = fs::read_to_string(&md_path).unwrap();
    assert_eq!(stripped.trim(), "# User Grok Rules");
}

#[test]
fn uninstall_grok_harness_clean_install_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "grok",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(home.join(".grok/config.toml").exists());
    assert!(home.join(".grok/skills").exists());

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "grok"])
        .assert()
        .success();

    let content = fs::read_to_string(home.join(".grok/config.toml")).unwrap();
    let root: toml::Table = content.parse().unwrap();
    if let Some(mcp) = root.get("mcp_servers").and_then(|v| v.as_table()) {
        assert!(mcp.is_empty());
    }
    assert!(!home.join(".grok/skills").exists());
}

#[test]
fn uninstall_grok_harness_cleans_native_dir_artifacts_and_preserves_user_configs() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    // Pre-populate user config
    let initial_toml = r#"
model = "grok-beta"
"#;
    let grok_dir = home.join(".grok");
    fs::create_dir_all(&grok_dir).unwrap();
    fs::write(grok_dir.join("config.toml"), initial_toml).unwrap();

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "grok",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "grok"])
        .assert()
        .success();

    let grok_config = home.join(".grok/config.toml");
    assert!(grok_config.exists());
    let content = fs::read_to_string(&grok_config).unwrap();
    let root: toml::Table = content.parse().unwrap();
    assert_eq!(root["model"].as_str().unwrap(), "grok-beta");
    if let Some(mcp) = root.get("mcp_servers").and_then(|v| v.as_table()) {
        assert!(!mcp.contains_key("codegraph"));
        assert!(!mcp.contains_key("engram"));
    }

    assert!(!home.join(".grok/skills").exists());
    assert!(!home.join(".config/opencode").exists());

    let state_file = config_dir.join("state.json");
    let state_text = fs::read_to_string(&state_file).unwrap();
    assert!(!state_text.contains("\"grok\""));
}

#[test]
fn uninstall_failure_propagates_error_and_preserves_state() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "cursor",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let state_file = config_dir.join("state.json");
    let initial_state = std::fs::read_to_string(&state_file).unwrap();
    assert!(initial_state.contains("cursor"));

    // Convert target_config (.cursor/mcp.json) into a non-empty directory to force IO failure cross-platform on Windows, macOS, and Linux
    let target_config = home.join(".cursor").join("mcp.json");
    std::fs::remove_file(&target_config).unwrap();
    std::fs::create_dir_all(target_config.join("blocker")).unwrap();

    let result = ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "cursor"])
        .assert();

    // 1. Assert non-zero exit status (must fail)
    result.failure();

    // 2. Unconditionally assert state preservation
    let current_state = std::fs::read_to_string(&state_file).unwrap();
    assert!(
        current_state.contains("cursor"),
        "state.json should preserve cursor harness entry when uninstall fails"
    );
}

#[test]
fn uninstall_invalid_harness_name_returns_usage_error() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "invalid-harness-xyz"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn init_prj_kimi_writes_and_deinits_agents_md() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("my-project");
    let kimi_dir = prj_dir.join(".kimi-code");
    let legacy_rules_dir = kimi_dir.join("rules");
    fs::create_dir_all(&legacy_rules_dir).unwrap();
    let md_path = kimi_dir.join("AGENTS.md");
    fs::write(&md_path, "# User Kimi Rules\n").unwrap();
    let legacy_rule_path = legacy_rules_dir.join("compound-engineering.md");
    fs::write(
        &legacy_rule_path,
        "<!-- CE-AI MANAGED BLOCK BEGIN -->\nlegacy block\n<!-- CE-AI MANAGED BLOCK END -->",
    )
    .unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let content = fs::read_to_string(&md_path).unwrap();
    assert!(content.starts_with("# User Kimi Rules"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK BEGIN -->"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK END -->"));

    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();

    assert!(md_path.exists());
    let stripped = fs::read_to_string(&md_path).unwrap();
    assert_eq!(stripped.trim(), "# User Kimi Rules");

    // Legacy rule file must be cleaned up and deleted when empty
    assert!(!legacy_rule_path.exists());
}

#[test]
fn install_kimi_harness_writes_to_native_dir_and_leaves_opencode_pristine() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "kimi",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let kimi_config = home.join(".kimi-code/mcp.json");
    assert!(kimi_config.exists());
    assert!(home.join(".kimi-code/skills").exists());

    let content = fs::read_to_string(&kimi_config).unwrap();
    let config: serde_json::Value = serde_json::from_str(&content).unwrap();
    let mcp = config["mcpServers"].as_object().unwrap();
    assert!(mcp.contains_key("codegraph"));
    assert!(mcp.contains_key("engram"));
    let codegraph = mcp["codegraph"].as_object().unwrap();
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

    assert!(config.get("plugin").is_none(), "Zero OpenCode key leaks");
    assert!(
        config.get("skills.paths").is_none(),
        "Zero OpenCode key leaks"
    );
    assert!(!content.contains("plugin"), "Zero OpenCode key leaks");

    // opencode directory must remain pristine / non-existent
    assert!(!home.join(".config/opencode").exists());
}

#[test]
fn uninstall_kimi_harness_clean_install_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "kimi",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(home.join(".kimi-code/mcp.json").exists());
    assert!(home.join(".kimi-code/skills").exists());

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "kimi"])
        .assert()
        .success();

    let content = fs::read_to_string(home.join(".kimi-code/mcp.json")).unwrap();
    let config: serde_json::Value = serde_json::from_str(&content).unwrap();
    if let Some(mcp) = config.get("mcpServers").and_then(|v| v.as_object()) {
        assert!(mcp.is_empty());
    }
    assert!(!home.join(".kimi-code/skills").exists());
    assert!(!home.join(".config/opencode").exists());

    let state_text = fs::read_to_string(config_dir.join("state.json")).unwrap();
    assert!(!state_text.contains("\"kimi\""));
}

#[test]
fn uninstall_kimi_harness_cleans_native_dir_artifacts_and_preserves_user_configs() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    // Pre-populate user config
    let initial_json = r#"{
      "user_setting": "enabled",
      "mcpServers": {
        "user_custom": {
          "command": "my-custom-cmd",
          "args": ["run"]
        }
      }
    }"#;
    let kimi_dir = home.join(".kimi-code");
    fs::create_dir_all(&kimi_dir).unwrap();
    fs::write(kimi_dir.join("mcp.json"), initial_json).unwrap();

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "kimi",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "kimi"])
        .assert()
        .success();

    let kimi_config = home.join(".kimi-code/mcp.json");
    assert!(kimi_config.exists());
    let content = fs::read_to_string(&kimi_config).unwrap();
    let config: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(config["user_setting"].as_str().unwrap(), "enabled");
    let mcp = config["mcpServers"].as_object().unwrap();
    assert!(mcp.contains_key("user_custom"));
    assert!(!mcp.contains_key("codegraph"));
    assert!(!mcp.contains_key("engram"));
    assert!(!home.join(".kimi-code/skills").exists());
    assert!(!home.join(".config/opencode").exists());

    let state_text = fs::read_to_string(config_dir.join("state.json")).unwrap();
    assert!(!state_text.contains("\"kimi\""));
}

#[test]
fn init_prj_agy_writes_and_deinits_rules() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("my-project");
    let rules_dir = prj_dir.join(".agents").join("rules");
    fs::create_dir_all(&rules_dir).unwrap();
    let md_path = rules_dir.join("compound-engineering.md");
    fs::write(&md_path, "# User AGY Rules\n").unwrap();
    let gemini_path = prj_dir.join("GEMINI.md");
    fs::write(&gemini_path, "# User GEMINI Rules\n").unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let content = fs::read_to_string(&md_path).unwrap();
    assert!(content.starts_with("# User AGY Rules"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK BEGIN -->"));
    assert!(content.contains("<!-- CE-AI MANAGED BLOCK END -->"));

    let gemini_content = fs::read_to_string(&gemini_path).unwrap();
    assert!(gemini_content.starts_with("# User GEMINI Rules"));
    assert!(gemini_content.contains("<!-- CE-AI MANAGED BLOCK BEGIN -->"));

    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();

    assert!(md_path.exists());
    let stripped = fs::read_to_string(&md_path).unwrap();
    assert_eq!(stripped.trim(), "# User AGY Rules");

    assert!(gemini_path.exists());
    let g_stripped = fs::read_to_string(&gemini_path).unwrap();
    assert_eq!(g_stripped.trim(), "# User GEMINI Rules");
}

#[test]
fn install_agy_harness_writes_to_native_dir_and_leaves_opencode_pristine() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "agy",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let agy_config = home.join(".gemini/config/mcp_config.json");
    assert!(agy_config.exists());
    assert!(home.join(".gemini/config/skills").exists());

    let content = fs::read_to_string(&agy_config).unwrap();
    let config: serde_json::Value = serde_json::from_str(&content).unwrap();
    let mcp = config["mcpServers"].as_object().unwrap();
    assert!(mcp.contains_key("codegraph"));
    assert!(mcp.contains_key("engram"));
    let codegraph = mcp["codegraph"].as_object().unwrap();
    assert_eq!(codegraph["command"].as_str().unwrap(), "codegraph");

    assert!(config.get("plugin").is_none(), "Zero OpenCode key leaks");
    assert!(
        config.pointer("/skills/paths").is_none(),
        "Zero OpenCode key leaks"
    );
    assert!(!content.contains("plugin"), "Zero OpenCode key leaks");

    // opencode directory must remain pristine / non-existent
    assert!(!home.join(".config/opencode").exists());
}

#[test]
fn uninstall_agy_harness_clean_install_lifecycle() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "agy",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(home.join(".gemini/config/mcp_config.json").exists());
    assert!(home.join(".gemini/config/skills").exists());

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "agy"])
        .assert()
        .success();

    let content = fs::read_to_string(home.join(".gemini/config/mcp_config.json")).unwrap();
    let config: serde_json::Value = serde_json::from_str(&content).unwrap();
    if let Some(mcp) = config.get("mcpServers").and_then(|v| v.as_object()) {
        assert!(mcp.is_empty());
    }
    assert!(!home.join(".gemini/config/skills").exists());
    assert!(!home.join(".config/opencode").exists());

    let state_text = fs::read_to_string(config_dir.join("state.json")).unwrap();
    assert!(!state_text.contains("\"agy\""));
}

#[test]
fn uninstall_agy_harness_cleans_native_dir_artifacts_and_preserves_user_configs() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    // Pre-populate user config with remote server using serverUrl
    let initial_json = r#"{
      "user_setting": "enabled",
      "mcpServers": {
        "user_remote": {
          "serverUrl": "https://mcp.example.com/sse",
          "headers": { "Auth": "Bearer token" }
        }
      }
    }"#;
    let gemini_config_dir = home.join(".gemini/config");
    fs::create_dir_all(&gemini_config_dir).unwrap();
    fs::write(gemini_config_dir.join("mcp_config.json"), initial_json).unwrap();

    // Pre-populate legacy antigravity.json
    let legacy_dir = home.join(".gemini/antigravity-cli");
    fs::create_dir_all(&legacy_dir).unwrap();
    fs::write(legacy_dir.join("antigravity.json"), "{}").unwrap();

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "agy",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    // Verify post-install state: serverUrl preserved alongside codegraph and engram
    let post_install_content =
        fs::read_to_string(gemini_config_dir.join("mcp_config.json")).unwrap();
    let post_install_config: serde_json::Value =
        serde_json::from_str(&post_install_content).unwrap();
    let post_mcp = post_install_config["mcpServers"].as_object().unwrap();
    assert!(post_mcp.contains_key("user_remote"));
    assert_eq!(
        post_mcp["user_remote"]["serverUrl"].as_str().unwrap(),
        "https://mcp.example.com/sse"
    );
    assert!(post_mcp.contains_key("codegraph"));
    assert!(post_mcp.contains_key("engram"));

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "agy"])
        .assert()
        .success();

    let agy_config = home.join(".gemini/config/mcp_config.json");
    assert!(agy_config.exists());
    let content = fs::read_to_string(&agy_config).unwrap();
    let config: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(config["user_setting"].as_str().unwrap(), "enabled");
    let mcp = config["mcpServers"].as_object().unwrap();
    assert!(mcp.contains_key("user_remote"));
    let remote = &mcp["user_remote"];
    assert_eq!(
        remote["serverUrl"].as_str().unwrap(),
        "https://mcp.example.com/sse"
    );
    assert!(!mcp.contains_key("codegraph"));
    assert!(!mcp.contains_key("engram"));

    // Legacy antigravity.json must be cleaned up
    assert!(!legacy_dir.join("antigravity.json").exists());
    assert!(!home.join(".gemini/config/skills").exists());
    assert!(!home.join(".config/opencode").exists());

    let state_text = fs::read_to_string(config_dir.join("state.json")).unwrap();
    assert!(!state_text.contains("\"agy\""));
}

#[test]
fn install_pi_harness_writes_to_native_dir_and_leaves_opencode_pristine() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "pi",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let pi_dir = home.join(".pi/agent");
    assert!(pi_dir.join("skills").exists());
    assert!(pi_dir.join("skills/ce-brainstorm/SKILL.md").exists());

    // Pi does NOT write any mcp.json, config.json, mcp-config.json or plugins.json
    assert!(!pi_dir.join("config.json").exists());
    assert!(!pi_dir.join("mcp.json").exists());
    assert!(!pi_dir.join("mcp-config.json").exists());
    assert!(!pi_dir.join("plugins.json").exists());

    // opencode directory must remain pristine / non-existent
    assert!(!home.join(".config/opencode").exists());
}

#[test]
fn uninstall_pi_harness_cleans_native_dir_artifacts_and_preserves_user_configs() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    // Pre-populate custom user skill in ~/.pi/agent/
    let user_custom_dir = home.join(".pi/agent/user_custom_skill");
    fs::create_dir_all(&user_custom_dir).unwrap();
    let user_skill_file = user_custom_dir.join("SKILL.md");
    fs::write(&user_skill_file, "# My Custom Skill\n").unwrap();

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "pi",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "pi"])
        .assert()
        .success();

    assert!(!home.join(".pi/agent/skills").exists());
    assert!(user_skill_file.exists());
    assert_eq!(
        fs::read_to_string(&user_skill_file).unwrap(),
        "# My Custom Skill\n"
    );
    assert!(!home.join(".config/opencode").exists());

    let state_text = fs::read_to_string(config_dir.join("state.json")).unwrap();
    assert!(!state_text.contains("\"pi\""));
}

#[test]
fn install_pi_harness_respects_pi_coding_agent_dir_env() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let custom_pi_dir = tmp.path().join("custom_pi_agent");
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .env("PI_CODING_AGENT_DIR", custom_pi_dir.to_str().unwrap())
        .args([
            "install",
            "--harness",
            "pi",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(custom_pi_dir.join("skills").exists());
    assert!(custom_pi_dir.join("skills/ce-brainstorm/SKILL.md").exists());
    assert!(!home.join(".pi").exists());

    ceai(&config_dir, &home)
        .env("PI_CODING_AGENT_DIR", custom_pi_dir.to_str().unwrap())
        .args(["uninstall", "--harness", "pi"])
        .assert()
        .success();

    assert!(!custom_pi_dir.join("skills").exists());
}

#[test]
fn install_fx_harness_writes_to_native_dir_and_leaves_opencode_pristine() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "fx",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    let fx_dir = home.join(".fx");
    let mcp_config = fx_dir.join("mcp.json");
    assert!(mcp_config.exists());
    assert!(fx_dir.join("skills/ce-brainstorm/SKILL.md").exists());

    let content = fs::read_to_string(&mcp_config).unwrap();
    let config: ce_ai::harness::fx::FxMcpConfig = serde_json::from_str(&content).unwrap();
    assert!(config.mcp.contains_key("codegraph"));
    assert!(config.mcp.contains_key("engram"));

    let codegraph = &config.mcp["codegraph"];
    assert_eq!(codegraph.r#type.as_deref(), Some("local"));
    assert_eq!(codegraph.command, vec!["codegraph", "mcp"]);
    assert!(codegraph.environment.is_empty());

    let engram = &config.mcp["engram"];
    assert_eq!(engram.r#type.as_deref(), Some("local"));
    assert_eq!(engram.command, vec!["engram", "serve"]);
    assert!(engram.environment.is_empty());

    // opencode directory must remain pristine / non-existent
    assert!(!home.join(".config/opencode").exists());
}

#[test]
fn uninstall_fx_harness_cleans_native_dir_artifacts_and_preserves_user_configs() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    let mcp_config_file = home.join(".fx/mcp.json");
    fs::create_dir_all(mcp_config_file.parent().unwrap()).unwrap();
    let user_json = r#"{
        "user_setting": "enabled",
        "mcp": {
            "user_remote": {
                "type": "http",
                "url": "https://mcp.example.com"
            }
        }
    }"#;
    fs::write(&mcp_config_file, user_json).unwrap();

    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "fx",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "fx"])
        .assert()
        .success();

    assert!(mcp_config_file.exists());
    let content = fs::read_to_string(&mcp_config_file).unwrap();
    let config: ce_ai::harness::fx::FxMcpConfig = serde_json::from_str(&content).unwrap();
    assert_eq!(config.extra.get("user_setting").unwrap(), "enabled");
    assert!(config.mcp.contains_key("user_remote"));
    assert!(!config.mcp.contains_key("codegraph"));
    assert!(!config.mcp.contains_key("engram"));

    assert!(!home.join(".fx/skills").exists());
    assert!(!home.join(".config/opencode").exists());

    let state_text = fs::read_to_string(config_dir.join("state.json")).unwrap();
    assert!(!state_text.contains("\"fx\""));
}

#[test]
fn install_fx_harness_respects_fx_home_env() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let custom_fx_dir = tmp.path().join("custom_fx_dir");
    let source = ce_source(tmp.path());

    ceai(&config_dir, &home)
        .env("FX_HOME", custom_fx_dir.to_str().unwrap())
        .args([
            "install",
            "--harness",
            "fx",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(custom_fx_dir.join("mcp.json").exists());
    assert!(custom_fx_dir.join("skills/ce-brainstorm/SKILL.md").exists());
    assert!(!home.join(".fx").exists());

    ceai(&config_dir, &home)
        .env("FX_HOME", custom_fx_dir.to_str().unwrap())
        .args(["uninstall", "--harness", "fx"])
        .assert()
        .success();

    assert!(!custom_fx_dir.join("mcp.json").exists());
    assert!(!custom_fx_dir.join("skills").exists());
}
