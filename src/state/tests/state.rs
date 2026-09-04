use std::collections::BTreeMap;
use tempfile::tempdir;

use crate::state::state::{GuardLevel, GuardrailState, ModelAssignment, ReleaseProvenance, State};

fn state_with(slot: &str) -> State {
    State {
        version: 1,
        installed_harnesses: vec![],
        managed_asset_digest: BTreeMap::new(),
        release_provenance: None,
        model_assignments: BTreeMap::from([(
            slot.to_string(),
            ModelAssignment {
                provider_id: "opencode-go".into(),
                model_id: "kimi-k2.6".into(),
                effort: None,
            },
        )]),
        last_update_check: None,
        workflow: None,
        latest_release_tag: None,
        projects: vec![],
        skill_surfaces: vec![],
        guardrail: None,
    }
}

#[test]
fn round_trips_through_state_json() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state.json");
    let state = state_with("ce-brainstorm");
    state.save(&path).unwrap();
    assert_eq!(State::load(&path).unwrap(), state);
}

#[test]
fn skill_surfaces_ledger_round_trips_and_defaults_empty() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut state = State::new();
    state
        .skill_surfaces
        .push(crate::state::state::SkillSurface {
            harness: "opencode".into(),
            root: std::path::PathBuf::from("/tmp/skills"),
            status: "adopted".into(),
            files: vec![crate::state::state::SkillSurfaceFile {
                path: "ce-brainstorm/SKILL.md".into(),
                sha256: "a".repeat(64),
            }],
            adopted_at: Some("2026-08-24T00:00:00Z".into()),
        });
    state.save(&path).unwrap();
    let loaded = State::load(&path).unwrap();
    assert_eq!(loaded.skill_surfaces.len(), 1);
    assert_eq!(loaded.skill_surfaces[0].status, "adopted");

    // Legacy state.json without the ledger loads with an empty default.
    let legacy = dir.path().join("legacy.json");
    std::fs::write(&legacy, r#"{"version":1}"#).unwrap();
    assert!(State::load(&legacy).unwrap().skill_surfaces.is_empty());
}

#[test]
fn atomic_write_leaves_no_temp_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state.json");
    state_with("ce-brainstorm").save(&path).unwrap();
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn persists_model_assignments_across_reloads() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut state = State::new();
    state.set_model_assignment("ce-brainstorm", "opencode-go", "kimi-k2.6");
    state.save(&path).unwrap();
    let assignment = &State::load(&path).unwrap().model_assignments["ce-brainstorm"];
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

#[test]
fn workspace_overrides_precedence() {
    let dir = tempdir().unwrap();
    let global_path = dir.path().join("global_state.json");
    let ws_dir = dir.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();
    let local_path = ws_dir.join(".ce-ai.json");

    let mut global = State::new();
    global.set_model_assignment("ce-brainstorm", "opencode-go", "kimi-k2.6");
    global.set_model_assignment("ce-work", "anthropic", "claude-3-5-sonnet");
    global.save(&global_path).unwrap();

    let mut local = State::new();
    local.set_model_assignment("ce-work", "openai", "gpt-4o");
    local.save(&local_path).unwrap();

    let loaded = State::load_with_workspace_overrides(&global_path, Some(&ws_dir)).unwrap();
    assert_eq!(
        loaded.model_assignments["ce-brainstorm"].provider_id,
        "opencode-go"
    );
    assert_eq!(loaded.model_assignments["ce-work"].provider_id, "openai");
    assert_eq!(loaded.model_assignments["ce-work"].model_id, "gpt-4o");
}

#[test]
fn project_adoption_entry_roundtrip() {
    use crate::state::state::{AdoptionTier, ProjectAdoptionEntry};
    use std::path::PathBuf;

    let dir = tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut state = State::new();
    state.projects.push(ProjectAdoptionEntry {
        path: PathBuf::from("/tmp/repo"),
        file: "AGENTS.md".into(),
        tier: AdoptionTier::Full,
        block_version: 1,
        block_sha256: "abc123sha".into(),
        created_file: true,
        adopted_at: "2026-08-22T00:00:00Z".into(),
    });
    state.save(&path).unwrap();

    let loaded = State::load(&path).unwrap();
    assert_eq!(loaded.projects.len(), 1);
    assert_eq!(loaded.projects[0].path, PathBuf::from("/tmp/repo"));
    assert_eq!(loaded.projects[0].tier, AdoptionTier::Full);
    assert!(loaded.projects[0].created_file);
}

#[test]
fn release_provenance_round_trips_through_state_json() {
    use std::path::PathBuf;

    let dir = tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut state = State::new();
    state
        .managed_asset_digest
        .insert("tarball".into(), "sha256:deadbeef".into());
    state.release_provenance = Some(ReleaseProvenance {
        tag: "v1.18.0".into(),
        url: "https://github.com/everyinc/compound-engineering-plugin/archive/refs/tags/v1.18.0.tar.gz".into(),
        archive_sha256: "deadbeef".into(),
        extraction_path: PathBuf::from("/tmp/ce-ai/cache/trees/v1.18.0"),
    });
    state.save(&path).unwrap();

    let loaded = State::load(&path).unwrap();
    let prov = loaded.release_provenance.expect("provenance persisted");
    assert_eq!(prov.tag, "v1.18.0");
    assert_eq!(prov.archive_sha256, "deadbeef");
    assert_eq!(
        loaded
            .managed_asset_digest
            .get("tarball")
            .map(String::as_str),
        Some("sha256:deadbeef")
    );
}

#[test]
fn legacy_state_without_provenance_loads() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state.json");
    let legacy = serde_json::json!({
        "version": 1,
        "installed_harnesses": [],
        "managed_asset_digest": { "tarball": "sha256:abc" }
    });
    std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();

    let loaded = State::load(&path).unwrap();
    assert!(loaded.release_provenance.is_none());
    assert!(loaded.guardrail.is_none());
    assert_eq!(
        loaded
            .managed_asset_digest
            .get("tarball")
            .map(String::as_str),
        Some("sha256:abc")
    );
}

#[test]
fn guardrail_state_round_trips_and_parses() {
    assert_eq!(GuardLevel::parse("junior").unwrap(), GuardLevel::Junior);
    assert_eq!(GuardLevel::parse("strict").unwrap(), GuardLevel::Strict);
    assert!(GuardLevel::parse("unknown").is_err());

    let dir = tempdir().unwrap();
    let path = dir.path().join("state.json");
    let mut state = State::new();
    state.guardrail = Some(GuardrailState {
        enabled: true,
        level: GuardLevel::Strict,
        harness: Some("claude".into()),
        updated_at: "2026-08-28T00:00:00Z".into(),
    });
    state.save(&path).unwrap();

    let loaded = State::load(&path).unwrap();
    let guard = loaded.guardrail.expect("guardrail should exist");
    assert!(guard.enabled);
    assert_eq!(guard.level, GuardLevel::Strict);
    assert_eq!(guard.harness.as_deref(), Some("claude"));
    assert_eq!(guard.updated_at, "2026-08-28T00:00:00Z");
}

#[test]
fn reset_to_stage_1_without_feature_clears_previous_feature() {
    let mut state = State::new();
    state
        .validate_and_set_workflow(
            crate::state::state::WorkflowStage::OpenSpec,
            "specifying feature A",
            Some("feature-a".into()),
        )
        .unwrap();
    assert_eq!(
        state.current_workflow().unwrap().feature_name.as_deref(),
        Some("feature-a")
    );

    // Resetting to Stage 1 without --feature MUST clear the previous feature
    state
        .validate_and_set_workflow(
            crate::state::state::WorkflowStage::Ideation,
            "ideation for feature B",
            None,
        )
        .unwrap();
    assert_eq!(
        state.current_workflow().unwrap().feature_name,
        None,
        "reset to Stage 1 without explicit feature must clear feature_name"
    );
}

#[test]
fn advance_stage_without_feature_inherits_previous_feature() {
    let mut state = State::new();
    state
        .validate_and_set_workflow(
            crate::state::state::WorkflowStage::OpenSpec,
            "specifying feature A",
            Some("feature-a".into()),
        )
        .unwrap();

    // Advancing from Stage 2 to Stage 3 without --feature MUST inherit feature-a
    state
        .validate_and_set_workflow(
            crate::state::state::WorkflowStage::ExecutionPlan,
            "planning feature A",
            None,
        )
        .unwrap();
    assert_eq!(
        state.current_workflow().unwrap().feature_name.as_deref(),
        Some("feature-a")
    );
}

#[test]
fn reset_to_stage_1_with_explicit_feature_sets_new_feature() {
    let mut state = State::new();
    state
        .validate_and_set_workflow(
            crate::state::state::WorkflowStage::OpenSpec,
            "specifying feature A",
            Some("feature-a".into()),
        )
        .unwrap();

    // Resetting to Stage 1 with explicit --feature sets the new feature
    state
        .validate_and_set_workflow(
            crate::state::state::WorkflowStage::Ideation,
            "ideation for feature B",
            Some("feature-b".into()),
        )
        .unwrap();
    assert_eq!(
        state.current_workflow().unwrap().feature_name.as_deref(),
        Some("feature-b")
    );
}

#[test]
fn explicit_empty_feature_clears_feature_name() {
    let mut state = State::new();
    state
        .validate_and_set_workflow(
            crate::state::state::WorkflowStage::OpenSpec,
            "specifying feature A",
            Some("feature-a".into()),
        )
        .unwrap();

    // Passing empty string explicitly clears the feature
    state
        .validate_and_set_workflow(
            crate::state::state::WorkflowStage::ExecutionPlan,
            "planning tasks",
            Some("".into()),
        )
        .unwrap();
    assert_eq!(
        state.current_workflow().unwrap().feature_name,
        None,
        "explicit empty feature must clear feature_name"
    );

    // Whitespace-only string also clears the feature
    state
        .validate_and_set_workflow(
            crate::state::state::WorkflowStage::WorkTdd,
            "working",
            Some("   ".into()),
        )
        .unwrap();
    assert_eq!(
        state.current_workflow().unwrap().feature_name,
        None,
        "whitespace feature must clear feature_name"
    );
}
