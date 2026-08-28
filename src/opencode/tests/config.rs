use std::path::Path;

use tempfile::tempdir;

use super::*;

fn write_json(path: &Path, value: serde_json::Value) {
    std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn loader_entry(config_dir: &Path) -> String {
    config_dir
        .join("compound-engineering")
        .join("plugins")
        .join("compound-engineering.js")
        .to_string_lossy()
        .into_owned()
}

fn skills_path(config_dir: &Path) -> String {
    config_dir
        .join("compound-engineering")
        .join("skills")
        .to_string_lossy()
        .into_owned()
}

#[test]
fn merges_plugin_entry_without_clobbering_user_config() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("opencode.json");
    write_json(
        &path,
        serde_json::json!({
            "plugin": ["user-plugin"],
            "agent": { "ce-brainstorm": { "model": "user-model" } }
        }),
    );
    let entry = loader_entry(dir.path());
    ensure_plugin_and_skills(&path, &entry, &skills_path(dir.path())).unwrap();

    let config = read_json(&path);
    let plugins = config["plugin"].as_array().expect("plugin is an array");
    assert_eq!(plugins.len(), 2, "user entry plus CE entry");
    assert!(plugins.iter().any(|v| v.as_str() == Some("user-plugin")));
    assert!(plugins.iter().any(|v| v.as_str() == Some(&entry)));
    assert_eq!(config["agent"]["ce-brainstorm"]["model"], "user-model");
}

#[test]
fn reinstall_does_not_duplicate_entries() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("opencode.json");
    let entry = loader_entry(dir.path());
    let skills = skills_path(dir.path());
    ensure_plugin_and_skills(&path, &entry, &skills).unwrap();
    ensure_plugin_and_skills(&path, &entry, &skills).unwrap();

    let config = read_json(&path);
    assert_eq!(config["plugin"].as_array().unwrap().len(), 1);
    assert_eq!(config["skills"]["paths"].as_array().unwrap().len(), 1);
}

#[test]
fn merges_skills_paths_with_dedup_keeping_user_paths() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("opencode.json");
    write_json(
        &path,
        serde_json::json!({ "skills": { "paths": ["/home/user/custom-skills"] } }),
    );
    let skills = skills_path(dir.path());
    ensure_plugin_and_skills(&path, &loader_entry(dir.path()), &skills).unwrap();

    let config = read_json(&path);
    let paths = config["skills"]["paths"].as_array().unwrap();
    assert_eq!(paths.len(), 2);
    assert!(paths
        .iter()
        .any(|v| v.as_str() == Some("/home/user/custom-skills")));
    assert!(paths.iter().any(|v| v.as_str() == Some(&skills)));
}

#[test]
fn creates_plugin_and_skills_arrays_when_missing() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("opencode.json");
    let entry = loader_entry(dir.path());
    let skills = skills_path(dir.path());
    ensure_plugin_and_skills(&path, &entry, &skills).unwrap();

    let config = read_json(&path);
    assert_eq!(config["plugin"], serde_json::json!([entry]));
    assert_eq!(config["skills"]["paths"], serde_json::json!([skills]));
}

#[test]
fn invalid_existing_json_hard_fails_with_fix_guidance() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("opencode.json");
    std::fs::write(&path, "{ this is not json").unwrap();

    let err = ensure_plugin_and_skills(&path, "plugin-entry", "skills-path").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("not valid JSON"), "names the problem: {msg}");
    assert!(msg.contains("opencode.json"), "names the file: {msg}");
    assert!(msg.contains("Fix the file"), "gives fix guidance: {msg}");
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "{ this is not json",
        "broken config is never overwritten (D4)"
    );
}

#[test]
fn non_array_plugin_key_hard_fails_instead_of_clobbering() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("opencode.json");
    write_json(&path, serde_json::json!({ "plugin": "not-an-array" }));

    let err = ensure_plugin_and_skills(&path, "plugin-entry", "skills-path").unwrap_err();
    assert!(err.to_string().contains("plugin"));
    assert_eq!(
        read_json(&path)["plugin"],
        "not-an-array",
        "user config preserved"
    );
}

#[test]
fn register_mcp_server_creates_block_and_preserves_malformed_failures() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("opencode.json");

    // 1. Create on empty config
    write_json(&path, serde_json::json!({}));
    register_mcp_server(&path, "context7", serde_json::json!({"command": "npx"})).unwrap();
    let val = read_json(&path);
    assert_eq!(val["mcpServers"]["context7"]["command"], "npx");

    // 2. Fail safely on non-object mcpServers
    write_json(&path, serde_json::json!({ "mcpServers": "invalid-string" }));
    let err = register_mcp_server(&path, "rtk", serde_json::json!({"command": "rtk"})).unwrap_err();
    assert!(err.to_string().contains("mcpServers"));
    assert_eq!(read_json(&path)["mcpServers"], "invalid-string");
}

#[test]
fn in_memory_config_store_works_with_ensure_and_register() {
    let store = crate::state::InMemoryConfigStore::new();
    let path = Path::new("/virtual/config/opencode.json");

    let mutation =
        ensure_plugin_and_skills_with_store(&store, path, "/virtual/plugin.js", "/virtual/skills")
            .unwrap();
    assert_eq!(mutation.keys, vec!["plugin", "skills.paths"]);

    register_mcp_server_with_store(
        &store,
        path,
        "codegraph",
        serde_json::json!({ "command": "codegraph" }),
    )
    .unwrap();

    let cfg = store.read_config(path).unwrap();
    assert_eq!(cfg["plugin"], serde_json::json!(["/virtual/plugin.js"]));
    assert_eq!(
        cfg["skills"]["paths"],
        serde_json::json!(["/virtual/skills"])
    );
    assert_eq!(
        cfg["mcpServers"]["codegraph"],
        serde_json::json!({ "command": "codegraph" })
    );
}
