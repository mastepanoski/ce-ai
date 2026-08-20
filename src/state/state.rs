#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;
    use tempfile::tempdir;

    use crate::state::state::{ModelAssignment, State};

    fn state_with(slot: &str) -> State {
        let mut assignments = BTreeMap::new();
        assignments.insert(
            slot.to_string(),
            ModelAssignment {
                provider_id: "opencode-go".to_string(),
                model_id: "kimi-k2.6".to_string(),
                effort: None,
            },
        );
        State {
            version: 1,
            installed_harnesses: vec![],
            managed_asset_digest: BTreeMap::new(),
            model_assignments: assignments,
            last_update_check: None,
        }
    }

    fn assert_round_trip(path: &Path, state: &State) {
        let loaded = State::load(path).unwrap();
        assert_eq!(loaded, *state);
    }

    #[test]
    fn round_trips_through_state_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let state = state_with("sdd-explore");
        state.save(&path).unwrap();
        assert_round_trip(&path, &state);
    }

    #[test]
    fn atomic_write_leaves_no_temp_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        state_with("sdd-explore").save(&path).unwrap();
        let names: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["state.json".to_string()]);
    }

    #[test]
    fn persists_model_assignments_across_reloads() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut state = State::new();
        state.set_model_assignment("sdd-explore", "opencode-go", "kimi-k2.6");
        state.save(&path).unwrap();
        let loaded = State::load(&path).unwrap();
        let assignment = &loaded.model_assignments["sdd-explore"];
        assert_eq!(
            (assignment.provider_id.as_str(), assignment.model_id.as_str()),
            ("opencode-go", "kimi-k2.6")
        );
    }

    #[test]
    fn load_missing_file_returns_default_state() {
        let dir = tempdir().unwrap();
        let loaded = State::load(&dir.path().join("absent.json")).unwrap();
        assert_eq!(loaded.version, 1);
        assert!(loaded.model_assignments.is_empty());
    }
}
