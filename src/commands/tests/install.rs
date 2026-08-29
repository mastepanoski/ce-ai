use crate::state::{ConfigStore, InMemoryConfigStore, InMemoryStateStore, StateStore};
use std::path::Path;

#[test]
fn install_state_store_port_loads_and_saves_without_filesystem() {
    let store = InMemoryStateStore::new();
    let path = Path::new("/virtual/ce-ai/state.json");

    let mut state = store.load(path).unwrap();
    assert_eq!(state.version, 1);
    state.installed_harnesses.push(serde_json::json!({
        "name": "opencode",
        "version": "1.0.0",
        "installed_at": "2026-08-27T00:00:00Z"
    }));
    store.save(path, &state).unwrap();

    let loaded = store.load(path).unwrap();
    assert_eq!(loaded.installed_harnesses.len(), 1);
    assert_eq!(loaded.installed_harnesses[0]["name"], "opencode");
}

#[test]
fn install_config_store_port_mutates_without_filesystem() {
    let store = InMemoryConfigStore::new();
    let path = Path::new("/virtual/config/opencode.json");

    let mutation = crate::opencode::config::ensure_plugin_and_skills_with_store(
        &store,
        path,
        "/virtual/plugin.js",
        "/virtual/skills",
    )
    .unwrap();

    assert_eq!(mutation.keys, vec!["plugin", "skills.paths"]);
    let cfg = store.read_config(path).unwrap();
    assert_eq!(cfg["plugin"], serde_json::json!(["/virtual/plugin.js"]));
}
