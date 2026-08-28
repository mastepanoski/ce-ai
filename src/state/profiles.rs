//! Named model profiles (MM-3) and append-only versioned snapshots (MM-4).

use crate::error::CeError;
use crate::state::write_atomic;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Named profile: `~/.ce-ai/profiles/<name>.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub created_at: String,
    pub models: BTreeMap<String, String>,
}

/// Append-only snapshot: `profiles/versions/<name>-<utc-ts>.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub name: String,
    pub created_at: String,
    pub before_raw: BTreeMap<String, String>,
    pub preview: BTreeMap<String, String>,
}

fn profile_path(root: &Path, name: &str) -> Result<PathBuf, CeError> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(CeError::Usage(format!("invalid profile name: {name:?}")));
    }
    Ok(root.join(format!("{name}.json")))
}

fn snapshot_ts() -> String {
    Utc::now().format("%Y%m%dT%H%M%S%.6fZ").to_string()
}

pub fn save_profile(root: &Path, profile: &Profile) -> Result<(), CeError> {
    write_atomic(
        &profile_path(root, &profile.name)?,
        &serde_json::to_vec(profile)?,
    )
}

pub fn load_profile(root: &Path, name: &str) -> Result<Profile, CeError> {
    Ok(serde_json::from_slice(&std::fs::read(profile_path(
        root, name,
    )?)?)?)
}

/// Writes an append-only snapshot and returns its filename.
pub fn save_snapshot(
    root: &Path,
    name: &str,
    before_raw: &BTreeMap<String, String>,
    preview: &BTreeMap<String, String>,
) -> Result<String, CeError> {
    let snapshot = Snapshot {
        name: name.to_string(),
        created_at: Utc::now().to_rfc3339(),
        before_raw: before_raw.clone(),
        preview: preview.clone(),
    };
    let filename = format!("{name}-{}.json", snapshot_ts());
    write_atomic(
        &root.join("versions").join(&filename),
        &serde_json::to_vec(&snapshot)?,
    )?;
    Ok(filename)
}

#[cfg(test)]
#[path = "tests/profiles.rs"]
mod tests;
