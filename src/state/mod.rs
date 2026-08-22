//! Persistent state: state.json, profiles, snapshots, backups, and the sync diff engine.
//! Wired into CLI commands in later PRs; until then items are exercised by unit tests.

#![allow(dead_code)]

pub mod backups;
pub mod diff;
pub mod profiles;
// `state::state` holds the State type; module_inception is intentional.
#[allow(clippy::module_inception)]
pub mod state;

use crate::error::CeError;
use std::io::Write;
use std::path::Path;

/// Reads a harness JSON config file. Missing file → empty object; invalid
/// JSON → hard-fail with fix guidance (D4) — never silently overwrite a
/// broken config. Neutral layer shared by every harness backend.
pub fn read_config(path: &Path) -> Result<serde_json::Value, CeError> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).map_err(|err| {
            CeError::Runtime(format!(
                "{} is not valid JSON: {err}. Refusing to overwrite it. Fix the file manually, then re-run.",
                path.display()
            ))
        }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::json!({})),
        Err(err) => Err(err.into()),
    }
}

/// Atomic write via a temp file + rename in the same directory.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), CeError> {
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
