#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use tempfile::tempdir;

    use crate::state::diff::{diff, sha256_hex, Action};

    fn map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
        entries
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn missing_desired_file_plans_copy() {
        let dir = tempdir().unwrap();
        let hash = sha256_hex(b"loader");
        let desired = map(&[("plugins/ce.js", &hash)]);
        let plan = diff(&desired, &BTreeMap::new(), dir.path());
        assert_eq!(plan.actions, vec![Action::Copy { path: "plugins/ce.js".into() }]);
    }

    #[test]
    fn modified_file_plans_restore() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("plugins")).unwrap();
        fs::write(dir.path().join("plugins/ce.js"), b"tampered").unwrap();
        let hash = sha256_hex(b"loader");
        let desired = map(&[("plugins/ce.js", &hash)]);
        let plan = diff(&desired, &desired, dir.path());
        assert_eq!(plan.actions, vec![Action::Restore { path: "plugins/ce.js".into() }]);
    }

    #[test]
    fn stale_managed_file_plans_remove() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("plugins")).unwrap();
        fs::write(dir.path().join("plugins/old.js"), b"old").unwrap();
        let hash = sha256_hex(b"old");
        let manifest = map(&[("plugins/old.js", &hash)]);
        let plan = diff(&BTreeMap::new(), &manifest, dir.path());
        assert_eq!(plan.actions, vec![Action::Remove { path: "plugins/old.js".into() }]);
    }

    #[test]
    fn up_to_date_files_plan_nothing() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("plugins")).unwrap();
        fs::write(dir.path().join("plugins/ce.js"), b"loader").unwrap();
        let hash = sha256_hex(b"loader");
        let desired = map(&[("plugins/ce.js", &hash)]);
        assert!(diff(&desired, &desired, dir.path()).actions.is_empty());
    }

    #[test]
    fn diff_plans_without_writing() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("plugins")).unwrap();
        let hash = sha256_hex(b"x");
        let desired = map(&[("plugins/missing.js", &hash)]);
        let _ = diff(&desired, &BTreeMap::new(), dir.path());
        assert!(!dir.path().join("plugins/missing.js").exists());
    }
}
