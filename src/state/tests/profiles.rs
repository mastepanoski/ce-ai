use std::collections::BTreeMap;
use tempfile::tempdir;

use crate::state::profiles::{load_profile, save_profile, save_snapshot, Profile};

fn models() -> BTreeMap<String, String> {
    BTreeMap::from([(
        "ce-brainstorm".to_string(),
        "opencode-go/kimi-k2.6".to_string(),
    )])
}

#[test]
fn named_profile_round_trip() {
    let dir = tempdir().unwrap();
    let profile = Profile {
        name: "fast".into(),
        created_at: "2026-08-20T00:00:00Z".into(),
        models: models(),
    };
    save_profile(dir.path(), &profile).unwrap();
    assert_eq!(load_profile(dir.path(), "fast").unwrap(), profile);
}

#[test]
fn load_missing_profile_is_error() {
    let dir = tempdir().unwrap();
    assert!(load_profile(dir.path(), "nope").is_err());
}

#[test]
fn snapshots_are_append_only() {
    let dir = tempdir().unwrap();
    let before = models();
    let first = save_snapshot(dir.path(), "fast", &before, &before).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let second = save_snapshot(dir.path(), "fast", &before, &before).unwrap();
    assert_ne!(first, second);
    let version_count = std::fs::read_dir(dir.path().join("versions"))
        .unwrap()
        .count();
    assert_eq!(version_count, 2);
    let kept: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.path().join("versions").join(first)).unwrap(),
    )
    .unwrap();
    assert_eq!(kept["name"], "fast");
}
