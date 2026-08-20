//! Persistent state: state.json, profiles, snapshots, backups, and the sync diff engine.
//! Wired into CLI commands in later PRs; until then items are exercised by unit tests.

#![allow(dead_code)]

pub mod backups;
pub mod diff;
pub mod profiles;
pub mod state;

use std::io::Write;
use std::path::Path;
use crate::error::CeError;

/// Atomic write via a temp file + rename in the same directory.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), CeError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
    let tmp = parent.join(format!(".{name}.tmp{}", std::process::id()));
    let mut file = std::fs::File::create(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}