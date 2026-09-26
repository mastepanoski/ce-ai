use tempfile::tempdir;

use super::*;
use crate::state::diff::sha256_hex;

#[test]
fn copies_loader_into_managed_plugins_dir() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("ce-source");
    let loader_src = source.join(".opencode/plugins/compound-engineering.js");
    std::fs::create_dir_all(loader_src.parent().unwrap()).unwrap();
    let loader_bytes = b"export default function ceLoader() {}";
    std::fs::write(&loader_src, loader_bytes).unwrap();

    let config_dir = dir.path().join("opencode-config");
    let installed = install_loader(&source, &config_dir).unwrap();

    assert_eq!(installed.path, "plugins/compound-engineering.js");
    assert_eq!(installed.sha256, sha256_hex(loader_bytes));
    let dest = config_dir.join("compound-engineering/plugins/compound-engineering.js");
    assert_eq!(std::fs::read(&dest).unwrap(), loader_bytes);
}

#[test]
fn skills_path_points_at_managed_skills_dir() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("opencode-config");
    assert_eq!(
        skills_path(&config_dir),
        config_dir.join("compound-engineering/skills")
    );
}

#[test]
fn install_loader_falls_back_to_builtin_when_missing() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("empty-source");
    let config_dir = dir.path().join("opencode-config");

    let installed = install_loader(&source, &config_dir).unwrap();
    assert_eq!(installed.path, "plugins/compound-engineering.js");
    let dest = config_dir.join("compound-engineering/plugins/compound-engineering.js");
    let content = std::fs::read_to_string(&dest).unwrap();
    assert!(content.contains("session.created"));
    assert!(content.contains("ce-ai"));
}

// ---- V1/V2 loader content validity (opencode-v2-plugin-loader) ----

/// The exact pre-V2 canonical loader shape: default export of an async
/// function returning a hooks object. OpenCode V2 fails it with
/// "Plugin must export a default definition with an id and an effect or
/// setup function", so ce-ai must classify it as outdated/stale.
const V1_LOADER_FIXTURE: &str = r#"import { spawnSync } from "child_process";

export const CompoundEngineeringPlugin = async ({ client }) => {
  return {
    event: async ({ event }) => {
      if (event && event.type === "session.created") {
        // handled
      }
    },
  };
};

export default CompoundEngineeringPlugin;
"#;

/// Minimal V2 loader shape: default export with `id` + `setup(ctx)` and the
/// SessionStart `session.created` marker.
const V2_LOADER_FIXTURE: &str = r#"export default {
  id: "compound-engineering",
  async setup(ctx) {
    void ctx.event.subscribe(() => {});
    // handles session.created and session.idle
    return () => {};
  },
};
"#;

#[test]
fn rejects_v1_function_export_loader() {
    assert!(!is_valid_loader_content(V1_LOADER_FIXTURE));
    // Even though it mentions session.created, the absence of the V2
    // `setup` signature makes it stale.
    assert!(V1_LOADER_FIXTURE.contains("session.created"));
}

#[test]
fn accepts_v2_setup_loader_and_builtin() {
    assert!(is_valid_loader_content(V2_LOADER_FIXTURE));
    // The embedded canonical loader must keep passing its own validity gate.
    assert!(is_valid_loader_content(BUILTIN_LOADER));
}

#[test]
fn accepts_legacy_ce_loader_function_exports() {
    // #325 loader-safety: function-export stubs (test fixtures, Dockerfile.e2e,
    // and newer-function-export fixtures like the v9 upgrade tarball) must
    // never be regressed by an older builtin.
    assert!(is_valid_loader_content(
        "export default function ceLoader() {}"
    ));
    assert!(is_valid_loader_content(
        "export default function ceLoaderV9() { /* session.created */ }\n"
    ));
}

#[test]
fn resolve_loader_bytes_substitutes_builtin_for_v1_source() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("v1-source");
    let loader_src = source.join(".opencode/plugins/compound-engineering.js");
    std::fs::create_dir_all(loader_src.parent().unwrap()).unwrap();
    std::fs::write(&loader_src, V1_LOADER_FIXTURE).unwrap();

    let bytes = resolve_loader_bytes(&source);
    let text = String::from_utf8(bytes).unwrap();
    assert!(
        text.contains("setup"),
        "V2 builtin substituted for V1 source"
    );
    assert_eq!(text, BUILTIN_LOADER);
}

#[test]
fn resolve_loader_bytes_uses_v2_source_as_is() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("v2-source");
    let loader_src = source.join(".opencode/plugins/compound-engineering.js");
    std::fs::create_dir_all(loader_src.parent().unwrap()).unwrap();
    std::fs::write(&loader_src, V2_LOADER_FIXTURE).unwrap();

    let bytes = resolve_loader_bytes(&source);
    assert_eq!(bytes, V2_LOADER_FIXTURE.as_bytes());
}

#[test]
fn has_session_start_plugin_false_for_v1_loader_on_disk() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("opencode-config");
    let loader_path = plugin_entry(&config_dir);
    std::fs::create_dir_all(loader_path.parent().unwrap()).unwrap();
    std::fs::write(&loader_path, V1_LOADER_FIXTURE).unwrap();
    let entry = loader_path.display().to_string();
    std::fs::write(
        config_dir.join("opencode.json"),
        serde_json::json!({ "plugin": [entry] }).to_string(),
    )
    .unwrap();

    assert!(
        !has_session_start_plugin(&config_dir),
        "V1-format loader must be reported outdated so doctor can surface it"
    );
}

#[test]
fn ensures_and_removes_session_start_plugin_lifecycle() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("opencode-config");

    assert!(!has_session_start_plugin(&config_dir));

    let changed = ensure_session_start_plugin(&config_dir).unwrap();
    assert!(changed);
    assert!(has_session_start_plugin(&config_dir));

    // Idempotent second call
    let changed_second = ensure_session_start_plugin(&config_dir).unwrap();
    assert!(!changed_second);
    assert!(has_session_start_plugin(&config_dir));

    // Remove
    let removed = remove_session_start_plugin(&config_dir).unwrap();
    assert!(removed);
    assert!(!has_session_start_plugin(&config_dir));

    let removed_second = remove_session_start_plugin(&config_dir).unwrap();
    assert!(!removed_second);
}

#[test]
fn preserves_user_plugins_in_opencode_json() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("opencode-config");
    std::fs::create_dir_all(&config_dir).unwrap();

    let initial = serde_json::json!({
        "plugin": ["custom-user-plugin", "@org/telemetry"],
        "model": "claude-3-5-sonnet"
    });
    std::fs::write(
        config_dir.join("opencode.json"),
        serde_json::to_string_pretty(&initial).unwrap(),
    )
    .unwrap();

    ensure_session_start_plugin(&config_dir).unwrap();
    let text = std::fs::read_to_string(config_dir.join("opencode.json")).unwrap();
    let val: serde_json::Value = serde_json::from_str(&text).unwrap();
    let plugins = val["plugin"].as_array().unwrap();
    assert_eq!(plugins.len(), 3);
    assert_eq!(plugins[0], "custom-user-plugin");
    assert_eq!(plugins[1], "@org/telemetry");
    assert_eq!(val["model"], "claude-3-5-sonnet");

    remove_session_start_plugin(&config_dir).unwrap();
    let text_after = std::fs::read_to_string(config_dir.join("opencode.json")).unwrap();
    let val_after: serde_json::Value = serde_json::from_str(&text_after).unwrap();
    let plugins_after = val_after["plugin"].as_array().unwrap();
    assert_eq!(plugins_after.len(), 2);
    assert_eq!(plugins_after[0], "custom-user-plugin");
    assert_eq!(plugins_after[1], "@org/telemetry");
    assert_eq!(val_after["model"], "claude-3-5-sonnet");
}
