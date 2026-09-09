use tempfile::tempdir;

use super::*;

fn sample_manifest() -> InstallManifest {
    InstallManifest {
        version: "compound-engineering-v3.4.2".into(),
        plugin_name: "compound-engineering".into(),
        installed_at: "2026-08-20T00:00:00Z".into(),
        source: serde_json::json!({ "kind": "local", "path": "/tmp/ce" }),
        files: vec![
            ManifestFile {
                path: "plugins/compound-engineering.js".into(),
                sha256: "a".repeat(64),
            },
            ManifestFile {
                path: "skills/ce-brainstorm/SKILL.md".into(),
                sha256: "b".repeat(64),
            },
        ],
        config_mutations: vec![ConfigMutation {
            file: "opencode.json".into(),
            backup: None,
            keys: vec!["plugin".into(), "skills.paths".into()],
        }],
    }
}

#[test]
fn manifest_round_trips_with_per_file_sha256() {
    let dir = tempdir().unwrap();
    let manifest = sample_manifest();
    manifest.write(dir.path()).unwrap();

    let loaded = InstallManifest::load(dir.path()).unwrap();
    assert_eq!(loaded, manifest);
    assert_eq!(loaded.files[0].sha256, "a".repeat(64));
    assert_eq!(loaded.files[1].sha256, "b".repeat(64));
}

#[test]
fn manifest_written_under_managed_dir() {
    let dir = tempdir().unwrap();
    sample_manifest().write(dir.path()).unwrap();
    assert!(dir
        .path()
        .join("compound-engineering/install-manifest.json")
        .exists());
}

#[test]
fn load_missing_manifest_errors() {
    let dir = tempdir().unwrap();
    assert!(InstallManifest::load(dir.path()).is_err());
}

#[test]
fn harvest_collects_sha256_for_all_managed_files() {
    let dir = tempdir().unwrap();
    let managed = dir.path().join("compound-engineering");
    std::fs::create_dir_all(managed.join("plugins")).unwrap();
    std::fs::create_dir_all(managed.join("skills/ce-brainstorm")).unwrap();
    std::fs::write(managed.join("plugins/compound-engineering.js"), b"loader").unwrap();
    std::fs::write(managed.join("skills/ce-brainstorm/SKILL.md"), b"# skill").unwrap();
    // install-manifest.json itself must never be harvested.
    std::fs::write(managed.join("install-manifest.json"), b"{}").unwrap();

    let files = InstallManifest::harvest(&managed);
    assert_eq!(files.len(), 2);
    // Deterministic path-sorted order.
    assert_eq!(files[0].path, "plugins/compound-engineering.js");
    assert_eq!(files[1].path, "skills/ce-brainstorm/SKILL.md");
    assert_eq!(files[0].sha256, crate::state::diff::sha256_hex(b"loader"));
    assert_eq!(files[1].sha256, crate::state::diff::sha256_hex(b"# skill"));
}

#[test]
fn harvest_missing_dir_returns_empty() {
    let dir = tempdir().unwrap();
    assert!(InstallManifest::harvest(&dir.path().join("nope")).is_empty());
}
