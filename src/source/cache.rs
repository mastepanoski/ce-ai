//! Tarball cache + SHA256 digest recording (SF-3) and local CE tree reader
//! for `--source <local-path>` (SF-4).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::CeError;
use crate::state::diff::sha256_hex;
use crate::state::state::{ReleaseProvenance, State};
use crate::state::write_atomic;

/// Cache directory holding downloaded tarballs.
#[derive(Debug, Clone)]
pub struct Cache {
    pub dir: PathBuf,
}

impl Cache {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    /// Stores `bytes` under the cache dir keyed by its SHA256 and returns
    /// `(path, lowercase-hex digest)`. Does not touch state.json — callers
    /// must pair it with [`record_tarball_provenance`] so the digest and the
    /// release provenance land atomically together (Issue #161).
    pub fn cache_tarball(&self, bytes: &[u8]) -> Result<(PathBuf, String), CeError> {
        let hex = sha256_hex(bytes);
        let dest = self.dir.join(format!("ce-{hex}.tar.gz"));
        write_atomic(&dest, bytes)?;
        Ok((dest, hex))
    }
}

/// Atomically records the cached tarball digest **and** its release
/// provenance `{tag, url, archive_sha256, extraction_path}` in one
/// temp-file+rename state.json write (Issue #161).
pub fn record_tarball_provenance(
    state_path: &Path,
    provenance: ReleaseProvenance,
) -> Result<(), CeError> {
    let mut state = State::load(state_path)?;
    state.managed_asset_digest.insert(
        "tarball".to_string(),
        format!("sha256:{}", provenance.archive_sha256),
    );
    state.release_provenance = Some(provenance);
    state.save(state_path)
}

/// Walks a local CE tree and returns `relative/path -> sha256` for every file
/// (SF-4). Pure filesystem read — never touches the network.
pub fn read_local_tree(root: &Path) -> Result<BTreeMap<String, String>, CeError> {
    let mut tree = BTreeMap::new();
    walk_tree(root, root, &mut tree)?;
    Ok(tree)
}

/// Harvested managed set: managed-relative path -> (source-relative path,
/// sha256). Managed prefixes: `.opencode/plugins`, `.opencode/skills` (the
/// `.opencode/` strip applies), and top-level `skills/`. Deterministic
/// precedence: legacy `.opencode/`-prefixed assets are collected first, then
/// top-level `skills/` overwrites on collision (warning emitted).
pub fn managed_tree(source_root: &Path) -> Result<BTreeMap<String, (String, String)>, CeError> {
    let tree = read_local_tree(source_root)?;
    let mut managed: BTreeMap<String, (String, String)> = BTreeMap::new();
    for (rel, hash) in &tree {
        if rel.starts_with(".opencode/plugins") || rel.starts_with(".opencode/skills") {
            let managed_rel = rel.trim_start_matches(".opencode/").to_string();
            managed.insert(managed_rel, (rel.clone(), hash.clone()));
        }
    }
    let mut overlapped = false;
    for (rel, hash) in &tree {
        if rel.starts_with("skills/")
            && managed
                .insert(rel.clone(), (rel.clone(), hash.clone()))
                .is_some()
        {
            overlapped = true;
        }
    }
    if overlapped {
        eprintln!(
            "warning: source ships both .opencode/skills and top-level skills/; top-level wins"
        );
    }
    Ok(managed)
}

fn walk_tree(root: &Path, dir: &Path, tree: &mut BTreeMap<String, String>) -> Result<(), CeError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_tree(root, &path, tree)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(root)
                .map_err(|err| CeError::Runtime(err.to_string()))?
                .to_string_lossy()
                .replace('\\', "/");
            tree.insert(rel, sha256_hex(&std::fs::read(&path)?));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/cache.rs"]
mod tests;
