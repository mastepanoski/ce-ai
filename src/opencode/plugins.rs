//! CE plugin loader placement (OI-3) and skills-path registration (OI-4).

use std::path::{Path, PathBuf};

use crate::error::CeError;
use crate::opencode::manifest::ManifestFile;
use crate::state::diff::sha256_hex;

/// Directory under the OpenCode config dir that ce-ai manages (D3).
pub const MANAGED_DIR: &str = "compound-engineering";
/// Loader file path relative to the managed dir (design §Interfaces).
pub const LOADER_REL_PATH: &str = "plugins/compound-engineering.js";
/// Loader location inside the CE source tree (proposal open item 3).
const SOURCE_LOADER_PATH: &str = ".opencode/plugins/compound-engineering.js";

/// Absolute path of the installed loader — the `plugin[]` entry value (D2).
pub fn plugin_entry(config_dir: &Path) -> PathBuf {
    config_dir
        .join(MANAGED_DIR)
        .join("plugins")
        .join("compound-engineering.js")
}

/// Absolute skills directory registered in `skills.paths` (OI-4).
pub fn skills_path(config_dir: &Path) -> PathBuf {
    config_dir.join(MANAGED_DIR).join("skills")
}

/// Copies the CE loader from the source tree into
/// `<config>/compound-engineering/plugins/compound-engineering.js` (OI-3).
/// Returns the managed-relative path and its SHA256 for the manifest (OI-5).
pub fn install_loader(source_root: &Path, config_dir: &Path) -> Result<ManifestFile, CeError> {
    let src = source_root.join(SOURCE_LOADER_PATH);
    let bytes = std::fs::read(&src).map_err(|err| {
        CeError::Runtime(format!("CE loader not found at {}: {err}", src.display()))
    })?;
    let dest = plugin_entry(config_dir);
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&dest, &bytes)?;
    Ok(ManifestFile {
        path: LOADER_REL_PATH.to_string(),
        sha256: sha256_hex(&bytes),
    })
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;
    use crate::state::diff::sha256_hex;

    #[test]
    fn copies_loader_into_managed_plugins_dir() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("ce-source");
        let loader_src = source.join(".opencode/plugins/compound-engineering.js");
        std::fs::create_dir_all(loader_src.parent().unwrap()).unwrap();
        let loader_bytes = b"export default function ceLoader() {}";
        std::fs::write(&loader_src, loader_bytes).unwrap();

        let config_dir = dir.path().join("opencode-config");
        let installed = install_loader(&source, &config_dir).unwrap();

        assert_eq!(installed.path, "plugins/compound-engineering.js");
        assert_eq!(installed.sha256, sha256_hex(loader_bytes));
        let dest = config_dir.join("compound-engineering/plugins/compound-engineering.js");
        assert_eq!(std::fs::read(&dest).unwrap(), loader_bytes);
    }

    #[test]
    fn skills_path_points_at_managed_skills_dir() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path().join("opencode-config");
        assert_eq!(
            skills_path(&config_dir),
            config_dir.join("compound-engineering/skills")
        );
    }
}
