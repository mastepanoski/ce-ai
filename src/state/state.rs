//! `state.json` I/O with atomic temp-file+rename writes (MM-1).

use crate::error::CeError;
use crate::state::write_atomic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelAssignment {
    pub provider_id: String,
    pub model_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
}

/// Canonical state file at `~/.ce-ai/state.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub installed_harnesses: Vec<serde_json::Value>,
    #[serde(default)]
    pub managed_asset_digest: BTreeMap<String, String>,
    #[serde(default)]
    pub model_assignments: BTreeMap<String, ModelAssignment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update_check: Option<String>,
}

fn default_version() -> u32 {
    1
}

impl State {
    pub fn new() -> Self {
        Self {
            version: 1,
            ..Self::default()
        }
    }

    pub fn load(path: &Path) -> Result<Self, CeError> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::new()),
            Err(err) => Err(err.into()),
        }
    }

    /// Persists state.json atomically (temp file + rename).
    pub fn save(&self, path: &Path) -> Result<(), CeError> {
        write_atomic(path, &serde_json::to_vec(self)?)
    }

    /// Records a model assignment for a slot (MM-1).
    pub fn set_model_assignment(&mut self, slot: &str, provider_id: &str, model_id: &str) {
        let assignment = ModelAssignment {
            provider_id: provider_id.into(),
            model_id: model_id.into(),
            effort: None,
        };
        self.model_assignments.insert(slot.into(), assignment);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    use crate::state::state::{ModelAssignment, State};

    fn state_with(slot: &str) -> State {
        State {
            version: 1,
            installed_harnesses: vec![],
            managed_asset_digest: BTreeMap::new(),
            model_assignments: BTreeMap::from([(
                slot.to_string(),
                ModelAssignment {
                    provider_id: "opencode-go".into(),
                    model_id: "kimi-k2.6".into(),
                    effort: None,
                },
            )]),
            last_update_check: None,
        }
    }

    #[test]
    fn round_trips_through_state_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let state = state_with("sdd-explore");
        state.save(&path).unwrap();
        assert_eq!(State::load(&path).unwrap(), state);
    }

    #[test]
    fn atomic_write_leaves_no_temp_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        state_with("sdd-explore").save(&path).unwrap();
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn persists_model_assignments_across_reloads() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut state = State::new();
        state.set_model_assignment("sdd-explore", "opencode-go", "kimi-k2.6");
        state.save(&path).unwrap();
        let assignment = &State::load(&path).unwrap().model_assignments["sdd-explore"];
        assert_eq!(assignment.provider_id, "opencode-go");
        assert_eq!(assignment.model_id, "kimi-k2.6");
    }

    #[test]
    fn load_missing_file_returns_default_state() {
        let dir = tempdir().unwrap();
        let loaded = State::load(&dir.path().join("absent.json")).unwrap();
        assert_eq!(loaded.version, 1);
        assert!(loaded.model_assignments.is_empty());
    }
}
