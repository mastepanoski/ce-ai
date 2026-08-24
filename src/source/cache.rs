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

    use crate::source::cache::{read_local_tree, record_tarball_provenance, Cache};
    use crate::state::diff::sha256_hex;
    use crate::state::state::{ReleaseProvenance, State};

    #[test]
    fn caches_tarball_under_cache_dir() {
        let dir = tempdir().unwrap();
        let bytes = b"fake-tarball-bytes";
        let (cached, hex) = Cache::new(dir.path().join("cache"))
            .cache_tarball(bytes)
            .unwrap();
        assert!(cached.starts_with(dir.path().join("cache")));
        assert_eq!(std::fs::read(&cached).unwrap(), bytes);
        assert_eq!(hex, sha256_hex(bytes));
    }

    #[test]
    fn caching_alone_does_not_touch_state() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");
        Cache::new(dir.path().join("cache"))
            .cache_tarball(b"fake-tarball-bytes")
            .unwrap();
        assert!(!state_path.exists());
    }

    #[test]
    fn provenance_recording_is_atomic_and_complete() {
        let dir = tempdir().unwrap();
        let bytes = b"fake-tarball-bytes";
        let (_, hex) = Cache::new(dir.path().join("cache"))
            .cache_tarball(bytes)
            .unwrap();
        let state_path = dir.path().join("state.json");
        let extraction = dir.path().join("trees").join("v1.2.3");
        record_tarball_provenance(
            &state_path,
            ReleaseProvenance {
                tag: "v1.2.3".into(),
                url: "https://example.test/ce-v1.2.3.tar.gz".into(),
                archive_sha256: hex.clone(),
                extraction_path: extraction,
            },
        )
        .unwrap();

        // One atomic write → only cache/ and state.json exist, no temp leftovers.
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
        let state = State::load(&state_path).unwrap();
        assert_eq!(
            state
                .managed_asset_digest
                .get("tarball")
                .map(String::as_str),
            Some(format!("sha256:{hex}").as_str())
        );
        let prov = state.release_provenance.expect("provenance recorded");
        assert_eq!(prov.tag, "v1.2.3");
        assert_eq!(prov.archive_sha256, hex);
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
