//! install-manifest.json I/O with per-file SHA256 digests (OI-5, SU-1/3).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CeError;
use crate::opencode::config::ConfigMutation;
use crate::opencode::plugins::MANAGED_DIR;
use crate::state::write_atomic;

/// One managed file recorded with its SHA256 (design §Interfaces).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestFile {
    pub path: String,
    pub sha256: String,
}

/// Install manifest at `<opencode-config>/compound-engineering/install-manifest.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallManifest {
    pub version: String,
    pub plugin_name: String,
    pub installed_at: String,
    pub source: serde_json::Value,
    pub files: Vec<ManifestFile>,
    pub config_mutations: Vec<ConfigMutation>,
}

impl InstallManifest {
    fn path_for(config_dir: &Path) -> PathBuf {
        config_dir.join(MANAGED_DIR).join("install-manifest.json")
    }

    /// Atomically writes the manifest under the managed dir (OI-5).
    pub fn write(&self, config_dir: &Path) -> Result<(), CeError> {
        write_atomic(&Self::path_for(config_dir), &serde_json::to_vec_pretty(self)?)
    }

    /// Loads the manifest; errors when absent or malformed.
    pub fn load(config_dir: &Path) -> Result<Self, CeError> {
        let bytes = std::fs::read(Self::path_for(config_dir))?;
        Ok(serde_json::from_slice(&bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn sample_manifest() -> InstallManifest {
        InstallManifest {
            version: "compound-engineering-v3.4.2".into(),
            plugin_name: "compound-engineering".into(),
            installed_at: "2026-08-20T00:00:00Z".into(),
            source: serde_json::json!({ "kind": "local", "path": "/tmp/ce" }),
            files: vec![
                ManifestFile {
                    path: "plugins/compound-engineering.js".into(),
                    sha256: "a".repeat(64),
                },
                ManifestFile {
                    path: "skills/ce-brainstorm/SKILL.md".into(),
                    sha256: "b".repeat(64),
                },
            ],
            config_mutations: vec![ConfigMutation {
                file: "opencode.json".into(),
                backup: None,
                keys: vec!["plugin".into(), "skills.paths".into()],
            }],
        }
    }

    #[test]
    fn manifest_round_trips_with_per_file_sha256() {
        let dir = tempdir().unwrap();
        let manifest = sample_manifest();
        manifest.write(dir.path()).unwrap();

        let loaded = InstallManifest::load(dir.path()).unwrap();
        assert_eq!(loaded, manifest);
        assert_eq!(loaded.files[0].sha256, "a".repeat(64));
        assert_eq!(loaded.files[1].sha256, "b".repeat(64));
    }

    #[test]
    fn manifest_written_under_managed_dir() {
        let dir = tempdir().unwrap();
        sample_manifest().write(dir.path()).unwrap();
        assert!(dir.path().join("compound-engineering/install-manifest.json").exists());
    }

    #[test]
    fn load_missing_manifest_errors() {
        let dir = tempdir().unwrap();
        assert!(InstallManifest::load(dir.path()).is_err());
    }
}