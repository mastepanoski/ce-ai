//! CLI integration tests: install, status, uninstall (CC-1..CC-3, OI-1..OI-5, SU-4).
//! Every test pins ce-ai to hermetic temp dirs — never touches the real user config or HOME.

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

/// Current managed adoption-block version under test. Every test literal that
/// embeds a block header must derive from this constant: a stale hardcoded
/// version flips classifier branches after a bump (see
/// docs/solutions/test-failures/adoption-block-version-bump-test-coordination-2026-08-25.md).
const CUR_BLOCK_VERSION: u32 = 4;

fn block_begin_prefix(tier: &str) -> String {
    format!("<!-- ce-ai:block begin v={CUR_BLOCK_VERSION} tier={tier}")
}

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

/// Local CE source-tree fixture with the real release layout: loader under
/// `.opencode/plugins`, skills at the top-level `skills/` directory.
fn ce_source_top_level_skills(dir: &Path) -> PathBuf {
    let loader = dir.join("ce-tree/.opencode/plugins/compound-engineering.js");
    fs::create_dir_all(loader.parent().unwrap()).unwrap();
    fs::write(&loader, "export default function ceLoader() {}\n").unwrap();
    let skill = dir.join("ce-tree/skills/ce-brainstorm/SKILL.md");
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
fn install_harvests_top_level_skills_into_managed_dir_and_manifest() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source_top_level_skills(tmp.path());

    install(&config_dir, &home, &source);

    let managed_skill =
        home.join(".config/opencode/compound-engineering/skills/ce-brainstorm/SKILL.md");
    assert!(managed_skill.exists());
    let manifest = read_json(
        &home
            .join(".config/opencode/compound-engineering")
            .join("install-manifest.json"),
    );
    let paths: Vec<&str> = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"skills/ce-brainstorm/SKILL.md"));
}

#[test]
fn install_harness_receives_no_skill_files_from_harvest() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source_top_level_skills(tmp.path());
    // claude must be detected so `--harness all` targets it.
    fs::create_dir_all(home.join(".claude")).unwrap();

    ceai(&config_dir, &home)
        .args(["install", "--harness", "all", "--source"])
        .arg(&source)
        .assert()
        .success();

    // Token-neutrality: harvested skills stay in the OpenCode managed dir;
    // harness-owned directories receive nothing. Install opencode explicitly
    // so the managed surface exists regardless of host detection.
    assert!(!home.join(".claude/skills").exists());
    ceai(&config_dir, &home)
        .args(["install", "--harness", "opencode", "--source"])
        .arg(&source)
        .assert()
        .success();
    assert!(home
        .join(".config/opencode/compound-engineering/skills/ce-brainstorm/SKILL.md")
        .exists());
}

#[test]
fn sync_matrix_verifies_harvested_managed_surface() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source_top_level_skills(tmp.path());

    install(&config_dir, &home, &source);
    ceai(&config_dir, &home)
        .args(["sync"])
        .assert()
        .success()
        .stdout(predicates::str::contains("managed files match SHA256"));
}

#[test]
fn sync_with_host_detected_native_harnesses_reports_registered_and_succeeds() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source_top_level_skills(tmp.path());

    // Create markers for host-detected harnesses
    fs::create_dir_all(home.join(".config/opencode")).unwrap();
    fs::write(home.join(".config/opencode/opencode.json"), "{}").unwrap();
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::write(home.join(".claude/settings.json"), "{}").unwrap();
    fs::create_dir_all(home.join(".copilot")).unwrap();
    fs::write(home.join(".copilot/mcp-config.json"), "{}").unwrap();
    fs::create_dir_all(home.join(".codex")).unwrap();
    fs::write(home.join(".codex/config.toml"), "").unwrap();

    // Install all detected harnesses
    ceai(&config_dir, &home)
        .args(["install", "--harness", "all", "--source"])
        .arg(&source)
        .assert()
        .success();

    // Sync must report native harnesses as registered and succeed without drift
    ceai(&config_dir, &home)
        .args(["sync"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "✓ opencode: verified — 2/2 managed files match SHA256",
        ))
        .stdout(predicates::str::contains(
            "○ claude: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)",
        ))
        .stdout(predicates::str::contains(
            "○ copilot: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)",
        ))
        .stdout(predicates::str::contains(
            "○ codex: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)",
        ))
        .stdout(predicates::str::contains(
            "0 failed",
        ));
}

#[test]
fn install_prefers_top_level_skills_on_overlap() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    // Overlap: same skill in both layouts with different content.
    let legacy = source.join(".opencode/skills/ce-brainstorm/SKILL.md");
    fs::write(&legacy, "# legacy version\n").unwrap();
    let top = source.join("skills/ce-brainstorm/SKILL.md");
    fs::create_dir_all(top.parent().unwrap()).unwrap();
    fs::write(&top, "# top-level version\n").unwrap();

    ceai(&config_dir, &home)
        .args(["install", "--harness", "opencode", "--source"])
        .arg(&source)
        .assert()
        .success()
        .stderr(predicates::str::contains("top-level wins"));

    let managed_skill =
        home.join(".config/opencode/compound-engineering/skills/ce-brainstorm/SKILL.md");
    assert_eq!(
        fs::read_to_string(&managed_skill).unwrap(),
        "# top-level version\n"
    );
}

/// Local CE source-tree fixture with the real release layout and two skills,
/// for partial-set completion scenarios.
fn ce_source_top_level_skills_two(dir: &Path) -> PathBuf {
    let source = ce_source_top_level_skills(dir);
    let extra = source.join("skills/ce-work/SKILL.md");
    fs::create_dir_all(extra.parent().unwrap()).unwrap();
    fs::write(&extra, "# ce-work\n").unwrap();
    source
}

fn stale_local_copy(home: &Path) -> PathBuf {
    let stale = home.join(".config/opencode/skills/ce-brainstorm/SKILL.md");
    fs::create_dir_all(stale.parent().unwrap()).unwrap();
    fs::write(
        &stale,
        "---\nname: ce-brainstorm\ndescription: brainstorm\n---\n# stale local copy\n",
    )
    .unwrap();
    stale
}

#[test]
fn adopt_yes_executes_transactional_adoption_and_retires_managed_copy() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source_top_level_skills(tmp.path());
    install(&config_dir, &home, &source);
    let stale = stale_local_copy(&home);

    ceai(&config_dir, &home)
        .args(["skills", "adopt", "--harness", "opencode", "--yes"])
        .assert()
        .success()
        .stdout(predicates::str::contains("adopted under ce-ai management"));

    // Stale copy rewritten to canonical content.
    assert_eq!(fs::read_to_string(&stale).unwrap(), "# ce-brainstorm\n");
    // Managed-dir skills tree retired whole (R13): ce-ai-owned territory.
    assert!(!home
        .join(".config/opencode/compound-engineering/skills")
        .exists());
    let manifest = read_json(
        &home
            .join(".config/opencode/compound-engineering")
            .join("install-manifest.json"),
    );
    let manifest_skills: Vec<&serde_json::Value> = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| {
            f["path"]
                .as_str()
                .unwrap_or_default()
                .starts_with("skills/")
        })
        .collect();
    assert!(manifest_skills.is_empty());
    // Ledger records the adopted surface with tracked files.
    let state = read_json(&config_dir.join("state.json"));
    let surfaces = state["skill_surfaces"].as_array().unwrap();
    assert_eq!(surfaces.len(), 1);
    assert_eq!(surfaces[0]["status"], "adopted");
    assert_eq!(surfaces[0]["harness"], "opencode");
    assert!(!surfaces[0]["files"].as_array().unwrap().is_empty());
    assert!(surfaces[0]["adopted_at"].is_string());
}

#[test]
fn adopt_completes_partial_surfaces_to_full_canonical_set() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source_top_level_skills_two(tmp.path());
    install(&config_dir, &home, &source);
    // User only holds one of the two canonical skills, and stale.
    let stale = stale_local_copy(&home);

    ceai(&config_dir, &home)
        .args(["skills", "adopt", "--harness", "opencode", "--yes"])
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&stale).unwrap(), "# ce-brainstorm\n");
    let completed = home.join(".config/opencode/skills/ce-work/SKILL.md");
    assert_eq!(fs::read_to_string(&completed).unwrap(), "# ce-work\n");
    let state = read_json(&config_dir.join("state.json"));
    let files = state["skill_surfaces"][0]["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
}

#[test]
fn adopt_is_idempotent_and_preserves_adopted_at() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source_top_level_skills(tmp.path());
    install(&config_dir, &home, &source);
    stale_local_copy(&home);

    ceai(&config_dir, &home)
        .args(["skills", "adopt", "--harness", "opencode", "--yes"])
        .assert()
        .success();
    let first =
        read_json(&config_dir.join("state.json"))["skill_surfaces"][0]["adopted_at"].clone();

    ceai(&config_dir, &home)
        .args(["skills", "adopt", "--harness", "opencode", "--yes"])
        .assert()
        .success();
    let second =
        read_json(&config_dir.join("state.json"))["skill_surfaces"][0]["adopted_at"].clone();
    assert_eq!(first, second);
    assert_eq!(
        fs::read_to_string(stale_local_copy_path(&home)).unwrap(),
        "# ce-brainstorm\n"
    );
}

fn stale_local_copy_path(home: &Path) -> PathBuf {
    home.join(".config/opencode/skills/ce-brainstorm/SKILL.md")
}

#[test]
fn uninstall_adopted_surface_is_scoped_and_cleans_ledger() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source_top_level_skills(tmp.path());
    install(&config_dir, &home, &source);
    stale_local_copy(&home);
    ceai(&config_dir, &home)
        .args(["skills", "adopt", "--harness", "opencode", "--yes"])
        .assert()
        .success();
    // User-authored file inside the adopted root: never tracked, never removed.
    let notes = home.join(".config/opencode/skills/user-notes.md");
    fs::write(&notes, "mine\n").unwrap();

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "opencode", "--yes"])
        .assert()
        .success();

    assert!(!stale_local_copy_path(&home).exists());
    assert!(notes.exists());
    let state = read_json(&config_dir.join("state.json"));
    assert!(
        state.get("skill_surfaces").is_none()
            || state["skill_surfaces"]
                .as_array()
                .is_none_or(|a| a.is_empty())
    );
}

#[test]
fn uninstall_without_adoption_never_touches_harness_skills_dir() {
    // Regression pin: the legacy whole-dir skills removal is gone. User
    // content in a harness skills root survives install+uninstall cycles.
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let own = home.join(".claude/skills/user-own/SKILL.md");
    fs::create_dir_all(own.parent().unwrap()).unwrap();
    fs::write(&own, "user authored\n").unwrap();

    ceai(&config_dir, &home)
        .args(["install", "--harness", "claude", "--source"])
        .arg(&source)
        .assert()
        .success();
    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "claude", "--yes"])
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&own).unwrap(), "user authored\n");
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
fn uninstall_deepseek_harness_exits_usage_code() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "deepseek"])
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

    // Seed the cache with a v9 tarball and record digest + release provenance
    // in state.json (#161 contract: --to binds to matching provenance).
    let tarball = ce_tarball_v9(tmp.path());
    let bytes = fs::read(&tarball).unwrap();
    let hex = sha256_hex(&bytes);
    let tag = "compound-engineering-v9.9.9";
    let cache_dir = config_dir.join("cache");
    fs::create_dir_all(&cache_dir).unwrap();
    fs::write(cache_dir.join(format!("ce-{hex}.tar.gz")), &bytes).unwrap();
    let mut state = read_json(&config_dir.join("state.json"));
    state["managed_asset_digest"]["tarball"] = serde_json::json!(format!("sha256:{hex}"));
    state["release_provenance"] = serde_json::json!({
        "tag": tag,
        "url": format!("https://example.test/ce-{hex}.tar.gz"),
        "archive_sha256": hex,
        "extraction_path": config_dir
            .join("cache/trees")
            .join(tag)
            .to_string_lossy()
            .to_string(),
    });
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

#[test]
fn upgrade_to_mismatched_tag_fails_without_relabeling() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // Cache holds v9 provenance; request a different tag.
    let tarball = ce_tarball_v9(tmp.path());
    let bytes = fs::read(&tarball).unwrap();
    let hex = sha256_hex(&bytes);
    let tag = "compound-engineering-v9.9.9";
    let cache_dir = config_dir.join("cache");
    fs::create_dir_all(&cache_dir).unwrap();
    fs::write(cache_dir.join(format!("ce-{hex}.tar.gz")), &bytes).unwrap();
    let mut state = read_json(&config_dir.join("state.json"));
    state["managed_asset_digest"]["tarball"] = serde_json::json!(format!("sha256:{hex}"));
    state["release_provenance"] = serde_json::json!({
        "tag": tag,
        "url": format!("https://example.test/ce-{hex}.tar.gz"),
        "archive_sha256": hex,
        "extraction_path": config_dir
            .join("cache/trees")
            .join(tag)
            .to_string_lossy()
            .to_string(),
    });
    fs::write(
        config_dir.join("state.json"),
        serde_json::to_vec_pretty(&state).unwrap(),
    )
    .unwrap();

    ceai(&config_dir, &home)
        .args(["upgrade", "--to", "compound-engineering-v1.0.0"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("usage error"))
        .stderr(predicate::str::contains("compound-engineering-v1.0.0"))
        .stderr(predicate::str::contains("never relabels"));

    // Provenance still binds the artifact to v9 — no relabeling happened.
    let after = read_json(&config_dir.join("state.json"));
    assert_eq!(after["release_provenance"]["tag"], tag);
    assert_eq!(
        read_json(&manifest_path(&home))["version"],
        "local",
        "manifest untouched by the aborted upgrade"
    );
}

#[test]
fn upgrade_tampered_cache_fails_closed_with_exit_six() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    let tarball = ce_tarball_v9(tmp.path());
    let bytes = fs::read(&tarball).unwrap();
    let hex = sha256_hex(&bytes);
    let tag = "compound-engineering-v9.9.9";
    let cache_dir = config_dir.join("cache");
    fs::create_dir_all(&cache_dir).unwrap();
    let cached_path = cache_dir.join(format!("ce-{hex}.tar.gz"));
    fs::write(&cached_path, &bytes).unwrap();
    let mut state = read_json(&config_dir.join("state.json"));
    state["managed_asset_digest"]["tarball"] = serde_json::json!(format!("sha256:{hex}"));
    state["release_provenance"] = serde_json::json!({
        "tag": tag,
        "url": format!("https://example.test/ce-{hex}.tar.gz"),
        "archive_sha256": hex,
        "extraction_path": config_dir
            .join("cache/trees")
            .join(tag)
            .to_string_lossy()
            .to_string(),
    });
    fs::write(
        config_dir.join("state.json"),
        serde_json::to_vec_pretty(&state).unwrap(),
    )
    .unwrap();

    // Tamper with the cached archive in place.
    fs::write(&cached_path, b"tampered-payload").unwrap();

    ceai(&config_dir, &home)
        .args(["upgrade", "--to", tag])
        .assert()
        .failure()
        .code(6)
        .stderr(predicate::str::contains("verification error"))
        .stderr(predicate::str::contains("integrity check failed"));

    // State untouched by the aborted upgrade.
    let after = read_json(&config_dir.join("state.json"));
    assert_eq!(after["release_provenance"]["tag"], tag);
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

fn dir_snapshot(dir: &Path) -> std::collections::BTreeMap<PathBuf, String> {
    let mut map = std::collections::BTreeMap::new();
    if !dir.exists() {
        return map;
    }
    for file in walkdir_recursive(dir) {
        if let Ok(rel) = file.strip_prefix(dir) {
            if let Ok(bytes) = fs::read(&file) {
                use sha2::Digest;
                let hash = format!("{:x}", sha2::Sha256::digest(&bytes));
                map.insert(rel.to_path_buf(), hash);
            }
        }
    }
    map
}

fn walkdir_recursive(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walkdir_recursive(&path));
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    files
}

fn assert_dry_run_zero_mutation(
    config_dir: &Path,
    home_dir: &Path,
    workspace_dir: &Path,
    args: &[&str],
) {
    let before_config = dir_snapshot(config_dir);
    let before_home = dir_snapshot(home_dir);
    let before_workspace = dir_snapshot(workspace_dir);

    let mut full_args = vec!["--dry-run"];
    full_args.extend_from_slice(args);
    ceai(config_dir, home_dir)
        .args(full_args)
        .assert()
        .success();

    let after_config = dir_snapshot(config_dir);
    let after_home = dir_snapshot(home_dir);
    let after_workspace = dir_snapshot(workspace_dir);

    assert_eq!(
        before_config, after_config,
        "config_dir mutated during dry-run!"
    );
    assert_eq!(before_home, after_home, "home_dir mutated during dry-run!");
    assert_eq!(
        before_workspace, after_workspace,
        "workspace_dir mutated during dry-run!"
    );
}

#[test]
fn workflow_status_checkpoint_and_resume_subcommands() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let workspace = tmp.path().join("workspace");

    ceai(&config_dir, &home)
        .args(["workflow", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Workflow FSM"));

    // Stage 1 -> Stage 2: Valid transition
    ceai(&config_dir, &home)
        .args([
            "workflow",
            "checkpoint",
            "--stage",
            "2",
            "--task",
            "Authoring proposal.md",
            "--feature",
            "dry-run-purity",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("checkpoint saved"));

    // Stage 2 -> Stage 5: Invalid jump (fails with exit code 2)
    ceai(&config_dir, &home)
        .args([
            "workflow",
            "checkpoint",
            "--stage",
            "5",
            "--task",
            "Empirical testing",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("invalid workflow transition"));

    // Status derives phase/task from saved workflow state
    ceai(&config_dir, &home)
        .args(["workflow", "status"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("current phase: Stage 2: OpenSpec Definition (openspec)").and(
                predicate::str::contains("active subtask: Authoring proposal.md"),
            ),
        );

    // Dry-run workflow checkpoint zero mutation
    assert_dry_run_zero_mutation(
        &config_dir,
        &home,
        &workspace,
        &["workflow", "checkpoint", "--stage", "3", "--task", "Plan"],
    );

    ceai(&config_dir, &home)
        .args(["workflow", "resume"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("resuming execution").and(predicate::str::contains(
                "== [Environment State & Drift Status] ==",
            )),
        );
}

#[test]
fn workflow_resume_surfaces_live_repo_state_and_detects_drift() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // 1. Advance through workflow FSM: Stage 2 -> Stage 3 -> Stage 4
    ceai(&config_dir, &home)
        .args([
            "workflow",
            "checkpoint",
            "--stage",
            "2",
            "--task",
            "Specifying feature",
            "--feature",
            "zero-step-drift-recovery",
        ])
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args([
            "workflow",
            "checkpoint",
            "--stage",
            "3",
            "--task",
            "Planning feature",
        ])
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args([
            "workflow",
            "checkpoint",
            "--stage",
            "4",
            "--task",
            "Implementing feature",
        ])
        .assert()
        .success();

    // 2. Resume output contains RepoState block
    ceai(&config_dir, &home)
        .args(["workflow", "resume"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("== [Environment State & Drift Status] ==")
                .and(predicate::str::contains("git branch:"))
                .and(predicate::str::contains("working tree:"))
                .and(predicate::str::contains("manifest integrity: clean")),
        );

    // 3. Resume --json returns repo_state object
    let assert_json = ceai(&config_dir, &home)
        .args(["workflow", "resume", "--json"])
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert_json.get_output().stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(parsed.get("repo_state").is_some());
    assert_eq!(
        parsed["workflow"]["feature_name"].as_str().unwrap(),
        "zero-step-drift-recovery"
    );
    assert_eq!(
        parsed["repo_state"]["manifest_drift_count"]
            .as_u64()
            .unwrap(),
        0
    );

    // 4. Mutate a managed file to simulate external drift
    let loader_file = managed_dir(&home).join("plugins/compound-engineering.js");
    fs::write(&loader_file, "external modification").unwrap();

    // 5. Resume immediately surfaces drift warning in Turn 0
    ceai(&config_dir, &home)
        .args(["workflow", "resume"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("manifest integrity: ! 1 files modified outside ce-ai").and(
                predicate::str::contains(
                    "Drift detected in managed files. Run 'ce-ai sync' to reconcile.",
                ),
            ),
        );
}

#[test]
fn sync_watch_detects_and_repairs_drift() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // Mutate a managed file to introduce drift
    let loader_file = managed_dir(&home).join("plugins/compound-engineering.js");
    assert!(loader_file.exists(), "managed loader missing after install");
    fs::write(&loader_file, "drift content").unwrap();

    ceai(&config_dir, &home)
        .args([
            "sync",
            "--watch",
            "--max-passes",
            "2",
            "--interval-ms",
            "10",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("repaired drift"));
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
    assert!(agents_text.contains(&block_begin_prefix("full")));
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
    assert!(updated_text.contains(&block_begin_prefix("minimal")));
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
    assert!(agents_text.contains("retained by default as the permanent raw-history record"));
    assert!(agents_text.contains("\"disposable\" never means deleting them."));
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
    assert_eq!(
        agents_text
            .matches("retain them as raw history instead of deleting them")
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
fn init_prj_replaces_stale_v1_block_with_current_preserving_content_and_crlf() {
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
    assert!(updated_text.contains(&block_begin_prefix("full")));
    assert!(updated_text.contains("### Single Source of Truth Rule"));

    let state_text = fs::read_to_string(config_dir.join("state.json")).unwrap();
    let state_val: serde_json::Value = serde_json::from_str(&state_text).unwrap();
    assert_eq!(state_val["projects"][0]["block_version"], CUR_BLOCK_VERSION);
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
    assert!(updated_text.contains(&block_begin_prefix("full")));
    assert!(!updated_text.contains("\r\n"));
    assert!(updated_text.contains("### Single Source of Truth Rule"));
}

#[test]
fn init_prj_upgrades_stale_v2_block_to_v3_preserving_provenance() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("v2-adopted-project");
    fs::create_dir_all(&prj_dir).unwrap();

    // Adopt for real so the project is registered in state.json.
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    // Simulate a project adopted while BLOCK_VERSION was 2: replace the file
    // contents with user content plus a hand-written v2 block whose sha cannot
    // match the current template.
    let agents_file = prj_dir.join("AGENTS.md");
    let user_head = "# My Existing Project\r\n\r\nCustom developer notes.\r\n\r\n";
    let user_tail = "\r\nTrailing custom section.\r\n";
    let v2_block = "<!-- ce-ai:block begin v=2 tier=full sha256=deadbeef -->\r\n## \u{1f504} Mandatory 7-Stage Development Cycle & OpenSpec Enforcement\r\n\r\nStale v2 content.\r\n<!-- ce-ai:block end -->";
    fs::write(
        &agents_file,
        format!("{}{}{}", user_head, v2_block, user_tail),
    )
    .unwrap();

    // Pre-bump-style diagnostics: an untouched v2 adoption classifies as stale.
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .failure()
        .stdout(predicate::str::contains("stale block version v=2"))
        .stdout(predicate::str::contains(
            "re-run ce-ai init-prj --tier full to upgrade",
        ));

    // Re-running init-prj takes the replacement path with no migration step.
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let updated_text = fs::read_to_string(&agents_file).unwrap();
    assert!(updated_text.starts_with(user_head));
    assert!(updated_text.ends_with(user_tail));
    assert!(!updated_text.contains("Stale v2 content."));
    assert!(updated_text.contains(&block_begin_prefix("full")));
    assert!(updated_text.contains("retained by default as the permanent raw-history record"));
    assert!(updated_text.contains("\r\n"));

    let state_val: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(config_dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(state_val["projects"][0]["block_version"], CUR_BLOCK_VERSION);
    // Provenance is carried forward through the replacement path, not
    // recomputed (see init-prj-created-file-clobber learning): the initial
    // adoption created this file, so the flag must survive the upgrade.
    assert_eq!(state_val["projects"][0]["created_file"], true);

    // Third run: the upgraded block must hit the already-adopted early-return
    // and stay byte-identical (post-upgrade idempotency).
    let upgraded = fs::read_to_string(&agents_file).unwrap();
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already adopted"));
    assert_eq!(fs::read_to_string(&agents_file).unwrap(), upgraded);
}

#[test]
fn init_prj_upgrades_stale_v2_block_to_v3_orchestrator_tier() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("v2-orchestrator-project");
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

    let agents_file = prj_dir.join("AGENTS.md");
    let user_head = "# Orchestrator Workspace\n\nCustom notes.\n\n";
    let v2_block = "<!-- ce-ai:block begin v=2 tier=orchestrator sha256=deadbeef -->\n## \u{1f504} Orchestrator Agent Governance & Delegation Directives\n\nStale orchestrator content.\n<!-- ce-ai:block end -->";
    fs::write(
        &agents_file,
        format!("{}{}{}", user_head, v2_block, "\nTrailing section.\n"),
    )
    .unwrap();

    // The upgrade hint must interpolate the registered tier.
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .failure()
        .stdout(predicate::str::contains("stale block version v=2"))
        .stdout(predicate::str::contains(
            "re-run ce-ai init-prj --tier orchestrator to upgrade",
        ));

    ceai(&config_dir, &home)
        .args([
            "init-prj",
            prj_dir.to_str().unwrap(),
            "--tier",
            "orchestrator",
        ])
        .assert()
        .success();

    let updated_text = fs::read_to_string(&agents_file).unwrap();
    assert!(updated_text.starts_with(user_head));
    assert!(!updated_text.contains("Stale orchestrator content."));
    assert!(updated_text.contains(&block_begin_prefix("orchestrator")));
    assert_eq!(
        updated_text
            .matches("retain them as raw history instead of deleting them")
            .count(),
        1
    );

    let state_val: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(config_dir.join("state.json")).unwrap()).unwrap();
    assert_eq!(state_val["projects"][0]["block_version"], CUR_BLOCK_VERSION);
    assert_eq!(state_val["projects"][0]["created_file"], true);
}

#[test]
fn doctor_classifies_byte_identical_minimal_v2_body_as_healthy() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("minimal-v2-project");
    fs::create_dir_all(&prj_dir).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "minimal"])
        .assert()
        .success();

    // Rewrite the header as if adopted while BLOCK_VERSION was 2, keeping the
    // body byte-identical to the current template and the SHA matching it.
    // The SHA short-circuit precedes the declared-version check, so this must
    // classify Ok — no stale hint — even though v=2 < BLOCK_VERSION.
    let agents_file = prj_dir.join("AGENTS.md");
    let text = fs::read_to_string(&agents_file).unwrap();
    let begin = text.find("<!-- ce-ai:block begin").unwrap();
    let line_end = begin + text[begin..].find('\n').unwrap();
    let body_start = line_end + 1;
    let body_end = text.find("<!-- ce-ai:block end -->").unwrap();
    let body = text[body_start..body_end].trim_end_matches(['\n', '\r']);
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    let body_sha = format!("{:x}", hasher.finalize());
    let mut rewritten = text.clone();
    rewritten.replace_range(
        begin..line_end,
        &format!("<!-- ce-ai:block begin v=2 tier=minimal sha256={body_sha} -->"),
    );
    drop(text);
    fs::write(&agents_file, rewritten).unwrap();

    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("stale block version").not());

    ceai(&config_dir, &home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("STALE BLOCK").not());
}

#[test]
fn ssot_retention_core_phrasing_is_consistent_across_surfaces() {
    const SHARED_PHRASES: [&str; 2] = [
        "retained by default as the permanent raw-history record",
        "\"disposable\" never means deleting them",
    ];

    // Surface A: repo-root AGENTS.md (located via the compile-time manifest
    // path — deterministic, independent of ambient cwd).
    let root_agents =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("AGENTS.md"))
            .expect("root AGENTS.md readable");

    // Surface B: the rendered full-tier managed block.
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("surface-parity-project");
    fs::create_dir_all(&prj_dir).unwrap();
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();
    let agents_text = fs::read_to_string(prj_dir.join("AGENTS.md")).unwrap();

    for phrase in SHARED_PHRASES {
        assert!(
            root_agents.contains(phrase),
            "root AGENTS.md drifted from shared retention phrasing: {phrase}"
        );
        assert!(
            agents_text.contains(phrase),
            "rendered full-tier block drifted from shared retention phrasing: {phrase}"
        );
    }
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
fn doctor_reports_generic_drift_for_tampered_current_body() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("tampered-project");
    fs::create_dir_all(&prj_dir).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    // Tamper: keep the declared current version but corrupt the header sha so
    // the current-template hash no longer appears anywhere in the file.
    // The literal must match BLOCK_VERSION: an older declared version would
    // classify as StaleVersion instead of exercising the generic-drift branch.
    let agents_file = prj_dir.join("AGENTS.md");
    let text = fs::read_to_string(&agents_file).unwrap();
    let begin = text.find("<!-- ce-ai:block begin").unwrap();
    let line_end = begin + text[begin..].find('\n').unwrap();
    let mut tampered = text.clone();
    tampered.replace_range(
        begin..line_end,
        &format!("<!-- ce-ai:block begin v={CUR_BLOCK_VERSION} tier=full sha256=fedcba -->"),
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
    assert!(!home.join(".claude/skills").exists());

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
    assert!(!home.join(".codex/skills").exists());

    let content = fs::read_to_string(&config_toml).unwrap();
    let root: toml::Table = content.parse().unwrap();
    assert!(!root.contains_key("plugin"));
    assert!(!root.contains_key("skills"));

    let mcp = root["mcp_servers"].as_table().unwrap();
    let codegraph = mcp["codegraph"].as_table().unwrap();
    assert_eq!(codegraph["command"].as_str(), Some("codegraph"));

    let engram = mcp["engram"].as_table().unwrap();
    assert_eq!(engram["command"].as_str(), Some("engram"));

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
    assert!(!home.join(".copilot/skills").exists());

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
    assert!(!home.join(".copilot/skills").exists());

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
    assert!(!home.join(".grok/skills").exists());

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
    assert!(!home.join(".grok/skills").exists());

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
    assert!(!home.join(".kimi-code/skills").exists());

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
    assert!(!home.join(".kimi-code/skills").exists());

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
    assert!(!home.join(".gemini/config/skills").exists());

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
    assert!(!home.join(".gemini/config/skills").exists());

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
    assert!(!pi_dir.join("skills").exists());
    assert!(!pi_dir.join("skills/ce-brainstorm/SKILL.md").exists());

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

    assert!(!custom_pi_dir.join("skills").exists());
    assert!(!custom_pi_dir.join("skills/ce-brainstorm/SKILL.md").exists());
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
    assert!(!fx_dir.join("skills/ce-brainstorm/SKILL.md").exists());

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
    assert!(!custom_fx_dir.join("skills/ce-brainstorm/SKILL.md").exists());
    assert!(!home.join(".fx").exists());

    ceai(&config_dir, &home)
        .env("FX_HOME", custom_fx_dir.to_str().unwrap())
        .args(["uninstall", "--harness", "fx"])
        .assert()
        .success();

    assert!(!custom_fx_dir.join("mcp.json").exists());
    assert!(!custom_fx_dir.join("skills").exists());
}

// ---------------------------------------------------------------------------
// R4: --harness custom fallback mode (openspec/changes/custom-harness-r4)
// ---------------------------------------------------------------------------

fn custom_install(config_dir: &Path, home: &Path, source: &Path, plugins: &Path, skills: &Path) {
    ceai(config_dir, home)
        .args(["install", "--harness", "custom", "--plugins-dir"])
        .arg(plugins)
        .arg("--skills-dir")
        .arg(skills)
        .arg("--source")
        .arg(source)
        .assert()
        .success();
}

fn custom_manifest_path(plugins_dir: &Path) -> PathBuf {
    plugins_dir.join("compound-engineering/install-manifest.json")
}

#[test]
fn install_custom_without_configuration_fails_fast_with_usage_exit() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());

    // No flags, no ~/.ce-ai/custom_harness.json → Usage (exit 2), zero writes.
    ceai(&config_dir, &home)
        .args([
            "install",
            "--harness",
            "custom",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--plugins-dir"));

    assert!(!home.join(".config/custom").exists(), "no fictional dir");
    assert!(!home.join(".custom").exists(), "no legacy dir");
    assert!(!home.join(".ce-ai/custom_harness.json").exists());
    assert!(
        !config_dir.join("state.json").exists() || {
            let state = read_json(&config_dir.join("state.json"));
            state["installed_harnesses"].as_array().unwrap().is_empty()
        },
        "no custom state entry"
    );
}

#[test]
fn install_custom_dry_run_prints_resolved_plan_without_writing() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let (plugins, skills, rules) = (
        home.join("my/plugins"),
        home.join("my/skills"),
        home.join("my/rules.md"),
    );

    ceai(&config_dir, &home)
        .args(["install", "--harness", "custom", "--plugins-dir"])
        .arg(&plugins)
        .arg("--skills-dir")
        .arg(&skills)
        .arg("--rules-file")
        .arg(&rules)
        .arg("--source")
        .arg(&source)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(
            predicate::str::contains(format!("plan: create {}", plugins.display()))
                .and(predicate::str::contains(format!(
                    "plan: create {}",
                    skills.display()
                )))
                .and(predicate::str::contains(format!(
                    "plan: ensure managed CE block in {}",
                    rules.display()
                ))),
        );

    assert!(
        !plugins.exists() && !skills.exists() && !rules.exists(),
        "SU-4"
    );
}

#[test]
fn install_custom_with_flags_copies_layout_manifest_state_and_rules_block() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let (plugins, skills, rules) = (
        home.join("harness/plugins"),
        home.join("harness/skills"),
        home.join("harness/rules.md"),
    );
    fs::create_dir_all(rules.parent().unwrap()).unwrap();
    fs::write(&rules, "# my harness rules\nkeep me\n").unwrap();

    custom_install(&config_dir, &home, &source, &plugins, &skills);
    ceai(&config_dir, &home)
        .args(["install", "--harness", "custom", "--plugins-dir"])
        .arg(&plugins)
        .arg("--skills-dir")
        .arg(&skills)
        .arg("--rules-file")
        .arg(&rules)
        .arg("--source")
        .arg(&source)
        .assert()
        .success();

    // Layout: managed rel paths map directly under the configured roots.
    assert_eq!(
        fs::read_to_string(plugins.join("compound-engineering.js")).unwrap(),
        "export default function ceLoader() {}\n"
    );
    assert_eq!(
        fs::read_to_string(skills.join("ce-brainstorm/SKILL.md")).unwrap(),
        "# ce-brainstorm\n"
    );

    // Manifest with per-file SHA256 under the managed dir.
    let manifest = read_json(&custom_manifest_path(&plugins));
    let files = manifest["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    let loader = files
        .iter()
        .find(|f| f["path"] == "plugins/compound-engineering.js")
        .unwrap();
    assert_eq!(
        loader["sha256"],
        sha256_hex(b"export default function ceLoader() {}\n")
    );
    assert_eq!(
        manifest["config_mutations"][0]["file"],
        rules.display().to_string()
    );

    // Rules file keeps user bytes and gains exactly one current block.
    let text = fs::read_to_string(&rules).unwrap();
    assert!(text.starts_with("# my harness rules\nkeep me\n"));
    assert!(text.contains(&format!(
        "ce-ai:block begin v={CUR_BLOCK_VERSION} tier=full"
    )));
    assert_eq!(text.matches("ce-ai:block begin").count(), 1);

    // State entry embeds the resolved configuration.
    let state = read_json(&config_dir.join("state.json"));
    let entry = state["installed_harnesses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["name"] == "custom")
        .expect("custom state entry");
    assert_eq!(
        entry["custom"]["plugins_dir"],
        plugins.display().to_string()
    );
    assert_eq!(entry["custom"]["skills_dir"], skills.display().to_string());

    // Idempotent reinstall: no duplicated blocks or entries.
    let state = read_json(&config_dir.join("state.json"));
    assert_eq!(
        state["installed_harnesses"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["name"] == "custom")
            .count(),
        1
    );

    assert!(!home.join(".config/custom").exists());
}

#[test]
fn install_custom_config_file_works_and_flags_take_precedence() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let cfg_file = home.join(".ce-ai/custom_harness.json");
    fs::create_dir_all(cfg_file.parent().unwrap()).unwrap();
    fs::write(
        &cfg_file,
        serde_json::json!({
            "plugins_dir": "~/file/plugins",
            "skills_dir": "~/file/skills"
        })
        .to_string(),
    )
    .unwrap();

    // Config-file-only install resolves ~ against the hermetic HOME.
    ceai(&config_dir, &home)
        .args(["install", "--harness", "custom", "--source"])
        .arg(&source)
        .assert()
        .success();
    assert!(home.join("file/plugins/compound-engineering.js").exists());
    assert!(home.join("file/skills/ce-brainstorm/SKILL.md").exists());

    // Flags override the persisted file values.
    let flag_plugins = home.join("flag/plugins");
    let flag_skills = home.join("flag/skills");
    ceai(&config_dir, &home)
        .args(["install", "--harness", "custom", "--plugins-dir"])
        .arg(&flag_plugins)
        .arg("--skills-dir")
        .arg(&flag_skills)
        .arg("--source")
        .arg(&source)
        .assert()
        .success();
    assert!(flag_plugins.join("compound-engineering.js").exists());
    assert!(!flag_plugins.join("custom_harness.json").exists());

    let state = read_json(&config_dir.join("state.json"));
    let entry = state["installed_harnesses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["name"] == "custom")
        .unwrap();
    assert_eq!(
        entry["custom"]["plugins_dir"],
        flag_plugins.display().to_string()
    );
}

#[test]
fn uninstall_custom_is_surgical_and_strips_managed_block() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let (plugins, skills, rules) = (
        home.join("harness/plugins"),
        home.join("harness/skills"),
        home.join("harness/rules.md"),
    );
    fs::create_dir_all(rules.parent().unwrap()).unwrap();
    fs::write(&rules, "# mine\nstay\n").unwrap();

    ceai(&config_dir, &home)
        .args(["install", "--harness", "custom", "--plugins-dir"])
        .arg(&plugins)
        .arg("--skills-dir")
        .arg(&skills)
        .arg("--rules-file")
        .arg(&rules)
        .arg("--source")
        .arg(&source)
        .assert()
        .success();

    // Foreign content the user owns inside the same roots.
    fs::write(skills.join("user-notes.md"), "user data\n").unwrap();

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "custom", "-y"])
        .assert()
        .success();

    // Manifest-recorded files gone; foreign content untouched.
    assert!(!plugins.join("compound-engineering.js").exists());
    assert!(!skills.join("ce-brainstorm/SKILL.md").exists());
    assert!(!custom_manifest_path(&plugins).exists());
    assert_eq!(
        fs::read_to_string(skills.join("user-notes.md")).unwrap(),
        "user data\n"
    );

    // Block stripped; every other byte preserved.
    assert_eq!(fs::read_to_string(&rules).unwrap(), "# mine\nstay\n");

    // State entry dropped.
    let state = read_json(&config_dir.join("state.json"));
    assert!(state["installed_harnesses"]
        .as_array()
        .unwrap()
        .iter()
        .all(|h| h["name"] != "custom"));
}

#[test]
fn uninstall_all_skips_absent_custom_install_gracefully() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    ceai(&config_dir, &home)
        .args(["uninstall", "--harness", "all", "-y"])
        .assert()
        .success();
}

#[test]
fn sync_restores_drifted_custom_assets_and_reports_verified_surface() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let (plugins, skills) = (home.join("p"), home.join("s"));

    // Sync is opencode-anchored: it needs the opencode manifest to resolve
    // the desired tree, then reconciles every active harness including custom.
    install(&config_dir, &home, &source);
    custom_install(&config_dir, &home, &source, &plugins, &skills);

    // Drift: a custom skill file disappears from disk.
    let drifted = skills.join("ce-brainstorm/SKILL.md");
    fs::remove_file(&drifted).unwrap();

    ceai(&config_dir, &home).arg("sync").assert().success();

    // Repaired and hash-verified like native skill surfaces.
    assert_eq!(fs::read_to_string(&drifted).unwrap(), "# ce-brainstorm\n");
    let manifest = read_json(&custom_manifest_path(&plugins));
    for file in manifest["files"].as_array().unwrap() {
        let rel = file["path"].as_str().unwrap();
        let disk = if let Some(rest) = rel.strip_prefix("plugins/") {
            fs::read(plugins.join(rest)).unwrap()
        } else {
            fs::read(skills.join(rel.strip_prefix("skills/").unwrap())).unwrap()
        };
        assert_eq!(file["sha256"].as_str().unwrap(), sha256_hex(&disk));
    }

    // State rebuild preserved the directory snapshot (SU-5/R4.4).
    let state = read_json(&config_dir.join("state.json"));
    let entry = state["installed_harnesses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["name"] == "custom")
        .expect("snapshot survived sync");
    assert_eq!(
        entry["custom"]["plugins_dir"],
        plugins.display().to_string()
    );
}

// ---------------------------------------------------------------------------
// context-resilience R1: doctor branch-protection health probe
// ---------------------------------------------------------------------------

/// Creates a hermetic fake `gh` executable and returns its bin dir.
#[cfg(unix)]
fn fake_gh(dir: &Path, behavior: &str) -> PathBuf {
    let bin = dir.join("fake-bin");
    fs::create_dir_all(&bin).unwrap();
    let script = match behavior {
        // Simulates an unprotected main (GitHub answers 404).
        "notfound" => "#!/bin/sh\necho 'gh: Not Found (HTTP 404)' >&2\nexit 1\n",
        // Protected main with required status checks.
        _ => "#!/bin/sh\necho '{\"required_status_checks\":{\"contexts\":[\"ci\"]}}'\n",
    };
    let path = bin.join("gh");
    fs::write(&path, script).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    bin
}

#[cfg(unix)]
fn doctor_with_fake_gh(
    config_dir: &Path,
    home: &Path,
    repo: &Path,
    gh_bin: &Path,
) -> assert_cmd::assert::Assert {
    let mut cmd = ceai(config_dir, home);
    let path_var = std::env::var("PATH").unwrap_or_default();
    cmd.env("PATH", format!("{}:{path_var}", gh_bin.display()))
        .current_dir(repo)
        .arg("doctor");
    cmd.assert()
}

#[cfg(unix)]
#[test]
fn doctor_flags_unprotected_github_main() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let repo = tmp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();

    let mut git = std::process::Command::new("git");
    git.current_dir(&repo)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_PREFIX");
    let isolated = || {
        let mut c = std::process::Command::new("git");
        c.current_dir(&repo)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_PREFIX");
        c
    };
    git.args(["init"]).output().unwrap();
    isolated()
        .args([
            "remote",
            "add",
            "origin",
            "git@github.com:acme/ce-ai-test.git",
        ])
        .output()
        .unwrap();

    let gh_bin = fake_gh(tmp.path(), "notfound");
    doctor_with_fake_gh(&config_dir, &home, &repo, &gh_bin)
        .failure()
        .code(1)
        .stdout(predicate::str::contains(
            "branch-protection: missing or unconfigured for main",
        ));
}

#[cfg(unix)]
#[test]
fn doctor_stays_quiet_when_main_is_protected() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let repo = tmp.path().join("repo");
    fs::create_dir_all(&repo).unwrap();

    let mut git = std::process::Command::new("git");
    git.current_dir(&repo)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_PREFIX");
    let isolated = || {
        let mut c = std::process::Command::new("git");
        c.current_dir(&repo)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_PREFIX");
        c
    };
    git.args(["init"]).output().unwrap();
    isolated()
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/acme/ce-ai-test.git",
        ])
        .output()
        .unwrap();

    let gh_bin = fake_gh(tmp.path(), "protected");
    // A bare repo trips unrelated findings (e.g. missing skill registry);
    // what this test pins is that a *protected* main raises no
    // branch-protection finding and reports the reviews advisory.
    doctor_with_fake_gh(&config_dir, &home, &repo, &gh_bin)
        .stdout(predicate::str::contains(
            "branch-protection: PR reviews not required on main",
        ))
        .stdout(predicate::str::contains("branch-protection: missing or unconfigured").not());
}

// ---------------------------------------------------------------------------
// transactional-ops: operation journal + deterministic recovery (#166)
// ---------------------------------------------------------------------------

mod journal_fault_injection {
    use super::*;

    #[test]
    fn injected_fault_mid_adopt_auto_restores_surface_and_ledger() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source_top_level_skills(tmp.path());
        install(&config_dir, &home, &source);
        let stale = stale_local_copy(&home);
        let managed_skill =
            home.join(".config/opencode/compound-engineering/skills/ce-brainstorm/SKILL.md");
        assert!(managed_skill.exists());

        ceai(&config_dir, &home)
            .env("CE_AI_FAIL_AFTER_WRITES", "0")
            .args(["skills", "adopt", "--harness", "opencode", "--yes"])
            .assert()
            .failure();

        // Auto-restore (R15): stale content back, managed copy untouched
        // (retirement runs after the writes), no adopted ledger entry.
        assert!(fs::read_to_string(&stale)
            .unwrap()
            .contains("# stale local copy"));
        assert!(managed_skill.exists());
        let state = read_json(&config_dir.join("state.json"));
        assert!(
            state.get("skill_surfaces").is_none()
                || state["skill_surfaces"]
                    .as_array()
                    .is_none_or(|a| a.is_empty())
        );
    }

    fn user_opencode_config(home: &Path) {
        let dir = home.join(".config/opencode");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("opencode.json"),
            r#"{"plugin":["user-plugin"],"skills":{"paths":["/home/user/skills"]}}"#,
        )
        .unwrap();
    }

    #[test]
    fn injected_fault_mid_install_leaves_journal_and_next_run_recovers() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source(tmp.path());
        user_opencode_config(&home);

        ceai(&config_dir, &home)
            .env("CE_AI_FAIL_AFTER_WRITES", "2")
            .args(["install", "--harness", "opencode", "--source"])
            .arg(&source)
            .assert()
            .failure();

        // Journal survives the crash and doctor diagnoses it (#166 criterion).
        assert!(config_dir.join("install-journal.json").exists());
        ceai(&config_dir, &home)
            .arg("doctor")
            .assert()
            .failure()
            .stdout(predicate::str::contains("install-journal:"));

        // Deterministic recovery: next install rolls back the partial state,
        // proceeds fresh, preserves user content, and clears the journal.
        install(&config_dir, &home, &source);
        let cfg = read_json(&home.join(".config/opencode/opencode.json"));
        let plugins = cfg["plugin"].as_array().unwrap();
        assert!(plugins.iter().any(|v| v == "user-plugin"), "user data kept");
        assert!(plugins.len() >= 2, "CE entry re-added");
        assert!(!config_dir.join("install-journal.json").exists());

        // And doctor is clean again regarding the journal.
        ceai(&config_dir, &home)
            .arg("doctor")
            .assert()
            .stdout(predicate::str::contains("install-journal:").not());
    }

    #[test]
    fn early_fault_writes_nothing_and_still_journals_intent() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source(tmp.path());
        user_opencode_config(&home);

        ceai(&config_dir, &home)
            .env("CE_AI_FAIL_AFTER_WRITES", "0")
            .args(["install", "--harness", "opencode", "--source"])
            .arg(&source)
            .assert()
            .failure();

        // Nothing was applied: user config byte-identical, no managed tree.
        assert_eq!(
            fs::read_to_string(home.join(".config/opencode/opencode.json")).unwrap(),
            r#"{"plugin":["user-plugin"],"skills":{"paths":["/home/user/skills"]}}"#
        );
        assert!(!home
            .join(".config/opencode/compound-engineering/plugins/compound-engineering.js")
            .exists());
        assert!(config_dir.join("install-journal.json").exists());
    }

    #[test]
    fn adopt_non_interactive_report_only_and_requires_yes_to_confirm() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source_top_level_skills(tmp.path());
        install(&config_dir, &home, &source);
        // Stale local copy in the harness skills root (adoptable candidate).
        let stale = home.join(".config/opencode/skills/ce-brainstorm/SKILL.md");
        fs::create_dir_all(stale.parent().unwrap()).unwrap();
        fs::write(
            &stale,
            "---\nname: ce-brainstorm\ndescription: brainstorm\n---\n# stale local copy\n",
        )
        .unwrap();

        // Non-TTY without --yes: report-only, nothing adopted, nothing recorded.
        ceai(&config_dir, &home)
            .args(["skills", "adopt", "--harness", "opencode"])
            .write_stdin("\n")
            .assert()
            .success()
            .stdout(predicates::str::contains("pending-adoption"))
            .stdout(predicates::str::contains("ce-brainstorm — adoptable"));
        let state = read_json(&config_dir.join("state.json"));
        assert!(
            state.get("skill_surfaces").is_none()
                || state["skill_surfaces"]
                    .as_array()
                    .is_none_or(|a| a.is_empty())
        );
        assert!(fs::read_to_string(&stale)
            .unwrap()
            .contains("# stale local copy"));
    }

    #[test]
    fn adopt_reports_unrecognized_ce_dirs_and_never_touches_them() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source_top_level_skills(tmp.path());
        install(&config_dir, &home, &source);
        // User-authored ce-* skill: frontmatter name is NOT in the canonical set.
        let own = home.join(".config/opencode/skills/ce-my-own/SKILL.md");
        fs::create_dir_all(own.parent().unwrap()).unwrap();
        fs::write(&own, "---\nname: ce-my-own\ndescription: mine\n---\nbody\n").unwrap();

        // Only unrecognized dirs → nothing adoptable → report-only success.
        ceai(&config_dir, &home)
            .args(["skills", "adopt", "--harness", "opencode", "--yes"])
            .assert()
            .success()
            .stdout(predicates::str::contains(
                "unrecognized ce-* skill (frontmatter not in the canonical set)",
            ));

        assert_eq!(
            fs::read_to_string(&own).unwrap(),
            "---\nname: ce-my-own\ndescription: mine\n---\nbody\n"
        );
    }

    #[test]
    fn registry_resolves_adopted_surface_for_any_harness() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source_top_level_skills(tmp.path());
        install(&config_dir, &home, &source);
        // Stale pi-side copy (pi root: ~/.pi/agent/skills).
        let pi_skill = home.join(".pi/agent/skills/ce-brainstorm/SKILL.md");
        fs::create_dir_all(pi_skill.parent().unwrap()).unwrap();
        fs::write(
            &pi_skill,
            "---\nname: ce-brainstorm\ndescription: brainstorm\n---\n# stale pi copy\n",
        )
        .unwrap();

        ceai(&config_dir, &home)
            .args(["skills", "adopt", "--harness", "pi", "--yes"])
            .assert()
            .success()
            .stdout(predicates::str::contains("adopted under ce-ai management"));

        // AE6: resolution serves the adopted root (not ~/.ce-ai/harness-pi).
        ceai(&config_dir, &home)
            .args([
                "skills",
                "resolve",
                "--harness",
                "pi",
                "--query",
                "brainstorm",
            ])
            .assert()
            .success()
            .stdout(predicates::str::contains(
                std::path::Path::new(".pi")
                    .join("agent")
                    .join("skills")
                    .join("ce-brainstorm")
                    .join("SKILL.md")
                    .to_string_lossy()
                    .to_string(),
            ));
    }

    #[test]
    fn doctor_reports_orphaned_adopted_surface() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source_top_level_skills(tmp.path());
        install(&config_dir, &home, &source);
        stale_local_copy(&home);
        ceai(&config_dir, &home)
            .args(["skills", "adopt", "--harness", "opencode", "--yes"])
            .assert()
            .success();
        fs::remove_dir_all(home.join(".config/opencode/skills")).unwrap();

        ceai(&config_dir, &home)
            .args(["skills", "doctor"])
            .assert()
            .failure()
            .stdout(predicates::str::contains(
                "orphaned adopted skills surface for opencode",
            ));
    }

    #[test]
    fn status_reports_adopted_surface() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source_top_level_skills(tmp.path());
        install(&config_dir, &home, &source);
        stale_local_copy(&home);
        ceai(&config_dir, &home)
            .args(["skills", "adopt", "--harness", "opencode", "--yes"])
            .assert()
            .success();

        ceai(&config_dir, &home)
            .args(["status"])
            .assert()
            .success()
            .stdout(predicates::str::contains("skills: opencode adopted"));
    }

    #[test]
    fn adopt_then_sync_reports_verified_adopted_surface() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source_top_level_skills(tmp.path());
        install(&config_dir, &home, &source);
        stale_local_copy(&home);
        ceai(&config_dir, &home)
            .args(["skills", "adopt", "--harness", "opencode", "--yes"])
            .assert()
            .success();

        ceai(&config_dir, &home)
            .args(["sync"])
            .assert()
            .success()
            .stdout(predicates::str::contains("opencode: verified"));
    }

    #[test]
    fn sync_rewrites_adopted_surface_and_reports_restored_drift() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source_top_level_skills(tmp.path());
        install(&config_dir, &home, &source);
        stale_local_copy(&home);
        ceai(&config_dir, &home)
            .args(["skills", "adopt", "--harness", "opencode", "--yes"])
            .assert()
            .success();

        // User edit after adoption = drift; sync restores canonical (R16).
        let adopted = stale_local_copy_path(&home);
        fs::write(&adopted, "# my local tweak\n").unwrap();
        ceai(&config_dir, &home)
            .args(["sync"])
            .assert()
            .success()
            .stdout(predicates::str::contains("restored-drift"));
        assert_eq!(fs::read_to_string(&adopted).unwrap(), "# ce-brainstorm\n");
    }

    #[test]
    fn sync_matrix_reports_pending_adoption_for_untracked_surface() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source_top_level_skills(tmp.path());
        install(&config_dir, &home, &source);
        stale_local_copy(&home);

        ceai(&config_dir, &home)
            .args(["sync"])
            .assert()
            .success()
            .stdout(predicates::str::contains("pending-adoption"));
    }

    #[test]
    fn sync_matrix_reports_orphaned_when_adopted_root_vanishes() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source_top_level_skills(tmp.path());
        install(&config_dir, &home, &source);
        stale_local_copy(&home);
        ceai(&config_dir, &home)
            .args(["skills", "adopt", "--harness", "opencode", "--yes"])
            .assert()
            .success();
        fs::remove_dir_all(home.join(".config/opencode/skills")).unwrap();

        ceai(&config_dir, &home)
            .args(["sync"])
            .assert()
            .success()
            .stdout(predicates::str::contains("orphaned"));
        let state = read_json(&config_dir.join("state.json"));
        assert_eq!(state["skill_surfaces"][0]["status"], "orphaned");
    }

    #[test]
    fn sync_reports_external_duplicate_from_plugin_cache() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
        let source = ce_source_top_level_skills(tmp.path());
        install(&config_dir, &home, &source);
        let cache_hit =
            home.join(".claude/plugins/cache/compound-engineering-plugin/skills/ce-x/SKILL.md");
        fs::create_dir_all(cache_hit.parent().unwrap()).unwrap();
        fs::write(&cache_hit, "# ce-x\n").unwrap();

        ceai(&config_dir, &home)
            .args(["sync"])
            .assert()
            .success()
            .stdout(predicates::str::contains("external-duplicate"));
    }

    #[test]
    fn guard_enable_disable_and_status_cli_lifecycle() {
        let tmp = TempDir::new().unwrap();
        let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));

        // Status on fresh directory
        ceai(&config_dir, &home)
            .args(["guard", "status"])
            .assert()
            .success()
            .stdout(predicates::str::contains("Disabled"));

        // Enable default junior
        ceai(&config_dir, &home)
            .args(["guard", "enable"])
            .assert()
            .success()
            .stdout(predicates::str::contains("enabled"));

        // Status confirms enabled
        ceai(&config_dir, &home)
            .args(["guard", "status"])
            .assert()
            .success()
            .stdout(predicates::str::contains("Enabled"))
            .stdout(predicates::str::contains("junior"));

        // JSON status
        ceai(&config_dir, &home)
            .args(["guard", "status", "--json"])
            .assert()
            .success()
            .stdout(predicates::str::contains("\"enabled\": true"))
            .stdout(predicates::str::contains("\"level\": \"junior\""));

        // Enable strict with harness
        ceai(&config_dir, &home)
            .args([
                "guard",
                "enable",
                "--level",
                "strict",
                "--harness",
                "claude",
            ])
            .assert()
            .success();

        ceai(&config_dir, &home)
            .args(["guard", "status"])
            .assert()
            .success()
            .stdout(predicates::str::contains("strict"))
            .stdout(predicates::str::contains("claude"));

        // Disable
        ceai(&config_dir, &home)
            .args(["guard", "disable"])
            .assert()
            .success()
            .stdout(predicates::str::contains("disabled"));

        // Invalid level fails with usage code 2
        ceai(&config_dir, &home)
            .args(["guard", "enable", "--level", "invalid-level"])
            .assert()
            .code(2)
            .stderr(predicates::str::contains("invalid guard level"));
    }
}

#[test]
fn init_prj_claude_injects_and_deinits_session_start_hook() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("claude-project");
    let claude_dir = prj_dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let settings_file = claude_dir.join("settings.json");
    assert!(settings_file.exists());
    let content = fs::read_to_string(&settings_file).unwrap();
    assert!(content.contains("\"SessionStart\""));
    assert!(content.contains("ce-ai workflow resume"));

    // Verify doctor is happy with the hook present
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("Claude Code SessionStart hook missing").not());

    // Tamper by removing the hook: doctor must report finding
    fs::write(&settings_file, "{}").unwrap();
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains(
            "project-adoption: Claude Code SessionStart hook missing",
        ));

    // Re-run init-prj repairs the hook
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();
    let repaired = fs::read_to_string(&settings_file).unwrap();
    assert!(repaired.contains("ce-ai workflow resume"));

    // De-init cleans up the hook and the empty settings file
    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();
    assert!(!settings_file.exists());
}

#[test]
fn init_prj_full_tier_contains_turn_zero_directive() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("turn-zero-project");
    fs::create_dir_all(&prj_dir).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let agents_text = fs::read_to_string(prj_dir.join("AGENTS.md")).unwrap();
    assert!(agents_text.contains("### ⚡ Turn-0 Session Directives (Zero-Step Drift Recovery)"));
    assert!(agents_text.contains("ce-ai workflow resume"));
}

#[test]
fn doctor_reports_and_sync_repairs_missing_opencode_plugin() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    install(&config_dir, &home, &source);

    // Verify doctor is initially ok
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("opencode: SessionStart plugin missing").not());

    // Tamper: remove plugin from opencode.json
    let opencode_json = home.join(".config/opencode/opencode.json");
    fs::write(&opencode_json, "{\"plugin\": []}").unwrap();

    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains(
            "opencode: SessionStart plugin missing or outdated",
        ));

    // Sync repairs the missing plugin entry
    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .arg("sync")
        .assert()
        .success();

    ceai(&config_dir, &home)
        .current_dir(tmp.path())
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("opencode: SessionStart plugin missing").not());
}

#[test]
fn init_prj_copilot_injects_and_deinits_session_start_hook() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("copilot-project");
    let github_dir = prj_dir.join(".github");
    fs::create_dir_all(&github_dir).unwrap();

    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let hooks_file = github_dir.join("hooks/hooks.json");
    assert!(hooks_file.exists());
    let content = fs::read_to_string(&hooks_file).unwrap();
    assert!(content.contains("\"sessionStart\""));
    assert!(content.contains("ce-ai workflow resume --json"));

    // Verify doctor is happy with the hook present
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("Copilot CLI sessionStart hook missing").not());

    // Tamper by removing the hook: doctor must report finding
    fs::write(&hooks_file, "{\"version\": 1}").unwrap();
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains(
            "project-adoption: Copilot CLI sessionStart hook missing",
        ));

    // Re-run init-prj repairs the hook
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();
    let repaired = fs::read_to_string(&hooks_file).unwrap();
    assert!(repaired.contains("ce-ai workflow resume --json"));

    // Verify ce-ai workflow resume --json contains additionalContext
    ceai(&config_dir, &home)
        .current_dir(&prj_dir)
        .args(["workflow", "resume", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"additionalContext\""));

    // De-init cleans up the hook and the empty hooks file
    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();
    assert!(!hooks_file.exists());
}

#[test]
fn init_prj_codex_injects_and_deinits_session_start_hook() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("codex-project");
    fs::create_dir_all(&prj_dir).unwrap();

    // Create a mock git repository with .codex directory
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&prj_dir)
        .output()
        .unwrap();

    let codex_dir = prj_dir.join(".codex");
    fs::create_dir_all(&codex_dir).unwrap();

    // Adopt project
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let config_file = codex_dir.join("config.toml");
    assert!(config_file.exists());
    let content = fs::read_to_string(&config_file).unwrap();
    assert!(content.contains("[[hooks.SessionStart]]"));
    assert!(content.contains("ce-ai workflow resume"));

    // Doctor checks it and finds no issue
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("Codex CLI SessionStart hook missing").not());

    // Tamper by removing the hook: doctor must report finding
    fs::write(&config_file, "model = \"o3-mini\"\n").unwrap();
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains(
            "project-adoption: Codex CLI SessionStart hook missing",
        ));

    // Re-run init-prj repairs the hook
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();
    let repaired = fs::read_to_string(&config_file).unwrap();
    assert!(repaired.contains("ce-ai workflow resume"));
    assert!(repaired.contains("model = \"o3-mini\""));

    // Verify ce-ai workflow resume --json contains hookSpecificOutput
    ceai(&config_dir, &home)
        .current_dir(&prj_dir)
        .args(["workflow", "resume", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"hookSpecificOutput\""));

    // De-init cleans up the hook, preserving other config settings
    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();
    let after_deinit = fs::read_to_string(&config_file).unwrap();
    assert!(!after_deinit.contains("ce-ai workflow resume"));
    assert!(after_deinit.contains("model = \"o3-mini\""));
}

#[test]
fn init_prj_pi_injects_and_deinits_session_start_hook() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let prj_dir = tmp.path().join("pi-project");
    fs::create_dir_all(&prj_dir).unwrap();

    // Create a mock git repository with .pi directory
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&prj_dir)
        .output()
        .unwrap();

    let pi_dir = prj_dir.join(".pi");
    fs::create_dir_all(&pi_dir).unwrap();

    // Adopt project
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let ext_file = pi_dir.join("extensions").join("compound-engineering.ts");
    assert!(ext_file.exists());
    let content = fs::read_to_string(&ext_file).unwrap();
    assert!(content.contains("before_agent_start"));
    assert!(content.contains("ce-ai workflow resume"));

    // Doctor checks it and finds no issue
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("Pi before_agent_start extension missing").not());

    // Tamper by removing the hook: doctor must report finding
    fs::remove_file(&ext_file).unwrap();
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains(
            "project-adoption: Pi before_agent_start extension missing",
        ));

    // Re-run init-prj repairs the extension
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();
    assert!(ext_file.exists());
    let repaired = fs::read_to_string(&ext_file).unwrap();
    assert!(repaired.contains("ce-ai workflow resume"));

    // De-init cleans up the extension and prunes empty directories
    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();
    assert!(!ext_file.exists());
}

#[test]
fn init_prj_cursor_injects_and_deinits_session_start_hook() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let config_dir = home.join(".config").join("opencode");
    let prj_dir = tmp.path().join("cursor_prj");
    fs::create_dir_all(&prj_dir).unwrap();

    // Initialize git repository
    std::process::Command::new("git")
        .args(["init", prj_dir.to_str().unwrap()])
        .output()
        .unwrap();

    // Pre-create .cursor directory so init-prj triggers Cursor adoption
    let cursor_dir = prj_dir.join(".cursor");
    fs::create_dir_all(&cursor_dir).unwrap();

    // Adopt project with full tier
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let hooks_file = cursor_dir.join("hooks.json");
    let rule_file = cursor_dir.join("rules").join("compound-engineering.mdc");
    assert!(hooks_file.exists(), ".cursor/hooks.json must be created");
    assert!(
        rule_file.exists(),
        ".cursor/rules/compound-engineering.mdc must be created"
    );

    let hooks_content = fs::read_to_string(&hooks_file).unwrap();
    let hooks_val: serde_json::Value = serde_json::from_str(&hooks_content).unwrap();
    assert_eq!(hooks_val["version"], 1);
    let session_start = hooks_val["hooks"]["sessionStart"].as_array().unwrap();
    assert_eq!(session_start.len(), 1);
    assert_eq!(session_start[0]["command"], "ce-ai workflow resume --json");

    // Doctor check should pass cleanly without Cursor hook findings
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("Cursor sessionStart hook missing").not());

    // Tamper by removing the hook: doctor must report finding
    fs::remove_file(&hooks_file).unwrap();
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains(
            "project-adoption: Cursor sessionStart hook missing",
        ));

    // Re-run init-prj repairs the hook
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();
    assert!(hooks_file.exists());
    let repaired = fs::read_to_string(&hooks_file).unwrap();
    assert!(repaired.contains("ce-ai workflow resume --json"));

    // De-init cleans up the hooks and rules
    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();
    assert!(
        !hooks_file.exists(),
        ".cursor/hooks.json must be removed on deinit"
    );
    assert!(
        !rule_file.exists(),
        "compound-engineering.mdc must be removed on deinit"
    );
}

#[test]
fn init_prj_agy_injects_and_deinits_pre_invocation_hook() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let config_dir = home.join(".config").join("opencode");
    let prj_dir = tmp.path().join("agy_prj");
    fs::create_dir_all(&prj_dir).unwrap();

    // Initialize git repository
    std::process::Command::new("git")
        .args(["init", prj_dir.to_str().unwrap()])
        .output()
        .unwrap();

    // Pre-create .agents directory so init-prj triggers Antigravity adoption
    let agents_dir = prj_dir.join(".agents");
    fs::create_dir_all(&agents_dir).unwrap();

    // Adopt project with full tier
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();

    let hooks_file = agents_dir.join("hooks.json");
    let rule_file = agents_dir.join("rules").join("compound-engineering.md");
    assert!(hooks_file.exists(), ".agents/hooks.json must be created");
    assert!(
        rule_file.exists(),
        ".agents/rules/compound-engineering.md must be created"
    );

    let hooks_content = fs::read_to_string(&hooks_file).unwrap();
    let hooks_val: serde_json::Value = serde_json::from_str(&hooks_content).unwrap();
    let pre_inv = hooks_val["compound-engineering"]["PreInvocation"]
        .as_array()
        .unwrap();
    assert_eq!(pre_inv.len(), 1);
    assert_eq!(pre_inv[0]["type"], "command");
    assert_eq!(
        pre_inv[0]["command"],
        "ce-ai workflow resume --pre-invocation"
    );

    // Doctor check should pass cleanly without Antigravity hook findings
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains("Antigravity PreInvocation hook missing").not());

    // Tamper by removing the hook: doctor must report finding
    fs::remove_file(&hooks_file).unwrap();
    ceai(&config_dir, &home)
        .arg("doctor")
        .assert()
        .stdout(predicate::str::contains(
            "project-adoption: Antigravity PreInvocation hook missing",
        ));

    // Re-run init-prj repairs the hook
    ceai(&config_dir, &home)
        .args(["init-prj", prj_dir.to_str().unwrap(), "--tier", "full"])
        .assert()
        .success();
    assert!(hooks_file.exists());
    let repaired = fs::read_to_string(&hooks_file).unwrap();
    assert!(repaired.contains("ce-ai workflow resume --pre-invocation"));

    // De-init cleans up the hooks and rules
    ceai(&config_dir, &home)
        .args(["deinit-prj", prj_dir.to_str().unwrap()])
        .assert()
        .success();
    assert!(
        !hooks_file.exists(),
        ".agents/hooks.json must be removed on deinit"
    );
    assert!(
        !rule_file.exists(),
        "compound-engineering.md must be removed on deinit"
    );
}

#[test]
fn workflow_resume_pre_invocation_cli_deduplication() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let config_dir = home.join(".config").join("opencode");

    // First invocation with conversationId returns injectSteps
    let input1 = r#"{"conversationId": "test-agy-cli-conv-1", "invocationNum": 0}"#;
    let assert1 = ceai(&config_dir, &home)
        .env("CE_AI_AGY_MARKER_DIR", tmp.path())
        .args(["workflow", "resume", "--pre-invocation"])
        .write_stdin(input1)
        .assert()
        .success();

    let stdout1 = String::from_utf8_lossy(&assert1.get_output().stdout);
    assert!(stdout1.contains("injectSteps"));
    assert!(stdout1.contains("ephemeralMessage"));

    // Second invocation with same conversationId returns {}
    let input2 = r#"{"conversationId": "test-agy-cli-conv-1", "invocationNum": 1}"#;
    let assert2 = ceai(&config_dir, &home)
        .env("CE_AI_AGY_MARKER_DIR", tmp.path())
        .args(["workflow", "resume", "--pre-invocation"])
        .write_stdin(input2)
        .assert()
        .success();

    let stdout2 = String::from_utf8_lossy(&assert2.get_output().stdout);
    let trimmed2 = stdout2.trim();
    assert_eq!(trimmed2, "{}");
}

#[test]
fn workflow_checkpoint_reset_to_stage_1_clears_feature_in_resume() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let config_dir = home.join(".config").join("opencode");
    let repo_dir = tmp.path().join("my-repo");
    fs::create_dir_all(&repo_dir).unwrap();

    std::process::Command::new("git")
        .args(["init", repo_dir.to_str().unwrap()])
        .output()
        .unwrap();

    let feat_a_dir = repo_dir.join("openspec").join("changes").join("feature-a");
    let feat_b_dir = repo_dir.join("openspec").join("changes").join("feature-b");
    fs::create_dir_all(&feat_a_dir).unwrap();
    fs::write(feat_a_dir.join("proposal.md"), "# Feature A").unwrap();

    // 1. Checkpoint on Stage 2 with explicit feature-a
    ceai(&config_dir, &home)
        .current_dir(&repo_dir)
        .args([
            "workflow",
            "checkpoint",
            "--stage",
            "2",
            "--task",
            "spec feature-a",
            "--feature",
            "feature-a",
        ])
        .assert()
        .success();

    // Ensure filesystem timestamp ticks forward before creating feature-b
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::create_dir_all(&feat_b_dir).unwrap();
    fs::write(feat_b_dir.join("proposal.md"), "# Feature B").unwrap();

    // 2. Reset to Stage 1 without --feature
    ceai(&config_dir, &home)
        .current_dir(&repo_dir)
        .args([
            "workflow",
            "checkpoint",
            "--stage",
            "1",
            "--task",
            "ideation for feature-b",
        ])
        .assert()
        .success();

    // 3. Resume must NOT retain feature-a; must fall back to discovery of feature-b
    let assert_resume = ceai(&config_dir, &home)
        .current_dir(&repo_dir)
        .args(["workflow", "resume"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert_resume.get_output().stdout);
    assert!(
        !stdout.contains("active feature: feature-a"),
        "stdout must not retain active feature-a after reset to stage 1: {stdout}"
    );
    assert!(
        !stdout.contains("Context Re-hydration: feature-a"),
        "re-hydration must not bind to old feature-a after reset to stage 1: {stdout}"
    );
    assert!(
        stdout.contains("Context Re-hydration: feature-b"),
        "re-hydration should discover feature-b fallback: {stdout}"
    );
}

#[test]
fn doctor_workspace_scope_opencode_install_has_no_false_positive_findings() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let config_dir = home.join(".ce-ai");
    let repo_dir = tmp.path().join("repo");
    let source = ce_source(tmp.path());

    fs::create_dir_all(&repo_dir).unwrap();
    std::process::Command::new("git")
        .args(["init", "-q", repo_dir.to_str().unwrap()])
        .output()
        .unwrap();

    // Install with --scope workspace
    ceai(&config_dir, &home)
        .current_dir(&repo_dir)
        .args([
            "install",
            "--harness",
            "opencode",
            "--scope",
            "workspace",
            "--source",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    // ce-ai doctor inside the workspace must NOT report state-inconsistent or missing SessionStart plugin
    let assert_doctor = ceai(&config_dir, &home)
        .current_dir(&repo_dir)
        .args(["doctor"])
        .assert();

    let output = assert_doctor.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !stdout.contains("state-inconsistent"),
        "doctor stdout must not report state-inconsistent for workspace scope: {stdout}"
    );
    assert!(
        !stdout.contains("SessionStart plugin missing"),
        "doctor stdout must not report missing SessionStart plugin for workspace scope: {stdout}"
    );
    assert!(
        !stderr.contains("state-inconsistent"),
        "doctor stderr must not report state-inconsistent: {stderr}"
    );

    // ce-ai status inside workspace must report drift: none, not "unknown (no install manifest)"
    let assert_status = ceai(&config_dir, &home)
        .current_dir(&repo_dir)
        .args(["status"])
        .assert()
        .success();

    let status_out = String::from_utf8_lossy(&assert_status.get_output().stdout);
    assert!(
        status_out.contains("drift: none"),
        "status stdout must report drift: none: {status_out}"
    );
    assert!(
        !status_out.contains("drift: unknown"),
        "status stdout must not report unknown drift: {status_out}"
    );

    // ce-ai sync inside workspace must succeed without error
    ceai(&config_dir, &home)
        .current_dir(&repo_dir)
        .args(["sync"])
        .assert()
        .success();
}

#[test]
fn sync_standalone_claude_harness_without_opencode_succeeds() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source_top_level_skills(tmp.path());

    // Create marker only for claude — opencode is NOT present
    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::write(home.join(".claude/settings.json"), "{}").unwrap();

    // Install only claude
    ceai(&config_dir, &home)
        .args(["install", "--harness", "claude", "--source"])
        .arg(&source)
        .assert()
        .success();

    // ce-ai sync must succeed without requiring opencode manifest
    ceai(&config_dir, &home)
        .args(["sync"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "○ claude: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)",
        ))
        .stdout(predicates::str::contains("0 failed"))
        .stdout(predicates::str::contains("opencode").not());
}

#[test]
fn upgrade_standalone_claude_harness_without_opencode_succeeds() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source1 = ce_source_top_level_skills(tmp.path());

    fs::create_dir_all(home.join(".claude")).unwrap();
    fs::write(home.join(".claude/settings.json"), "{}").unwrap();

    ceai(&config_dir, &home)
        .args(["install", "--harness", "claude", "--source"])
        .arg(&source1)
        .assert()
        .success();

    let source2 = tmp.path().join("source2");
    fs::create_dir_all(source2.join("skills/ce-brainstorm")).unwrap();
    fs::write(source2.join("skills/ce-brainstorm/SKILL.md"), "# v2\n").unwrap();
    fs::create_dir_all(source2.join(".opencode/plugins")).unwrap();
    fs::write(
        source2.join(".opencode/plugins/compound-engineering.js"),
        "// v2\n",
    )
    .unwrap();

    // ce-ai upgrade --source must succeed without requiring opencode manifest
    ceai(&config_dir, &home)
        .args(["upgrade", "--source"])
        .arg(&source2)
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "○ claude: registered — ce-ai manages no skill files here (MCP companions only; nothing to hash-verify)",
        ));
}

#[test]
fn sync_with_no_harnesses_installed_fails_with_clear_error() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));

    ceai(&config_dir, &home)
        .args(["sync"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            "no harnesses installed — run ce-ai install first",
        ));
}

#[test]
fn sync_standalone_custom_harness_without_opencode_succeeds() {
    let tmp = TempDir::new().unwrap();
    let (config_dir, home) = (tmp.path().join("ce-ai"), tmp.path().join("home"));
    let source = ce_source(tmp.path());
    let (plugins, skills, rules) = (
        home.join("harness/plugins"),
        home.join("harness/skills"),
        home.join("harness/rules.md"),
    );
    fs::create_dir_all(rules.parent().unwrap()).unwrap();
    fs::write(&rules, "# my harness rules\n").unwrap();

    ceai(&config_dir, &home)
        .args(["install", "--harness", "custom", "--plugins-dir"])
        .arg(&plugins)
        .arg("--skills-dir")
        .arg(&skills)
        .arg("--rules-file")
        .arg(&rules)
        .arg("--source")
        .arg(&source)
        .assert()
        .success();

    ceai(&config_dir, &home)
        .args(["sync"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "✓ custom: verified — 2/2 managed files match SHA256",
        ))
        .stdout(predicates::str::contains("0 failed"))
        .stdout(predicates::str::contains("opencode").not());
}
