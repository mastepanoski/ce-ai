//! Ports and adapters for state and configuration persistence (KTD4, R6).
//!
//! Provides `StateStore` and `ConfigStore` traits along with filesystem-backed
//! production adapters (`FsStateStore`, `FsConfigStore`) and in-memory test
//! adapters (`InMemoryStateStore`, `InMemoryConfigStore`).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::error::CeError;
use crate::state::state::State;

/// Abstraction for loading and persisting `state.json` (KTD4).
pub trait StateStore: Send + Sync {
    /// Loads state from the given path. Missing file returns default state.
    fn load(&self, path: &Path) -> Result<State, CeError>;

    /// Saves state to the given path atomically.
    fn save(&self, path: &Path, state: &State) -> Result<(), CeError>;

    /// Loads global state and applies local `.ce-ai.json` overrides if present.
    fn load_with_workspace_overrides(
        &self,
        global_path: &Path,
        workspace_root: Option<&Path>,
    ) -> Result<State, CeError>;
}

/// Abstraction for reading and writing JSON config files (e.g. `opencode.json`).
pub trait ConfigStore: Send + Sync {
    /// Reads JSON config from the given path. Missing file returns `{}`.
    fn read_config(&self, path: &Path) -> Result<serde_json::Value, CeError>;

    /// Writes JSON config to the given path atomically.
    fn write_config(&self, path: &Path, config: &serde_json::Value) -> Result<(), CeError>;
}

/// Production filesystem adapter for `StateStore`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FsStateStore;

impl StateStore for FsStateStore {
    fn load(&self, path: &Path) -> Result<State, CeError> {
        State::load(path)
    }

    fn save(&self, path: &Path, state: &State) -> Result<(), CeError> {
        state.save(path)
    }

    fn load_with_workspace_overrides(
        &self,
        global_path: &Path,
        workspace_root: Option<&Path>,
    ) -> Result<State, CeError> {
        State::load_with_workspace_overrides(global_path, workspace_root)
    }
}

/// Production filesystem adapter for `ConfigStore`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FsConfigStore;

impl ConfigStore for FsConfigStore {
    fn read_config(&self, path: &Path) -> Result<serde_json::Value, CeError> {
        crate::state::read_config(path)
    }

    fn write_config(&self, path: &Path, config: &serde_json::Value) -> Result<(), CeError> {
        let bytes = serde_json::to_vec_pretty(config)?;
        crate::state::write_atomic(path, &bytes)
    }
}

/// In-memory thread-safe adapter for `StateStore`, enabling hermetic tests
/// without touching the host filesystem.
#[derive(Debug, Default)]
pub struct InMemoryStateStore {
    states: RwLock<HashMap<PathBuf, State>>,
}

impl InMemoryStateStore {
    /// Creates a new empty `InMemoryStateStore`.
    pub fn new() -> Self {
        Self {
            states: RwLock::new(HashMap::new()),
        }
    }

    /// Pre-populates a state at a given path.
    pub fn insert(&self, path: impl Into<PathBuf>, state: State) {
        let mut map = self.states.write().unwrap_or_else(|e| e.into_inner());
        map.insert(path.into(), state);
    }

    /// Retrieves the state stored at a given path if present.
    pub fn get(&self, path: &Path) -> Option<State> {
        let map = self.states.read().unwrap_or_else(|e| e.into_inner());
        map.get(path).cloned()
    }
}

impl StateStore for InMemoryStateStore {
    fn load(&self, path: &Path) -> Result<State, CeError> {
        let map = self.states.read().unwrap_or_else(|e| e.into_inner());
        Ok(map.get(path).cloned().unwrap_or_else(State::new))
    }

    fn save(&self, path: &Path, state: &State) -> Result<(), CeError> {
        let mut map = self.states.write().unwrap_or_else(|e| e.into_inner());
        map.insert(path.to_path_buf(), state.clone());
        Ok(())
    }

    fn load_with_workspace_overrides(
        &self,
        global_path: &Path,
        workspace_root: Option<&Path>,
    ) -> Result<State, CeError> {
        let mut state = self.load(global_path)?;
        if let Some(ws_root) = workspace_root {
            let local_config = ws_root.join(".ce-ai.json");
            let map = self.states.read().unwrap_or_else(|e| e.into_inner());
            if let Some(local_state) = map.get(&local_config) {
                state.merge_overrides(local_state.clone());
            }
        }
        Ok(state)
    }
}

/// In-memory thread-safe adapter for `ConfigStore`, enabling hermetic tests
/// without touching the host filesystem.
#[derive(Debug, Default)]
pub struct InMemoryConfigStore {
    configs: RwLock<HashMap<PathBuf, serde_json::Value>>,
}

impl InMemoryConfigStore {
    /// Creates a new empty `InMemoryConfigStore`.
    pub fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
        }
    }

    /// Pre-populates a configuration at a given path.
    pub fn insert(&self, path: impl Into<PathBuf>, value: serde_json::Value) {
        let mut map = self.configs.write().unwrap_or_else(|e| e.into_inner());
        map.insert(path.into(), value);
    }

    /// Retrieves the config stored at a given path if present.
    pub fn get(&self, path: &Path) -> Option<serde_json::Value> {
        let map = self.configs.read().unwrap_or_else(|e| e.into_inner());
        map.get(path).cloned()
    }
}

impl ConfigStore for InMemoryConfigStore {
    fn read_config(&self, path: &Path) -> Result<serde_json::Value, CeError> {
        let map = self.configs.read().unwrap_or_else(|e| e.into_inner());
        Ok(map
            .get(path)
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})))
    }

    fn write_config(&self, path: &Path, config: &serde_json::Value) -> Result<(), CeError> {
        let mut map = self.configs.write().unwrap_or_else(|e| e.into_inner());
        map.insert(path.to_path_buf(), config.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn fs_state_store_roundtrips() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let store = FsStateStore;

        let mut state = State::new();
        state.set_model_assignment("ce-brainstorm", "openrouter", "deepseek-v3");
        store.save(&path, &state).unwrap();

        let loaded = store.load(&path).unwrap();
        assert_eq!(loaded, state);
    }

    #[test]
    fn fs_config_store_roundtrips() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("opencode.json");
        let store = FsConfigStore;

        let config = serde_json::json!({ "plugin": ["custom-plugin"] });
        store.write_config(&path, &config).unwrap();

        let loaded = store.read_config(&path).unwrap();
        assert_eq!(loaded, config);
    }

    #[test]
    fn in_memory_state_store_operates_without_filesystem() {
        let store = InMemoryStateStore::new();
        let path = PathBuf::from("/virtual/state.json");

        // Missing returns new default state
        let initial = store.load(&path).unwrap();
        assert_eq!(initial.version, 1);
        assert!(initial.model_assignments.is_empty());

        let mut state = State::new();
        state.set_model_assignment("ce-work", "anthropic", "claude-3-7-sonnet");
        store.save(&path, &state).unwrap();

        let loaded = store.load(&path).unwrap();
        assert_eq!(loaded, state);
        assert_eq!(
            store.get(&path).unwrap().model_assignments["ce-work"].model_id,
            "claude-3-7-sonnet"
        );
    }

    #[test]
    fn in_memory_state_store_handles_workspace_overrides() {
        let store = InMemoryStateStore::new();
        let global_path = PathBuf::from("/virtual/global/state.json");
        let ws_root = PathBuf::from("/virtual/repo");
        let ws_override_path = ws_root.join(".ce-ai.json");

        let mut global_state = State::new();
        global_state.set_model_assignment("ce-plan", "openai", "gpt-4o");
        store.save(&global_path, &global_state).unwrap();

        let mut local_state = State::new();
        local_state.set_model_assignment("ce-plan", "anthropic", "claude-3-7-sonnet");
        store.insert(&ws_override_path, local_state);

        let merged = store
            .load_with_workspace_overrides(&global_path, Some(&ws_root))
            .unwrap();
        assert_eq!(merged.model_assignments["ce-plan"].provider_id, "anthropic");
    }

    #[test]
    fn in_memory_config_store_operates_without_filesystem() {
        let store = InMemoryConfigStore::new();
        let path = PathBuf::from("/virtual/opencode.json");

        // Missing returns empty object {}
        let empty = store.read_config(&path).unwrap();
        assert_eq!(empty, serde_json::json!({}));

        let config = serde_json::json!({ "skills": { "paths": ["/virtual/skills"] } });
        store.write_config(&path, &config).unwrap();

        let loaded = store.read_config(&path).unwrap();
        assert_eq!(loaded, config);
    }
}
