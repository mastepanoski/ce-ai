//! Sync diff engine: plan copy/restore/remove actions without writing (SU-1..SU-4).

use std::collections::BTreeMap;
use std::path::Path;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Desired file missing on disk — copy from source.
    Copy { path: String },
    /// On-disk hash differs from manifest — restore from source.
    Restore { path: String },
    /// Managed file no longer desired — remove.
    Remove { path: String },
}

/// Planned actions reconciling desired vs manifest vs filesystem.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diff {
    pub actions: Vec<Action>,
}

/// SHA-256 hex digest of raw bytes.
pub fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

/// Plans actions to reconcile desired against manifest and disk; never writes.
pub fn diff(desired: &BTreeMap<String, String>, manifest: &BTreeMap<String, String>, fs_root: &Path) -> Diff {
    let mut actions = Vec::new();
    for (path, want_hash) in desired {
        match std::fs::read(fs_root.join(path)) {
            Ok(bytes) if sha256_hex(&bytes) == *want_hash => {}
            Ok(_) => actions.push(Action::Restore { path: path.clone() }),
            Err(_) => actions.push(Action::Copy { path: path.clone() }),
        }
    }
    for path in manifest.keys().filter(|p| !desired.contains_key(p.as_str())).filter(|p| fs_root.join(p).exists()) {
        actions.push(Action::Remove { path: path.clone() });
    }
    Diff { actions }
}

#[cfg(test)]
mod tests {
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use crate::state::diff::{diff, sha256_hex, Action};

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
        assert_eq!(plan.actions, vec![Action::Copy { path: "plugins/ce.js".into() }]);
    }

    #[test]
    fn modified_file_plans_restore() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), "plugins/ce.js", b"tampered");
        let desired = BTreeMap::from([("plugins/ce.js".to_string(), sha256_hex(b"loader"))]);
        let plan = diff(&desired, &desired, dir.path());
        assert_eq!(plan.actions, vec![Action::Restore { path: "plugins/ce.js".into() }]);
    }

    #[test]
    fn stale_managed_file_plans_remove() {
        let dir = tempdir().unwrap();
        write_file(dir.path(), "plugins/old.js", b"old");
        let manifest = BTreeMap::from([("plugins/old.js".to_string(), sha256_hex(b"old"))]);
        let plan = diff(&BTreeMap::new(), &manifest, dir.path());
        assert_eq!(plan.actions, vec![Action::Remove { path: "plugins/old.js".into() }]);
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
}