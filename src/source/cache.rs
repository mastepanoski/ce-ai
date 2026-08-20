//! Tarball cache + SHA256 digest recording (SF-3) and local CE tree reader
//! for `--source <local-path>` (SF-4).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::CeError;
use crate::state::diff::sha256_hex;
use crate::state::state::State;
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

    /// Stores `bytes` under the cache dir keyed by its SHA256 and records
    /// `managed_asset_digest["tarball"] = "sha256:<hex>"` in state.json
    /// (SF-3). Returns the cached tarball path.
    pub fn cache_tarball(&self, bytes: &[u8], state_path: &Path) -> Result<PathBuf, CeError> {
        let hex = sha256_hex(bytes);
        let dest = self.dir.join(format!("ce-{hex}.tar.gz"));
        write_atomic(&dest, bytes)?;
        let mut state = State::load(state_path)?;
        state
            .managed_asset_digest
            .insert("tarball".to_string(), format!("sha256:{hex}"));
        state.save(state_path)?;
        Ok(dest)
    }
}

/// Walks a local CE tree and returns `relative/path -> sha256` for every file
/// (SF-4). Pure filesystem read — never touches the network.
pub fn read_local_tree(root: &Path) -> Result<BTreeMap<String, String>, CeError> {
    let mut tree = BTreeMap::new();
    walk_tree(root, root, &mut tree)?;
    Ok(tree)
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
mod tests {
    use std::collections::BTreeMap;

    use tempfile::tempdir;

    use crate::source::cache::{read_local_tree, Cache};
    use crate::state::diff::sha256_hex;
    use crate::state::state::State;

    #[test]
    fn caches_tarball_under_cache_dir() {
        let dir = tempdir().unwrap();
        let bytes = b"fake-tarball-bytes";
        let cached = Cache::new(dir.path().join("cache"))
            .cache_tarball(bytes, &dir.path().join("state.json"))
            .unwrap();
        assert!(cached.starts_with(dir.path().join("cache")));
        assert_eq!(std::fs::read(&cached).unwrap(), bytes);
    }

    #[test]
    fn records_sha256_digest_in_state() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");
        let bytes = b"fake-tarball-bytes";
        Cache::new(dir.path().join("cache"))
            .cache_tarball(bytes, &state_path)
            .unwrap();
        let state = State::load(&state_path).unwrap();
        let digest = state
            .managed_asset_digest
            .get("tarball")
            .expect("tarball digest recorded");
        assert_eq!(digest, &format!("sha256:{}", sha256_hex(bytes)));
    }

    #[test]
    fn local_source_tree_read_without_network() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("ce-tree");
        std::fs::create_dir_all(root.join("plugins")).unwrap();
        std::fs::create_dir_all(root.join("skills/ce-brainstorm")).unwrap();
        std::fs::write(root.join("plugins/compound-engineering.js"), b"loader").unwrap();
        std::fs::write(root.join("skills/ce-brainstorm/SKILL.md"), b"# skill").unwrap();
        std::fs::write(root.join("empty.tmp"), b"").unwrap();

        let tree = read_local_tree(&root).unwrap();
        assert_eq!(
            tree,
            BTreeMap::from([
                (
                    "plugins/compound-engineering.js".to_string(),
                    sha256_hex(b"loader")
                ),
                (
                    "skills/ce-brainstorm/SKILL.md".to_string(),
                    sha256_hex(b"# skill")
                ),
                ("empty.tmp".to_string(), sha256_hex(b"")),
            ])
        );
    }
}
