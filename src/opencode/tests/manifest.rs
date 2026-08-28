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
