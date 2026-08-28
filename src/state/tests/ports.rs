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
