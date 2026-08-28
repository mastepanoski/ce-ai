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
