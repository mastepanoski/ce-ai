use std::fs;
use std::path::Path;
use tempfile::tempdir;

use crate::state::backups::{backup_file, list_backups, restore_backup_by_id, restore_latest};

fn write_file(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn backup_creates_timestamped_dir_with_copy() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("opencode.json");
    write_file(dir.path(), "opencode.json", r#"{"version":1}"#);
    let dest = backup_file(&dir.path().join("backups"), &source).unwrap();
    assert!(dest.starts_with(dir.path().join("backups")));
    assert_eq!(fs::read_to_string(&dest).unwrap(), r#"{"version":1}"#);
    assert_eq!(fs::read_to_string(&source).unwrap(), r#"{"version":1}"#);
}

#[test]
fn list_backups_returns_sorted_entries_with_harness_filter() {
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    write_file(&backups, "20260821T000001Z/opencode.json", r#"{"a":1}"#);
    write_file(&backups, "20260821T000002Z/claude.json", r#"{"b":2}"#);

    let all = list_backups(&backups, None).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].id, "20260821T000002Z");
    assert_eq!(all[0].harness, "claude");
    assert_eq!(all[1].id, "20260821T000001Z");
    assert_eq!(all[1].harness, "opencode");

    let filtered = list_backups(&backups, Some("opencode")).unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].harness, "opencode");
}

#[test]
fn restore_backup_by_id_restores_snapshot_and_rejects_path_traversal() {
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    let target = dir.path().join("opencode.json");

    write_file(
        &backups,
        "20260821T000001Z/opencode.json",
        r#"{"state":"v1"}"#,
    );
    write_file(
        &backups,
        "20260821T000002Z/opencode.json",
        r#"{"state":"v2"}"#,
    );
    write_file(dir.path(), "opencode.json", r#"{"state":"current"}"#);

    let restored = restore_backup_by_id(&backups, "20260821T000001Z", &target).unwrap();
    assert_eq!(restored.id, "20260821T000001Z");
    assert_eq!(fs::read_to_string(&target).unwrap(), r#"{"state":"v1"}"#);

    // Path Traversal Security Check
    assert!(restore_backup_by_id(&backups, "../etc/passwd", &target).is_err());
}

#[test]
fn restore_latest_restores_newest_backup() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("opencode.json");
    let backups = dir.path().join("backups");
    write_file(dir.path(), "opencode.json", r#"{"v":"old"}"#);
    backup_file(&backups, &source).unwrap();
    write_file(dir.path(), "opencode.json", r#"{"v":"new"}"#);
    std::thread::sleep(std::time::Duration::from_millis(5));
    backup_file(&backups, &source).unwrap();
    write_file(dir.path(), "opencode.json", r#"{"v":"corrupted"}"#);
    restore_latest(&backups, &source).unwrap();
    assert_eq!(fs::read_to_string(&source).unwrap(), r#"{"v":"new"}"#);
}

#[test]
fn restore_latest_without_backups_is_error() {
    let dir = tempdir().unwrap();
    let backups = dir.path().join("backups");
    assert!(restore_latest(&backups, &dir.path().join("opencode.json")).is_err());
}
