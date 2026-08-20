//! Tarball cache + SHA256 digest recording (RED — tests only; implementation
//! lands in task 3.4).
//!
//! SF-3: tarballs are cached under `~/.ce-ai/cache` and their SHA256 digest is
//! recorded in `state.json` (`managed_asset_digest["tarball"]`).
//! SF-4: `--source <local-path>` reads a local CE tree with no network.

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
        let cached =
            Cache::new(dir.path().join("cache")).cache_tarball(bytes, &dir.path().join("state.json")).unwrap();
        assert!(cached.starts_with(dir.path().join("cache")));
        assert_eq!(std::fs::read(&cached).unwrap(), bytes);
    }

    #[test]
    fn records_sha256_digest_in_state() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");
        let bytes = b"fake-tarball-bytes";
        Cache::new(dir.path().join("cache")).cache_tarball(bytes, &state_path).unwrap();
        let state = State::load(&state_path).unwrap();
        let digest = state.managed_asset_digest.get("tarball").expect("tarball digest recorded");
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
                ("plugins/compound-engineering.js".to_string(), sha256_hex(b"loader")),
                ("skills/ce-brainstorm/SKILL.md".to_string(), sha256_hex(b"# skill")),
                ("empty.tmp".to_string(), sha256_hex(b"")),
            ])
        );
    }
}