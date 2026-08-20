//! opencode.json read → merge (dedup) → atomic write; hard-fails on invalid
//! existing JSON instead of clobbering user config (OI-2, D4).
//!
//! RED: tests reference `ensure_plugin_and_skills` / `ConfigMutation`, which do
//! not exist yet — this file fails to compile until the GREEN implementation.

#[cfg(test)]
mod tests {
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
            .join("compound-engineering/plugins/compound-engineering.js")
            .to_string_lossy()
            .into_owned()
    }

    fn skills_path(config_dir: &Path) -> String {
        config_dir.join("compound-engineering/skills").to_string_lossy().into_owned()
    }

    #[test]
    fn merges_plugin_entry_without_clobbering_user_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("opencode.json");
        write_json(
            &path,
            serde_json::json!({
                "plugin": ["user-plugin"],
                "agent": { "sdd-explore": { "model": "user-model" } }
            }),
        );
        let entry = loader_entry(dir.path());
        ensure_plugin_and_skills(&path, &entry, &skills_path(dir.path())).unwrap();

        let config = read_json(&path);
        let plugins = config["plugin"].as_array().expect("plugin is an array");
        assert_eq!(plugins.len(), 2, "user entry plus CE entry");
        assert!(plugins.iter().any(|v| v.as_str() == Some("user-plugin")));
        assert!(plugins.iter().any(|v| v.as_str() == Some(&entry)));
        assert_eq!(config["agent"]["sdd-explore"]["model"], "user-model");
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
        assert!(paths.iter().any(|v| v.as_str() == Some("/home/user/custom-skills")));
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
        assert_eq!(config["plugin"].as_array().unwrap(), &serde_json::json!([entry]));
        assert_eq!(config["skills"]["paths"].as_array().unwrap(), &serde_json::json!([skills]));
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
        assert_eq!(read_json(&path)["plugin"], "not-an-array", "user config preserved");
    }
}