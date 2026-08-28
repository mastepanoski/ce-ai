//! Sync diff engine: plan copy/restore/remove actions without writing (SU-1..SU-4).

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

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
pub fn diff(
    desired: &BTreeMap<String, String>,
    manifest: &BTreeMap<String, String>,
    fs_root: &Path,
) -> Diff {
    let mut actions = Vec::new();
    for (path, want_hash) in desired {
        match std::fs::read(fs_root.join(path)) {
            Ok(bytes) if sha256_hex(&bytes) == *want_hash => {}
            Ok(_) => actions.push(Action::Restore { path: path.clone() }),
            Err(_) => actions.push(Action::Copy { path: path.clone() }),
        }
    }
    for path in manifest
        .keys()
        .filter(|p| !desired.contains_key(p.as_str()))
        .filter(|p| fs_root.join(p).exists())
    {
        actions.push(Action::Remove { path: path.clone() });
    }
    Diff { actions }
}

#[cfg(test)]
#[path = "tests/diff.rs"]
mod tests;
