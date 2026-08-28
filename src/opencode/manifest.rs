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
        write_atomic(
            &Self::path_for(config_dir),
            &serde_json::to_vec_pretty(self)?,
        )
    }

    /// Loads the manifest; errors when absent or malformed.
    pub fn load(config_dir: &Path) -> Result<Self, CeError> {
        let bytes = std::fs::read(Self::path_for(config_dir))?;
        Ok(serde_json::from_slice(&bytes)?)
    }
}

#[cfg(test)]
#[path = "tests/manifest.rs"]
mod tests;
