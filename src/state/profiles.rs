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
mod tests {
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
}
