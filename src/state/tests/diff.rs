use crate::state::diff::{diff, sha256_hex, Action};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn write_file(root: &Path, rel: &str, content: &[u8]) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn missing_desired_file_plans_copy() {
    let dir = tempdir().unwrap();
    let desired = BTreeMap::from([("plugins/ce.js".to_string(), sha256_hex(b"loader"))]);
    let plan = diff(&desired, &BTreeMap::new(), dir.path());
    assert_eq!(
        plan.actions,
        vec![Action::Copy {
            path: "plugins/ce.js".into()
        }]
    );
}

#[test]
fn modified_file_plans_restore() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "plugins/ce.js", b"tampered");
    let desired = BTreeMap::from([("plugins/ce.js".to_string(), sha256_hex(b"loader"))]);
    let plan = diff(&desired, &desired, dir.path());
    assert_eq!(
        plan.actions,
        vec![Action::Restore {
            path: "plugins/ce.js".into()
        }]
    );
}

#[test]
fn stale_managed_file_plans_remove() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "plugins/old.js", b"old");
    let manifest = BTreeMap::from([("plugins/old.js".to_string(), sha256_hex(b"old"))]);
    let plan = diff(&BTreeMap::new(), &manifest, dir.path());
    assert_eq!(
        plan.actions,
        vec![Action::Remove {
            path: "plugins/old.js".into()
        }]
    );
}

#[test]
fn up_to_date_files_plan_nothing() {
    let dir = tempdir().unwrap();
    write_file(dir.path(), "plugins/ce.js", b"loader");
    let desired = BTreeMap::from([("plugins/ce.js".to_string(), sha256_hex(b"loader"))]);
    assert!(diff(&desired, &desired, dir.path()).actions.is_empty());
}

#[test]
fn diff_plans_without_writing() {
    let dir = tempdir().unwrap();
    let desired = BTreeMap::from([("plugins/missing.js".to_string(), sha256_hex(b"x"))]);
    diff(&desired, &BTreeMap::new(), dir.path());
    assert!(!dir.path().join("plugins/missing.js").exists());
}
