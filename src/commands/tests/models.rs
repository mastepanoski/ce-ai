use super::*;
use crate::harness::agents::ORCHESTRATOR_AGENT as ORCHESTRATOR_SLOT;
use tempfile::TempDir;

/// Builds a fully hermetic Context pointing at temp dirs; avoids
/// `Context::resolve` so tests never touch the host opencode.json.
fn hermetic_ctx(tmp: &TempDir) -> Context {
    Context {
        config_dir: tmp.path().join("ce-ai"),
        opencode_config_dir: tmp.path().join("home/.config/opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    }
}

fn ctx_with_config(config: &str) -> (TempDir, Context) {
    let tmp = TempDir::new().unwrap();
    let ctx = hermetic_ctx(&tmp);
    std::fs::create_dir_all(&ctx.opencode_config_dir).unwrap();
    std::fs::write(ctx.opencode_config_dir.join("opencode.json"), config).unwrap();
    (tmp, ctx)
}

#[test]
fn syncs_across_all_active_harnesses() {
    let tmp = TempDir::new().unwrap();
    let ctx = hermetic_ctx(&tmp);
    std::fs::create_dir_all(&ctx.opencode_config_dir).unwrap();
    std::fs::write(
        ctx.opencode_config_dir.join("opencode.json"),
        r#"{"plugin":["user"]}"#,
    )
    .unwrap();

    set(
        &ctx,
        "opencode",
        "ce-brainstorm",
        "anthropic/claude-3-5-sonnet",
    )
    .unwrap();
    let state = State::load(&ctx.config_dir.join("state.json")).unwrap();
    assert_eq!(state.model_assignments.len(), 1);
    assert_eq!(
        state
            .model_assignments
            .get("ce-brainstorm")
            .unwrap()
            .model_id,
        "claude-3-5-sonnet"
    );
}

#[test]
fn parse_models_output_extracts_provider_model_tokens() {
    let text = "\
opencode/big-pickle
opencode-go/kimi-k2.6  (recommended)
anthropic/claude-sonnet-4-5   vision tools

not-a-model-line
/bad
also/bad/
";
    assert_eq!(
        parse_models_output(text),
        vec![
            "anthropic/claude-sonnet-4-5".to_string(),
            "opencode-go/kimi-k2.6".to_string(),
            "opencode/big-pickle".to_string(),
        ]
    );
}

#[test]
fn drift_findings_detect_divergent_and_missing_slots() {
    let mut state = State::new();
    state.set_model_assignment("ce-brainstorm", "anthropic", "claude-x");
    let config = serde_json::json!({
        "agent": {
            "ce-brainstorm": { "model": "user/custom-model" },
            ORCHESTRATOR_SLOT: { "model": "opencode-go/kimi-k2.6" },
            "third-party": { "model": "vendor/model" }
        }
    });
    let findings = model_drift_findings(&state, &config);
    assert!(findings
        .iter()
        .any(|f| f.contains("'ce-brainstorm'") && f.contains("config='user/custom-model'")));
    assert!(findings
        .iter()
        .any(|f| f.contains(ORCHESTRATOR_SLOT) && f.contains("untracked in state.json")));
    assert!(!findings.iter().any(|f| f.contains("third-party")));
}

#[test]
fn drift_findings_detect_state_slot_missing_from_config() {
    let mut state = State::new();
    state.set_model_assignment("ce-plan", "opencode-go", "kimi-k2.6");
    let findings = model_drift_findings(&state, &serde_json::json!({}));
    assert_eq!(findings.len(), 1);
    assert!(findings[0].contains("missing from opencode.json agent map"));
}

#[test]
fn import_config_assignments_repairs_desync() {
    let mut state = State::new();
    state.set_model_assignment("ce-plan", "old", "model");
    let config = serde_json::json!({
        "agent": {
            "ce-plan": { "model": "new/model" },
            "custom-slot": { "model": "user/model" },
            "broken-slot": { "model": "no-slash" }
        }
    });
    let imported = import_config_assignments(&mut state, &config);
    assert_eq!(
        imported,
        vec![
            ("ce-plan".to_string(), "new/model".to_string()),
            ("custom-slot".to_string(), "user/model".to_string()),
        ]
    );
    assert_eq!(state.model_assignments["ce-plan"].provider_id, "new");
    // Re-import is a no-op once state matches config.
    assert!(import_config_assignments(&mut state, &config).is_empty());
}

#[test]
fn purge_stale_assignments_removes_deleted_slots() {
    let mut state = State::new();
    state.set_model_assignment("ce-plan", "old", "model");
    state.set_model_assignment("ce-work", "kept", "model");
    // Config no longer has ce-plan: user deleted it there.
    let config = serde_json::json!({
        "agent": { "ce-work": { "model": "kept/model" } }
    });
    let purged = purge_stale_assignments(&mut state, &config);
    assert_eq!(purged, vec!["ce-plan".to_string()]);
    assert!(!state.model_assignments.contains_key("ce-plan"));
    assert!(state.model_assignments.contains_key("ce-work"));
}

#[test]
fn purge_stale_assignments_noop_without_agent_map() {
    let mut state = State::new();
    state.set_model_assignment("ce-plan", "p", "m");
    let purged = purge_stale_assignments(&mut state, &serde_json::json!({}));
    assert!(purged.is_empty());
    assert_eq!(state.model_assignments.len(), 1);
}

#[test]
fn set_writes_selected_harness_config() {
    let tmp = TempDir::new().unwrap();
    let ctx = hermetic_ctx(&tmp);
    std::fs::create_dir_all(&ctx.opencode_config_dir).unwrap();

    set(&ctx, "opencode", "ce-brainstorm", "user/custom-model").unwrap();
    let opencode_config = read_config(&ctx.opencode_config_dir.join("opencode.json")).unwrap();
    assert_eq!(
        opencode_config["agent"]["ce-brainstorm"]["model"],
        "user/custom-model"
    );

    // Non-OpenCode harnesses reject agent assignment explicitly.
    let err = set(&ctx, "claude", "ce-brainstorm", "a/b").unwrap_err();
    assert!(err.to_string().contains("no agent-map config"));
}

#[test]
fn config_assignments_reads_live_harness_config() {
    let (_tmp, ctx) = ctx_with_config(
        r#"{"agent":{"ce-brainstorm":{"model":"live/one"},"other":{"model":"x/y"}}}"#,
    );
    let scoped = config_assignments(&ctx, "opencode");
    assert_eq!(
        scoped,
        vec![("ce-brainstorm".to_string(), "live/one".to_string())]
    );
    // A harness with no config file yields nothing — never another
    // harness's list.
    assert!(config_assignments(&ctx, "kimi").is_empty());
}

#[test]
fn discovery_supported_only_for_opencode() {
    assert!(discovery_supported("opencode"));
    assert!(!discovery_supported("claude"));
}
