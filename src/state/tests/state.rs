use std::collections::BTreeMap;
use std::path::PathBuf;
use tempfile::tempdir;

use crate::state::state::{
    GuardLevel, GuardrailState, ModelAssignment, ReleaseProvenance, State, WorkflowSource,
    WorkflowStage,
};

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
        workflows: BTreeMap::new(),
        latest_release_tag: None,
        projects: vec![],
        skill_surfaces: vec![],
        guardrail: None,
        auto_checkpoint: None,
        review_receipts: BTreeMap::new(),
        gate_mode: None,
        gate_receipts: BTreeMap::new(),
        doc_hygiene: None,
        decisions: None,
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

#[test]
fn workspace_scoped_workflow_isolation() {
    use crate::state::state::WorkflowStage;
    let dir = tempdir().unwrap();
    let ws_a = dir.path().join("project_a");
    let ws_b = dir.path().join("project_b");
    std::fs::create_dir_all(&ws_a).unwrap();
    std::fs::create_dir_all(&ws_b).unwrap();

    let mut state = State::new();

    // Advance workspace A to Stage 4
    state
        .validate_and_set_workflow_for(
            &ws_a,
            WorkflowStage::Ideation,
            "Ideation A",
            Some("feat-a".into()),
        )
        .unwrap();
    state
        .validate_and_set_workflow_for(&ws_a, WorkflowStage::OpenSpec, "Spec A", None)
        .unwrap();
    state
        .validate_and_set_workflow_for(&ws_a, WorkflowStage::ExecutionPlan, "Plan A", None)
        .unwrap();
    state
        .validate_and_set_workflow_for(
            &ws_a,
            WorkflowStage::WorkTdd,
            "Deep in TDD A, unit 7/12",
            None,
        )
        .unwrap();

    // Verify workspace A is at Stage 4
    let wf_a = state.current_workflow_for(&ws_a).unwrap();
    assert_eq!(wf_a.stage, WorkflowStage::WorkTdd);
    assert_eq!(wf_a.task, "Deep in TDD A, unit 7/12");
    assert_eq!(wf_a.feature_name.as_deref(), Some("feat-a"));

    // Workspace B is still at default/legacy
    assert_eq!(state.workflows.len(), 1);

    // Set workspace B to Stage 1
    state
        .validate_and_set_workflow_for(
            &ws_b,
            WorkflowStage::Ideation,
            "Ideation B",
            Some("feat-b".into()),
        )
        .unwrap();

    // Verify workspace B is at Stage 1
    let wf_b = state.current_workflow_for(&ws_b).unwrap();
    assert_eq!(wf_b.stage, WorkflowStage::Ideation);
    assert_eq!(wf_b.task, "Ideation B");
    assert_eq!(wf_b.feature_name.as_deref(), Some("feat-b"));

    // CRITICAL: Verify workspace A is STILL at Stage 4 with feat-a
    let wf_a_after = state.current_workflow_for(&ws_a).unwrap();
    assert_eq!(wf_a_after.stage, WorkflowStage::WorkTdd);
    assert_eq!(wf_a_after.task, "Deep in TDD A, unit 7/12");
    assert_eq!(wf_a_after.feature_name.as_deref(), Some("feat-a"));
}

#[test]
fn legacy_workflow_fallback_and_deserialization() {
    use crate::state::state::WorkflowStage;
    let json_legacy = r#"{
        "version": 1,
        "workflow": {
            "stage": "executionplan",
            "task": "Planning v1",
            "feature_name": "legacy-feat",
            "updated_at": "2026-08-20T00:00:00Z"
        }
    }"#;

    let state: State = serde_json::from_str(json_legacy).unwrap();
    assert!(state.workflows.is_empty());
    assert!(state.workflow.is_some());

    let dir = tempdir().unwrap();
    let ws = dir.path().join("some_repo");
    std::fs::create_dir_all(&ws).unwrap();

    // current_workflow_for falls back to legacy workflow when not in workflows map
    let wf = state.current_workflow_for(&ws).unwrap();
    assert_eq!(wf.stage, WorkflowStage::ExecutionPlan);
    assert_eq!(wf.task, "Planning v1");
    assert_eq!(wf.feature_name.as_deref(), Some("legacy-feat"));
}

#[test]
fn branch_scoped_workflow_isolation_and_fallback() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("repo_branch");
    std::fs::create_dir_all(&ws).unwrap();

    let mut state = State::new();

    // Advance branch feat-a: Stage 1 -> Stage 2
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-a"),
            WorkflowStage::Ideation,
            "Ideation A",
            Some("feat-a".into()),
            WorkflowSource::Manual,
        )
        .unwrap();
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-a"),
            WorkflowStage::OpenSpec,
            "Specifying A",
            None,
            WorkflowSource::Manual,
        )
        .unwrap();

    // Advance branch feat-b: Stage 1 -> Stage 2 -> Stage 3 -> Stage 4
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-b"),
            WorkflowStage::Ideation,
            "Ideation B",
            Some("feat-b".into()),
            WorkflowSource::Manual,
        )
        .unwrap();
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-b"),
            WorkflowStage::OpenSpec,
            "Spec B",
            None,
            WorkflowSource::Manual,
        )
        .unwrap();
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-b"),
            WorkflowStage::ExecutionPlan,
            "Plan B",
            None,
            WorkflowSource::Manual,
        )
        .unwrap();
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-b"),
            WorkflowStage::WorkTdd,
            "Coding B",
            None,
            WorkflowSource::Manual,
        )
        .unwrap();

    // Verify branch isolation
    let wf_a = state
        .current_workflow_for_branch(&ws, Some("feat-a"))
        .unwrap();
    assert_eq!(wf_a.stage, WorkflowStage::OpenSpec);
    assert_eq!(wf_a.task, "Specifying A");
    assert_eq!(wf_a.feature_name.as_deref(), Some("feat-a"));

    let wf_b = state
        .current_workflow_for_branch(&ws, Some("feat-b"))
        .unwrap();
    assert_eq!(wf_b.stage, WorkflowStage::WorkTdd);
    assert_eq!(wf_b.task, "Coding B");
    assert_eq!(wf_b.feature_name.as_deref(), Some("feat-b"));

    // Querying without branch returns the most recently updated branch entry for that workspace
    let wf_latest = state.current_workflow_for_branch(&ws, None).unwrap();
    assert_eq!(wf_latest.stage, WorkflowStage::WorkTdd);
    assert_eq!(wf_latest.feature_name.as_deref(), Some("feat-b"));

    // Querying an unknown branch returns None because that branch has no recorded workflow
    assert!(state
        .current_workflow_for_branch(&ws, Some("feat-unknown"))
        .is_none());
}

#[test]
fn monotonic_provenance_guard_protects_manual_checkpoints() {
    let dir = tempdir().unwrap();
    let ws = dir.path().join("repo_guard");
    std::fs::create_dir_all(&ws).unwrap();

    let mut state = State::new();

    // 1. Advance to manual checkpoint at Stage 4 legally
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-guard"),
            WorkflowStage::Ideation,
            "Manual task unit 1",
            Some("feat-guard".into()),
            WorkflowSource::Manual,
        )
        .unwrap();
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-guard"),
            WorkflowStage::OpenSpec,
            "Manual task unit 2",
            None,
            WorkflowSource::Manual,
        )
        .unwrap();
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-guard"),
            WorkflowStage::ExecutionPlan,
            "Manual task unit 3",
            None,
            WorkflowSource::Manual,
        )
        .unwrap();
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-guard"),
            WorkflowStage::WorkTdd,
            "Manual task unit 4",
            None,
            WorkflowSource::Manual,
        )
        .unwrap();

    let initial = state
        .current_workflow_for_branch(&ws, Some("feat-guard"))
        .unwrap();
    assert_eq!(initial.stage, WorkflowStage::WorkTdd);
    assert_eq!(initial.source, WorkflowSource::Manual);
    assert_eq!(initial.task, "Manual task unit 4");

    // 2. Inferred transition to Stage 3 (regress) must be safely ignored
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-guard"),
            WorkflowStage::ExecutionPlan,
            "Inferred plan task",
            Some("feat-guard".into()),
            WorkflowSource::Inferred,
        )
        .unwrap();

    let after_regress = state
        .current_workflow_for_branch(&ws, Some("feat-guard"))
        .unwrap();
    assert_eq!(after_regress.stage, WorkflowStage::WorkTdd);
    assert_eq!(after_regress.task, "Manual task unit 4");
    assert_eq!(after_regress.source, WorkflowSource::Manual);

    // 3. Inferred transition at same Stage 4 must NOT overwrite manual task/source
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-guard"),
            WorkflowStage::WorkTdd,
            "Inferred same-stage task",
            Some("feat-guard".into()),
            WorkflowSource::Inferred,
        )
        .unwrap();

    let after_same = state
        .current_workflow_for_branch(&ws, Some("feat-guard"))
        .unwrap();
    assert_eq!(after_same.stage, WorkflowStage::WorkTdd);
    assert_eq!(after_same.task, "Manual task unit 4");
    assert_eq!(after_same.source, WorkflowSource::Manual);

    // 4. Inferred transition advancing to Stage 5 (valid transition) is allowed
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feat-guard"),
            WorkflowStage::Verification,
            "Inferred verification task",
            Some("feat-guard".into()),
            WorkflowSource::Inferred,
        )
        .unwrap();

    let after_advance = state
        .current_workflow_for_branch(&ws, Some("feat-guard"))
        .unwrap();
    assert_eq!(after_advance.stage, WorkflowStage::Verification);
    assert_eq!(after_advance.source, WorkflowSource::Inferred);

    // 5. Inferred invalid jump (e.g. from Stage 1 directly to Stage 5 on fresh branch) is ignored cleanly without error
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("fresh-branch"),
            WorkflowStage::Verification,
            "Invalid jump task",
            Some("fresh-branch".into()),
            WorkflowSource::Inferred,
        )
        .unwrap();
    // Since current stage was Ideation (1), and 1 -> 5 is illegal, inferred transition did not apply
    assert!(state
        .current_workflow_for_branch(&ws, Some("fresh-branch"))
        .is_none());
}

#[test]
fn atomic_update_workflow_persists_and_reloads() {
    let dir = tempdir().unwrap();
    let state_path = dir.path().join("state.json");
    let ws = dir.path().join("repo_atomic");
    std::fs::create_dir_all(&ws).unwrap();

    // Initial state saved to disk
    let initial_state = State::new();
    initial_state.save(&state_path).unwrap();

    // Update using atomic_update_workflow
    let updated = State::atomic_update_workflow(&state_path, &ws, Some("main"), |state| {
        state.validate_and_set_workflow_for_branch(
            &ws,
            Some("main"),
            WorkflowStage::OpenSpec,
            "Atomic update task",
            Some("spec-feat".into()),
            WorkflowSource::Manual,
        )?;
        Ok(state.current_workflow_for_branch(&ws, Some("main")))
    })
    .unwrap();

    assert_eq!(updated.stage, WorkflowStage::OpenSpec);
    assert_eq!(updated.task, "Atomic update task");

    // Reload from disk independently and verify persistence
    let reloaded = State::load(&state_path).unwrap();
    let wf = reloaded
        .current_workflow_for_branch(&ws, Some("main"))
        .unwrap();
    assert_eq!(wf.stage, WorkflowStage::OpenSpec);
    assert_eq!(wf.task, "Atomic update task");
    assert_eq!(wf.feature_name.as_deref(), Some("spec-feat"));
}

#[test]
fn legacy_workflow_source_deserialization_defaults_to_manual() {
    let json = r#"{
        "version": 1,
        "workflow": {
            "stage": "ideation",
            "task": "Brainstorming idea",
            "feature_name": null,
            "updated_at": "2026-09-01T00:00:00Z"
        }
    }"#;
    let state: State = serde_json::from_str(json).unwrap();
    let wf = state.workflow.unwrap();
    assert_eq!(wf.source, WorkflowSource::Manual);
}

#[test]
fn inferred_checkpoint_cannot_regress_previous_inferred_checkpoint() {
    let mut state = State::new();
    let ws = PathBuf::from("/tmp/test-repo-monotonic");

    // 1. Advance legally: Stage 1 -> Stage 2 -> Stage 3 with Inferred
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feature-1"),
            WorkflowStage::OpenSpec,
            "Specifying task",
            Some("feature-1".into()),
            WorkflowSource::Inferred,
        )
        .unwrap();
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feature-1"),
            WorkflowStage::ExecutionPlan,
            "Planning task",
            Some("feature-1".into()),
            WorkflowSource::Inferred,
        )
        .unwrap();

    let wf = state
        .current_workflow_for_branch(&ws, Some("feature-1"))
        .unwrap();
    assert_eq!(wf.stage, WorkflowStage::ExecutionPlan);
    assert_eq!(wf.source, WorkflowSource::Inferred);

    // 2. Attempt to regress from Stage 3 to Stage 2 (OpenSpec) with Inferred
    // Although can_transition_to allows 3 -> 2 as a rewind, Inferred monotonicity MUST block it as a no-op
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feature-1"),
            WorkflowStage::OpenSpec,
            "Regressed task",
            Some("feature-1".into()),
            WorkflowSource::Inferred,
        )
        .unwrap();

    let wf_after_regress = state
        .current_workflow_for_branch(&ws, Some("feature-1"))
        .unwrap();
    assert_eq!(
        wf_after_regress.stage,
        WorkflowStage::ExecutionPlan,
        "Inferred transition must not regress an existing Inferred stage"
    );
    assert_eq!(wf_after_regress.task, "Planning task");

    // 3. Advancing from Stage 3 to Stage 4 (WorkTdd) with Inferred succeeds
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feature-1"),
            WorkflowStage::WorkTdd,
            "Working task",
            Some("feature-1".into()),
            WorkflowSource::Inferred,
        )
        .unwrap();

    let wf_after_advance = state
        .current_workflow_for_branch(&ws, Some("feature-1"))
        .unwrap();
    assert_eq!(wf_after_advance.stage, WorkflowStage::WorkTdd);
    assert_eq!(wf_after_advance.task, "Working task");

    // 4. Manual transition CAN rewind from Stage 4 to Stage 3
    state
        .validate_and_set_workflow_for_branch(
            &ws,
            Some("feature-1"),
            WorkflowStage::ExecutionPlan,
            "Manual rewind task",
            Some("feature-1".into()),
            WorkflowSource::Manual,
        )
        .unwrap();

    let wf_after_manual = state
        .current_workflow_for_branch(&ws, Some("feature-1"))
        .unwrap();
    assert_eq!(wf_after_manual.stage, WorkflowStage::ExecutionPlan);
    assert_eq!(wf_after_manual.source, WorkflowSource::Manual);
}

#[test]
fn gate_mode_and_gate_receipts_serialization_and_state_roundtrip() {
    use crate::state::state::{GateDecision, GateMode, GateReceipt};

    let receipt = GateReceipt {
        timestamp: "2026-09-11T22:00:00Z".to_string(),
        feature: "auth-flow".to_string(),
        target_path: "src/auth.rs".to_string(),
        decision: GateDecision::Blocked,
        stage: Some(4),
        entry_point: Some("ce-work".to_string()),
        tier: "full".to_string(),
        missing_artifacts: vec!["proposal.md".to_string(), "tasks.md".to_string()],
        reason: "Stage 4 without OpenSpec".to_string(),
    };

    let serialized = serde_json::to_string(&receipt).unwrap();
    assert!(serialized.contains(r#""decision":"blocked""#));
    assert!(serialized.contains(r#""feature":"auth-flow""#));

    let deserialized: GateReceipt = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.decision, GateDecision::Blocked);
    assert_eq!(deserialized.missing_artifacts.len(), 2);

    let mut state = State::new();
    assert_eq!(state.gate_mode, None);
    assert!(state.gate_receipts.is_empty());

    state.gate_mode = Some(GateMode::Enforce);
    state
        .gate_receipts
        .insert("auth-flow".to_string(), receipt.clone());

    let state_json = serde_json::to_string(&state).unwrap();
    let state_restored: State = serde_json::from_str(&state_json).unwrap();
    assert_eq!(state_restored.gate_mode, Some(GateMode::Enforce));
    assert_eq!(state_restored.gate_receipts.len(), 1);
    assert_eq!(
        state_restored.gate_receipts.get("auth-flow"),
        Some(&receipt)
    );
}

#[test]
fn test_doc_hygiene_config_defaults_and_serde() {
    use crate::state::state::DocHygieneConfig;

    let default_cfg = DocHygieneConfig::default();
    assert_eq!(default_cfg.stale_spec_days, 21);
    assert!(default_cfg.check_solution_paths);
    assert!(default_cfg.require_solution_frontmatter);
    assert_eq!(default_cfg.archive_compaction_threshold, 30);

    // Deserializing empty JSON object should yield defaults
    let empty_json = "{}";
    let deserialized: DocHygieneConfig = serde_json::from_str(empty_json).unwrap();
    assert_eq!(deserialized, default_cfg);

    // Partial JSON override preserves omitted defaults
    let partial_json = r#"{"stale_spec_days": 14, "archive_compaction_threshold": 50}"#;
    let partial_cfg: DocHygieneConfig = serde_json::from_str(partial_json).unwrap();
    assert_eq!(partial_cfg.stale_spec_days, 14);
    assert!(partial_cfg.check_solution_paths);
    assert!(partial_cfg.require_solution_frontmatter);
    assert_eq!(partial_cfg.archive_compaction_threshold, 50);

    // Explicit bool override
    let bool_override_json = r#"{"check_solution_paths": false}"#;
    let bool_cfg: DocHygieneConfig = serde_json::from_str(bool_override_json).unwrap();
    assert_eq!(bool_cfg.stale_spec_days, 21);
    assert!(!bool_cfg.check_solution_paths);
    assert!(bool_cfg.require_solution_frontmatter);
    assert_eq!(bool_cfg.archive_compaction_threshold, 30);
}

#[test]
fn test_doc_hygiene_workspace_overrides_precedence() {
    use crate::state::state::DocHygieneConfig;

    let dir = tempdir().unwrap();
    let global_path = dir.path().join("global_state.json");
    let ws_dir = dir.path().join("workspace");
    std::fs::create_dir_all(&ws_dir).unwrap();
    let local_path = ws_dir.join(".ce-ai.json");

    let mut global = State::new();
    global.doc_hygiene = Some(DocHygieneConfig::default());
    global.save(&global_path).unwrap();

    let mut local = State::new();
    local.doc_hygiene = Some(DocHygieneConfig {
        stale_spec_days: 7,
        check_solution_paths: false,
        require_solution_frontmatter: true,
        archive_compaction_threshold: 40,
    });
    local.save(&local_path).unwrap();

    let loaded = State::load_with_workspace_overrides(&global_path, Some(&ws_dir)).unwrap();
    let effective = loaded.doc_hygiene();
    assert_eq!(effective.stale_spec_days, 7);
    assert!(!effective.check_solution_paths);
    assert!(effective.require_solution_frontmatter);
    assert_eq!(effective.archive_compaction_threshold, 40);
}

#[test]
fn test_doc_hygiene_helper_fallback() {
    use crate::state::state::DocHygieneConfig;

    let state_without = State::new();
    assert_eq!(state_without.doc_hygiene, None);
    assert_eq!(state_without.doc_hygiene(), DocHygieneConfig::default());

    let mut state_with = State::new();
    state_with.doc_hygiene = Some(DocHygieneConfig {
        stale_spec_days: 45,
        check_solution_paths: true,
        require_solution_frontmatter: false,
        archive_compaction_threshold: 15,
    });
    assert_eq!(state_with.doc_hygiene().stale_spec_days, 45);
    assert!(!state_with.doc_hygiene().require_solution_frontmatter);
    assert_eq!(state_with.doc_hygiene().archive_compaction_threshold, 15);
}

#[test]
fn test_execution_mode_parse_and_display() {
    use crate::state::state::ExecutionMode;

    assert_eq!(ExecutionMode::parse("auto").unwrap(), ExecutionMode::Auto);
    assert_eq!(
        ExecutionMode::parse("organic").unwrap(),
        ExecutionMode::Organic
    );
    assert_eq!(ExecutionMode::parse("odd").unwrap(), ExecutionMode::Organic);
    assert_eq!(ExecutionMode::parse("ODD").unwrap(), ExecutionMode::Organic);
    assert_eq!(
        ExecutionMode::parse("compound").unwrap(),
        ExecutionMode::Compound
    );
    assert_eq!(ExecutionMode::parse("ce").unwrap(), ExecutionMode::Compound);
    assert_eq!(
        ExecutionMode::parse("openspec").unwrap(),
        ExecutionMode::Compound
    );
    assert!(ExecutionMode::parse("invalid-mode").is_err());

    assert_eq!(ExecutionMode::Auto.as_str(), "auto");
    assert_eq!(ExecutionMode::Organic.as_str(), "organic");
    assert_eq!(ExecutionMode::Compound.as_str(), "compound");
    assert_eq!(format!("{}", ExecutionMode::Organic), "organic");
}

#[test]
fn test_workflow_state_execution_mode_serialization_roundtrip() {
    use crate::state::state::{ExecutionMode, WorkflowSource, WorkflowStage, WorkflowState};

    let wf = WorkflowState {
        stage: WorkflowStage::WorkTdd,
        task: "Fix parser bug".to_string(),
        feature_name: Some("fix-parser".to_string()),
        updated_at: "2026-09-16T12:00:00Z".to_string(),
        source: WorkflowSource::Manual,
        resolution: None,
        new_cycle: false,
        execution_mode: Some(ExecutionMode::Organic),
    };

    let json = serde_json::to_string(&wf).unwrap();
    assert!(json.contains(r#""execution_mode":"organic""#));

    let deserialized: WorkflowState = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.execution_mode, Some(ExecutionMode::Organic));

    // Deserializing legacy JSON without execution_mode field
    let legacy_json = r#"{
        "stage": "worktdd",
        "task": "Legacy task",
        "updated_at": "2026-09-16T12:00:00Z",
        "source": "manual",
        "new_cycle": false
    }"#;
    let legacy_wf: WorkflowState = serde_json::from_str(legacy_json).unwrap();
    assert_eq!(legacy_wf.execution_mode, None);
}

#[test]
fn test_state_set_execution_mode_for_branch() {
    use crate::state::state::{ExecutionMode, State, WorkflowStage};
    use std::path::Path;

    let root = Path::new("/workspace/test");
    let mut state = State::new();
    state
        .validate_and_set_workflow_for_branch(
            root,
            Some("fix/parser"),
            WorkflowStage::Ideation,
            "fixing parser",
            Some("fix-parser".to_string()),
            crate::state::state::WorkflowSource::Manual,
        )
        .unwrap();

    let initial = state
        .current_workflow_for_branch(root, Some("fix/parser"))
        .unwrap();
    assert_eq!(initial.execution_mode, None);

    state.set_execution_mode_for_branch(root, Some("fix/parser"), Some(ExecutionMode::Organic));
    let updated = state
        .current_workflow_for_branch(root, Some("fix/parser"))
        .unwrap();
    assert_eq!(updated.execution_mode, Some(ExecutionMode::Organic));
}
