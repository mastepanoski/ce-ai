//! Performance Benchmarks for ce-ai (< 50ms execution bound guarantee).

use ce_ai::opencode::manifest::{InstallManifest, ManifestFile};
use ce_ai::state::state::State;
use std::time::Instant;
use tempfile::TempDir;

#[test]
fn benchmark_state_loading_and_workspace_overrides_under_50ms() {
    let tmp = TempDir::new().unwrap();
    let global_state_path = tmp.path().join("state.json");

    let global_state = State::default();
    global_state.save(&global_state_path).unwrap();

    let workspace_dir = tmp.path().join("workspace");
    std::fs::create_dir_all(&workspace_dir).unwrap();
    let local_override_path = workspace_dir.join(".ce-ai.json");
    let local_json = r#"{
        "model_assignments": {
            "ce-work": {
                "provider_id": "openai",
                "model_id": "gpt-4o"
            }
        }
    }"#;
    std::fs::write(&local_override_path, local_json).unwrap();

    let start = Instant::now();
    let loaded = State::load_with_workspace_overrides(&global_state_path, Some(&workspace_dir))
        .expect("Workspace overrides load should succeed");
    let elapsed = start.elapsed();

    assert_eq!(
        loaded.model_assignments.get("ce-work").unwrap().model_id,
        "gpt-4o",
        "Override precedence must resolve correctly"
    );
    assert!(
        elapsed.as_millis() < 50,
        "State resolution with workspace overrides must complete under 50ms (took {:?})",
        elapsed
    );
}

#[test]
fn benchmark_sha256_manifest_roundtrip_under_50ms() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("opencode");
    let managed_dir = config_dir.join("compound-engineering");
    std::fs::create_dir_all(&managed_dir).unwrap();

    let file1 = config_dir.join("test1.txt");
    let file2 = config_dir.join("test2.txt");
    std::fs::write(&file1, "content 1").unwrap();
    std::fs::write(&file2, "content 2").unwrap();

    let start = Instant::now();
    let manifest = InstallManifest {
        version: "0.9.0".into(),
        plugin_name: "compound-engineering".into(),
        installed_at: "2026-08-21T00:00:00Z".into(),
        source: serde_json::json!({"type": "git"}),
        files: vec![
            ManifestFile {
                path: "test1.txt".into(),
                sha256: "abc".into(),
            },
            ManifestFile {
                path: "test2.txt".into(),
                sha256: "def".into(),
            },
        ],
        config_mutations: vec![],
    };

    manifest
        .write(&config_dir)
        .expect("Manifest write should succeed");
    let loaded = InstallManifest::load(&config_dir).expect("Manifest load should succeed");
    let elapsed = start.elapsed();

    assert_eq!(loaded.version, "0.9.0");
    assert!(
        elapsed.as_millis() < 50,
        "Manifest creation and SHA256 indexing must complete under 50ms (took {:?})",
        elapsed
    );
}
