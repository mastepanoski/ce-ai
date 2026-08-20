#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    use crate::state::profiles::{list_snapshots, load_profile, save_profile, save_snapshot, Profile};

    fn models() -> BTreeMap<String, String> {
        BTreeMap::from([("sdd-explore".to_string(), "opencode-go/kimi-k2.6".to_string())])
    }

    #[test]
    fn named_profile_round_trip() {
        let dir = tempdir().unwrap();
        let profile = Profile {
            name: "fast".to_string(),
            created_at: "2026-08-20T00:00:00Z".to_string(),
            models: models(),
        };
        save_profile(dir.path(), &profile).unwrap();
        let loaded = load_profile(dir.path(), "fast").unwrap();
        assert_eq!(loaded, profile);
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
        let snapshots = list_snapshots(dir.path(), "fast").unwrap();
        assert_eq!(snapshots.len(), 2);
        assert!(snapshots[0].starts_with("fast-") && snapshots[0].ends_with(".json"));
        let first_content = std::fs::read_to_string(dir.path().join("versions").join(first)).unwrap();
        assert!(first_content.contains("\"name\": \"fast\""));
    }
}
