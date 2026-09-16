use std::sync::Mutex;

use crate::commands::gate::{
    evaluate_gate_decision, is_gate_kill_switched, GateDecision, GateEdgeCase,
};
use crate::state::state::{FeatureResolution, WorkflowStage};

static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_kill_switch_active_returns_true_for_env_vars_and_flag() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // Flag takes precedence
    assert!(is_gate_kill_switched(true));

    // When flag is false, check environment variables
    for val in ["1", "true", "yes", " 1 "] {
        std::env::set_var("CE_AI_DISABLE_GATE_CHECK", val);
        std::env::remove_var("CE_AI_GATE_CHECK_DISABLED");
        assert!(
            is_gate_kill_switched(false),
            "expected true for CE_AI_DISABLE_GATE_CHECK={val}"
        );
    }
    std::env::remove_var("CE_AI_DISABLE_GATE_CHECK");

    for val in ["1", "true", "yes"] {
        std::env::remove_var("CE_AI_DISABLE_GATE_CHECK");
        std::env::set_var("CE_AI_GATE_CHECK_DISABLED", val);
        assert!(
            is_gate_kill_switched(false),
            "expected true for CE_AI_GATE_CHECK_DISABLED={val}"
        );
    }
    std::env::remove_var("CE_AI_GATE_CHECK_DISABLED");

    // Negative values
    for val in ["0", "false", "no", "random"] {
        std::env::set_var("CE_AI_DISABLE_GATE_CHECK", val);
        std::env::set_var("CE_AI_GATE_CHECK_DISABLED", val);
        assert!(!is_gate_kill_switched(false), "expected false for {val}");
    }
    std::env::remove_var("CE_AI_DISABLE_GATE_CHECK");
    std::env::remove_var("CE_AI_GATE_CHECK_DISABLED");

    assert!(!is_gate_kill_switched(false));
}

#[test]
fn test_pure_decision_engine_stage_4_missing_artifacts_would_block() {
    // Missing proposal.md
    let (decision, edge, reason) = evaluate_gate_decision(
        Some(WorkflowStage::WorkTdd),
        Some("my-feat"),
        Some(FeatureResolution::Branch),
        false,
        false,
        false, // has_proposal
        true,  // has_spec
        true,  // has_tasks
    );
    assert_eq!(decision, GateDecision::WouldBlock);
    assert_eq!(edge, None);
    assert!(reason.contains("proposal.md"));

    // Missing spec.md
    let (decision, edge, reason) = evaluate_gate_decision(
        Some(WorkflowStage::WorkTdd),
        Some("my-feat"),
        Some(FeatureResolution::Branch),
        false,
        false,
        true,
        false, // has_spec
        true,
    );
    assert_eq!(decision, GateDecision::WouldBlock);
    assert_eq!(edge, None);
    assert!(reason.contains("spec.md"));

    // Missing tasks.md
    let (decision, edge, reason) = evaluate_gate_decision(
        Some(WorkflowStage::WorkTdd),
        Some("my-feat"),
        Some(FeatureResolution::Branch),
        false,
        false,
        true,
        true,
        false, // has_tasks
    );
    assert_eq!(decision, GateDecision::WouldBlock);
    assert_eq!(edge, None);
    assert!(reason.contains("tasks.md"));
}

#[test]
fn test_pure_decision_engine_stage_4_complete_artifacts_pass() {
    let (decision, edge, reason) = evaluate_gate_decision(
        Some(WorkflowStage::WorkTdd),
        Some("my-feat"),
        Some(FeatureResolution::Branch),
        false,
        false,
        true,
        true,
        true,
    );
    assert_eq!(decision, GateDecision::Pass);
    assert_eq!(edge, None);
    assert!(reason.contains("complete OpenSpec contract"));
}

#[test]
fn test_pure_decision_engine_non_stage_4_pass() {
    for stage in [
        WorkflowStage::Ideation,
        WorkflowStage::OpenSpec,
        WorkflowStage::ExecutionPlan,
        WorkflowStage::Verification,
        WorkflowStage::KnowledgeCapture,
        WorkflowStage::GitShipping,
    ] {
        let (decision, edge, _) = evaluate_gate_decision(
            Some(stage),
            Some("my-feat"),
            Some(FeatureResolution::Branch),
            false,
            false,
            false, // no artifacts required for non-stage 4
            false,
            false,
        );
        assert_eq!(
            decision,
            GateDecision::Pass,
            "Stage {:?} should pass",
            stage
        );
        assert_eq!(edge, None);
    }
}

#[test]
fn test_pure_decision_engine_undetermined_when_no_checkpoint() {
    let (decision, edge, reason) =
        evaluate_gate_decision(None, None, None, false, false, false, false, false);
    assert_eq!(decision, GateDecision::Undetermined);
    assert_eq!(edge, None);
    assert!(reason.contains("no active workflow checkpoint"));
}

#[test]
fn test_pure_decision_engine_edge_cases_isolated() {
    // 1. Mtime Fallback
    let (decision, edge, _) = evaluate_gate_decision(
        Some(WorkflowStage::WorkTdd),
        Some("fallback-feat"),
        Some(FeatureResolution::MtimeFallback),
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(decision, GateDecision::EdgeCase);
    assert_eq!(edge, Some(GateEdgeCase::MtimeFallback));

    // 2. Worktree Uncommitted Spec
    let (decision, edge, _) = evaluate_gate_decision(
        Some(WorkflowStage::WorkTdd),
        Some("uncommitted-feat"),
        Some(FeatureResolution::Branch),
        false,
        true, // has_uncommitted_spec
        false,
        false,
        false,
    );
    assert_eq!(decision, GateDecision::EdgeCase);
    assert_eq!(edge, Some(GateEdgeCase::WorktreeUncommitted));

    // 3. Stale Cycle Guard (new cycle reset detected)
    let (decision, edge, _) = evaluate_gate_decision(
        Some(WorkflowStage::WorkTdd),
        Some("stale-feat"),
        Some(FeatureResolution::Branch),
        true, // is_new_cycle_task
        false,
        false,
        false,
        false,
    );
    assert_eq!(decision, GateDecision::EdgeCase);
    assert_eq!(edge, Some(GateEdgeCase::StaleCycleGuard));
}

#[test]
fn test_evaluate_gate_policy_matrix_exemptions_and_enforcement() {
    use crate::commands::gate::evaluate_gate_policy;
    use crate::state::state::{AdoptionTier, ExecutionMode, GateMode};

    // 1. Enforce mode blocks Stage 4 with missing artifacts and returns missing list in Compound mode
    let (decision, edge, missing, reason) = evaluate_gate_policy(
        Some(WorkflowStage::WorkTdd),
        Some("blocked-feat"),
        Some(FeatureResolution::Branch),
        false,
        false,
        AdoptionTier::Full,
        Some("ce-work"),
        false, // missing proposal
        true,
        false, // missing tasks
        GateMode::Enforce,
        ExecutionMode::Compound,
    );
    assert_eq!(decision, GateDecision::Blocked);
    assert_eq!(edge, None);
    assert_eq!(missing, vec!["proposal.md", "tasks.md"]);
    assert!(reason.contains("Stage 4 (ce-work) active for 'blocked-feat'"));

    // 2. Observe mode returns WouldBlock for same scenario in Compound mode
    let (decision, edge, missing, _) = evaluate_gate_policy(
        Some(WorkflowStage::WorkTdd),
        Some("observe-feat"),
        Some(FeatureResolution::Branch),
        false,
        false,
        AdoptionTier::Full,
        Some("ce-work"),
        false,
        true,
        false,
        GateMode::Observe,
        ExecutionMode::Compound,
    );
    assert_eq!(decision, GateDecision::WouldBlock);
    assert_eq!(edge, None);
    assert_eq!(missing, vec!["proposal.md", "tasks.md"]);

    // 3. Direct Entry Point: ce-debug exempt from OpenSpec requirement
    for entry in [
        "ce-debug",
        "ce-debug: fix auth race condition",
        "DEBUG: crash on start",
    ] {
        let (decision, edge, missing, reason) = evaluate_gate_policy(
            Some(WorkflowStage::WorkTdd),
            Some("debug-feat"),
            Some(FeatureResolution::Branch),
            false,
            false,
            AdoptionTier::Full,
            Some(entry),
            false, // no proposal
            false, // no spec
            false, // no tasks
            GateMode::Enforce,
            ExecutionMode::Compound,
        );
        assert_eq!(decision, GateDecision::Pass);
        assert_eq!(edge, None);
        assert!(missing.is_empty());
        assert!(reason.contains("ce-debug direct entry point permits bug fix writes"));
    }

    // 4. AdoptionTier::Minimal exempt from full OpenSpec requirement
    let (decision, edge, missing, reason) = evaluate_gate_policy(
        Some(WorkflowStage::WorkTdd),
        Some("minimal-feat"),
        Some(FeatureResolution::Branch),
        false,
        false,
        AdoptionTier::Minimal,
        Some("ce-work"),
        false, // no proposal
        false, // no spec
        false, // no tasks
        GateMode::Enforce,
        ExecutionMode::Compound,
    );
    assert_eq!(decision, GateDecision::Pass);
    assert_eq!(edge, None);
    assert!(missing.is_empty());
    assert!(reason.contains("project tier minimal permits writes without full OpenSpec"));

    // 5. Stage != 4 passes regardless of missing artifacts
    let (decision, edge, missing, _) = evaluate_gate_policy(
        Some(WorkflowStage::ExecutionPlan),
        Some("plan-feat"),
        Some(FeatureResolution::Branch),
        false,
        false,
        AdoptionTier::Full,
        Some("ce-plan"),
        false,
        false,
        false,
        GateMode::Enforce,
        ExecutionMode::Compound,
    );
    assert_eq!(decision, GateDecision::Pass);
    assert_eq!(edge, None);
    assert!(missing.is_empty());

    // 6. Edge cases never block, even in enforce mode with missing artifacts
    let (decision, edge, missing, _) = evaluate_gate_policy(
        Some(WorkflowStage::WorkTdd),
        Some("edge-feat"),
        Some(FeatureResolution::MtimeFallback),
        false,
        false,
        AdoptionTier::Full,
        Some("ce-work"),
        false,
        false,
        false,
        GateMode::Enforce,
        ExecutionMode::Compound,
    );
    assert_eq!(decision, GateDecision::EdgeCase);
    assert_eq!(edge, Some(GateEdgeCase::MtimeFallback));
    assert!(missing.is_empty());

    // 7. Organic execution mode permits writes without OpenSpec contract
    let (decision, edge, missing, reason) = evaluate_gate_policy(
        Some(WorkflowStage::WorkTdd),
        Some("odd-feat"),
        Some(FeatureResolution::Branch),
        false,
        false,
        AdoptionTier::Full,
        Some("ce-work"),
        false, // no proposal
        false, // no spec
        false, // no tasks
        GateMode::Enforce,
        ExecutionMode::Organic,
    );
    assert_eq!(decision, GateDecision::Pass);
    assert_eq!(edge, None);
    assert!(missing.is_empty());
    assert!(
        reason.contains("organic execution mode permits writes without formal OpenSpec contract")
    );
}

#[test]
fn test_format_blocked_remediation_message() {
    use crate::commands::gate::format_blocked_remediation_message;

    let missing = vec![
        "proposal.md".to_string(),
        "spec.md".to_string(),
        "tasks.md".to_string(),
    ];
    let msg = format_blocked_remediation_message(
        "src/commands/foo.rs",
        "Stage 4 (WorkTdd)",
        "awesome-feature",
        &missing,
    );

    assert!(msg.contains("ce-ai gate check: Write blocked on 'src/commands/foo.rs'"));
    assert!(msg.contains("Active Stage: Stage 4 (WorkTdd) for feature 'awesome-feature'"));
    assert!(msg.contains("✖ proposal.md — Missing problem statement, boundaries & risk evaluation"));
    assert!(msg.contains("✖ spec.md — Missing formal WHEN/THEN requirements & acceptance criteria"));
    assert!(msg.contains("✖ tasks.md — Missing implementation checklist with ~200 LOC work units"));
    assert!(msg
        .contains("Run `/ce-plan` or create missing files in `openspec/changes/awesome-feature/`"));
    assert!(msg.contains("ce-ai workflow checkpoint --stage 4 --task \"ce-debug: <issue>\""));
    assert!(msg.contains("CE_AI_DISABLE_GATE_CHECK=1"));
}

#[test]
fn test_write_gate_receipt_atomic_and_state_persistence() {
    use crate::commands::gate::{write_gate_receipt, GateReceipt};
    use crate::commands::Context;
    use crate::state::state::{GateDecision, State};
    use tempfile::tempdir;

    let tmp_home = tempdir().unwrap();
    let tmp_repo = tempdir().unwrap();
    let config_dir = tmp_home.path().join(".ce-ai");
    std::fs::create_dir_all(&config_dir).unwrap();

    let state = State::new();
    state.save(&config_dir.join("state.json")).unwrap();

    let change_dir = tmp_repo
        .path()
        .join("openspec")
        .join("changes")
        .join("test-feature");
    std::fs::create_dir_all(&change_dir).unwrap();

    let ctx = Context {
        config_dir: config_dir.clone(),
        opencode_config_dir: tmp_home.path().join(".config/opencode"),
        workspace_root: Some(tmp_repo.path().to_path_buf()),
        dry_run: false,
        verbose: false,
        quiet: false,
    };

    let receipt = GateReceipt {
        timestamp: "2026-09-11T22:30:00Z".to_string(),
        feature: "test-feature".to_string(),
        target_path: "src/main.rs".to_string(),
        decision: GateDecision::Blocked,
        stage: Some(4),
        entry_point: Some("ce-work".to_string()),
        tier: "full".to_string(),
        missing_artifacts: vec!["tasks.md".to_string()],
        reason: "missing tasks.md".to_string(),
    };

    write_gate_receipt(&ctx, tmp_repo.path(), &receipt).unwrap();

    // 1. Verify .validation.json was written to openspec/changes/test-feature/.validation.json
    let validation_path = change_dir.join(".validation.json");
    assert!(validation_path.is_file());
    let val_content = std::fs::read_to_string(&validation_path).unwrap();
    assert!(val_content.contains(r#""decision": "blocked""#));
    assert!(val_content.contains(r#""target_path": "src/main.rs""#));

    // 2. Verify state.json was updated with gate_receipts
    let state_reloaded = State::load(&config_dir.join("state.json")).unwrap();
    assert_eq!(state_reloaded.gate_receipts.len(), 1);
    let stored = state_reloaded.gate_receipts.get("test-feature").unwrap();
    assert_eq!(stored.decision, GateDecision::Blocked);
    assert_eq!(stored.missing_artifacts, vec!["tasks.md"]);
}

#[test]
fn test_parse_tool_and_path_from_json_payload() {
    use crate::commands::gate::parse_tool_call_json;

    // Standard Claude PreToolUse payload
    let json1 = r#"{"tool_name":"Write","tool_input":{"path":"src/main.rs","content":"hello"}}"#;
    let (tool1, path1) = parse_tool_call_json(json1).expect("must parse json1");
    assert_eq!(tool1, "Write");
    assert_eq!(path1, "src/main.rs");

    // Edit payload with input.file_path
    let json2 = r#"{"tool":"Edit","input":{"file_path":"src/commands/gate.rs"}}"#;
    let (tool2, path2) = parse_tool_call_json(json2).expect("must parse json2");
    assert_eq!(tool2, "Edit");
    assert_eq!(path2, "src/commands/gate.rs");

    // Bash command (non-write tool)
    let json3 = r#"{"tool_name":"Bash","tool_input":{"command":"cargo test"}}"#;
    let (tool3, path3) = parse_tool_call_json(json3).expect("must parse json3");
    assert_eq!(tool3, "Bash");
    assert_eq!(path3, "");

    // Invalid JSON
    assert!(parse_tool_call_json("invalid json").is_none());
}

#[test]
fn test_is_target_write_operation() {
    use crate::commands::gate::is_target_write_operation;

    assert!(is_target_write_operation("Write", "src/main.rs"));
    assert!(is_target_write_operation("write", "src/commands/gate.rs"));
    assert!(is_target_write_operation("Edit", "src/state/state.rs"));
    assert!(is_target_write_operation("edit", "/repo/src/lib.rs"));

    // Non-write tool
    assert!(!is_target_write_operation("Bash", "src/main.rs"));
    assert!(!is_target_write_operation("Read", "src/main.rs"));

    // Non-src path
    assert!(!is_target_write_operation("Write", "README.md"));
    assert!(!is_target_write_operation("Write", "docs/plans/foo.md"));
    assert!(!is_target_write_operation("Edit", "tests/cli.rs"));
}

#[test]
fn test_gate_event_record_append_and_stats_aggregation() {
    use crate::commands::gate::{load_gate_stats, log_gate_event, GateEventRecord};
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_dir = dir.path();

    let rec1 = GateEventRecord {
        timestamp: "2026-09-09T12:00:00Z".to_string(),
        harness: "claude".to_string(),
        tool: "Write".to_string(),
        path: "src/main.rs".to_string(),
        workspace: "/repo".to_string(),
        branch: Some("feat/foo".to_string()),
        stage: Some(4),
        feature: Some("foo".to_string()),
        decision: GateDecision::WouldBlock,
        edge_case: None,
        reason: "missing proposal.md".to_string(),
    };

    let rec2 = GateEventRecord {
        timestamp: "2026-09-09T12:01:00Z".to_string(),
        harness: "claude".to_string(),
        tool: "Edit".to_string(),
        path: "src/foo.rs".to_string(),
        workspace: "/repo".to_string(),
        branch: Some("feat/foo".to_string()),
        stage: Some(4),
        feature: Some("foo".to_string()),
        decision: GateDecision::Pass,
        edge_case: None,
        reason: "complete".to_string(),
    };

    let rec3 = GateEventRecord {
        timestamp: "2026-09-09T12:02:00Z".to_string(),
        harness: "claude".to_string(),
        tool: "Write".to_string(),
        path: "src/bar.rs".to_string(),
        workspace: "/repo".to_string(),
        branch: None,
        stage: None,
        feature: None,
        decision: GateDecision::Undetermined,
        edge_case: None,
        reason: "no checkpoint".to_string(),
    };

    let rec4 = GateEventRecord {
        timestamp: "2026-09-09T12:03:00Z".to_string(),
        harness: "claude".to_string(),
        tool: "Write".to_string(),
        path: "src/baz.rs".to_string(),
        workspace: "/repo".to_string(),
        branch: Some("feat/edge".to_string()),
        stage: Some(4),
        feature: Some("edge".to_string()),
        decision: GateDecision::EdgeCase,
        edge_case: Some(GateEdgeCase::MtimeFallback),
        reason: "mtime fallback".to_string(),
    };

    log_gate_event(config_dir, &rec1).unwrap();
    log_gate_event(config_dir, &rec2).unwrap();
    log_gate_event(config_dir, &rec3).unwrap();
    log_gate_event(config_dir, &rec4).unwrap();

    let stats = load_gate_stats(config_dir).unwrap();
    assert_eq!(stats.total_observed, 4);
    assert_eq!(stats.would_block, 1);
    assert_eq!(stats.pass, 1);
    assert_eq!(stats.undetermined, 1);
    assert_eq!(stats.mtime_fallback, 1);
    assert_eq!(stats.worktree_uncommitted, 0);
    assert_eq!(stats.stale_cycle_guard, 0);
}

#[test]
fn test_run_gate_check_workflow_matrix() {
    use crate::commands::gate::{run_gate_check, GateCheckArgs};
    use crate::commands::Context;
    use crate::state::state::{State, WorkflowSource, WorkflowStage};
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_dir = dir.path().join(".ce-ai");
    std::fs::create_dir_all(&config_dir).unwrap();
    let repo_root = dir.path().join("repo");
    std::fs::create_dir_all(&repo_root).unwrap();

    let ctx = Context {
        config_dir: config_dir.clone(),
        opencode_config_dir: config_dir.join("opencode"),
        workspace_root: Some(repo_root.clone()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    // 1. Kill-switch bypass
    let args_disabled = GateCheckArgs {
        tool: Some("Write".to_string()),
        path: Some("src/main.rs".to_string()),
        disabled: true,
        ..Default::default()
    };
    run_gate_check(&ctx, &args_disabled).unwrap();
    assert!(!crate::commands::gate::gate_events_log_path(&config_dir).exists());

    // 2. Non-target write bypass
    let args_non_target = GateCheckArgs {
        tool: Some("Write".to_string()),
        path: Some("README.md".to_string()),
        disabled: false,
        ..Default::default()
    };
    run_gate_check(&ctx, &args_non_target).unwrap();
    assert!(!crate::commands::gate::gate_events_log_path(&config_dir).exists());

    // 3. Stage 4 without OpenSpec artifacts -> WouldBlock (in observe mode)
    let state_path = config_dir.join("state.json");
    let mut state = State::default();
    for s in [
        WorkflowStage::OpenSpec,
        WorkflowStage::ExecutionPlan,
        WorkflowStage::WorkTdd,
    ] {
        state
            .validate_and_set_workflow_for_branch(
                &repo_root,
                None,
                s,
                "Progressing to stage 4",
                Some("feat-missing".to_string()),
                WorkflowSource::Manual,
            )
            .unwrap();
    }
    state.save(&state_path).unwrap();

    let args_target = GateCheckArgs {
        tool: Some("Write".to_string()),
        path: Some("src/lib.rs".to_string()),
        mode: Some("observe".to_string()),
        ..Default::default()
    };
    run_gate_check(&ctx, &args_target).unwrap();

    let stats1 = crate::commands::gate::load_gate_stats(&config_dir).unwrap();
    assert_eq!(stats1.total_observed, 1);
    assert_eq!(stats1.would_block, 1);

    // 4. Create OpenSpec artifacts for feat-missing -> Pass
    let change_dir = repo_root
        .join("openspec")
        .join("changes")
        .join("feat-missing");
    std::fs::create_dir_all(&change_dir).unwrap();
    std::fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();
    std::fs::write(change_dir.join("spec.md"), "# Spec").unwrap();
    std::fs::write(change_dir.join("tasks.md"), "# Tasks").unwrap();

    run_gate_check(&ctx, &args_target).unwrap();
    let stats2 = crate::commands::gate::load_gate_stats(&config_dir).unwrap();
    assert_eq!(stats2.total_observed, 2);
    assert_eq!(stats2.would_block, 1);
    assert_eq!(stats2.pass, 1);
}

#[test]
fn test_run_gate_check_stale_cycle_guard_uses_typed_flag_not_display_string() {
    use crate::commands::gate::{run_gate_check, GateCheckArgs};
    use crate::commands::Context;
    use crate::state::state::{State, WorkflowSource, WorkflowStage, WorkflowState};
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_dir = dir.path().join(".ce-ai");
    std::fs::create_dir_all(&config_dir).unwrap();
    let repo_root = dir.path().join("repo");
    std::fs::create_dir_all(&repo_root).unwrap();

    let ctx = Context {
        config_dir: config_dir.clone(),
        opencode_config_dir: config_dir.join("opencode"),
        workspace_root: Some(repo_root.clone()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let state_path = config_dir.join("state.json");
    let mut state = State::default();

    // Construct a WorkflowState where new_cycle is true, BUT task does NOT contain
    // "(nuevo ciclo detectado)" (e.g. customized or localized wording).
    let wf = WorkflowState {
        stage: WorkflowStage::WorkTdd,
        task: "Building new cycle with modified phrasing".to_string(),
        feature_name: Some("cycle-2-feat".to_string()),
        updated_at: chrono::Utc::now().to_rfc3339(),
        source: WorkflowSource::Inferred,
        resolution: None,
        new_cycle: true,
        execution_mode: None,
    };
    let key = State::workspace_branch_key(&repo_root, None);
    state.workflows.insert(key.clone(), wf.clone());
    state.workflow = Some(wf);
    state.save(&state_path).unwrap();

    let args_target = GateCheckArgs {
        tool: Some("Write".to_string()),
        path: Some("src/lib.rs".to_string()),
        disabled: false,
        ..Default::default()
    };
    run_gate_check(&ctx, &args_target).unwrap();

    let stats = crate::commands::gate::load_gate_stats(&config_dir).unwrap();
    assert_eq!(stats.total_observed, 1);
    assert_eq!(
        stats.stale_cycle_guard, 1,
        "must detect stale_cycle_guard via typed new_cycle field even when task text is different"
    );
    assert_eq!(stats.would_block, 0);

    // Second run with completely different task wording (e.g. English phrase) and new_cycle: true
    let wf2 = WorkflowState {
        stage: WorkflowStage::WorkTdd,
        task: "Implementing task after cycle reset without keywords".to_string(),
        feature_name: Some("cycle-2-feat".to_string()),
        updated_at: chrono::Utc::now().to_rfc3339(),
        source: WorkflowSource::Inferred,
        resolution: None,
        new_cycle: true,
        execution_mode: None,
    };
    state.workflows.insert(key, wf2.clone());
    state.workflow = Some(wf2);
    state.save(&state_path).unwrap();

    run_gate_check(&ctx, &args_target).unwrap();

    let stats2 = crate::commands::gate::load_gate_stats(&config_dir).unwrap();
    assert_eq!(stats2.total_observed, 2);
    assert_eq!(
        stats2.stale_cycle_guard, 2,
        "must continue detecting stale_cycle_guard on subsequent writes with arbitrary task wording"
    );
    assert_eq!(stats2.would_block, 0);
}

#[test]
fn test_run_gate_check_blocking_enforcement_and_receipt_creation() {
    use crate::commands::gate::{run_gate_check, GateCheckArgs, GateDecision};
    use crate::commands::Context;
    use crate::error::CeError;
    use crate::state::state::{State, WorkflowSource, WorkflowStage, WorkflowState};
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_dir = dir.path().join(".ce-ai");
    std::fs::create_dir_all(&config_dir).unwrap();
    let repo_root = dir.path().join("repo");
    std::fs::create_dir_all(&repo_root).unwrap();

    let ctx = Context {
        config_dir: config_dir.clone(),
        opencode_config_dir: config_dir.join("opencode"),
        workspace_root: Some(repo_root.clone()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let state_path = config_dir.join("state.json");
    let mut state = State::default();

    // Stage 4 active with feature 'missing-contract', but no OpenSpec files created
    let wf = WorkflowState {
        stage: WorkflowStage::WorkTdd,
        task: "Implementing core logic without specs".to_string(),
        feature_name: Some("missing-contract".to_string()),
        updated_at: chrono::Utc::now().to_rfc3339(),
        source: WorkflowSource::Manual,
        resolution: None,
        new_cycle: false,
        execution_mode: None,
    };
    let key = State::workspace_branch_key(&repo_root, None);
    state.workflows.insert(key, wf.clone());
    state.workflow = Some(wf);
    state.save(&state_path).unwrap();

    let args = GateCheckArgs {
        tool: Some("Write".to_string()),
        path: Some("src/commands/new_feat.rs".to_string()),
        mode: Some("enforce".to_string()),
        entry_point: None,
        execution_mode: None,
        disabled: false,
    };

    // Must return CeError::Usage which maps to Exit Code 2
    let res = run_gate_check(&ctx, &args);
    assert!(res.is_err());
    match res.unwrap_err() {
        CeError::Usage(msg) => {
            assert_eq!(CeError::Usage(msg.clone()).exit_code(), 2);
            assert!(msg.contains("write blocked on 'src/commands/new_feat.rs'"));
            assert!(msg.contains("missing required OpenSpec contract artifacts"));
        }
        other => panic!("expected CeError::Usage (exit code 2), got {other:?}"),
    }

    // Telemetry logged as blocked
    let stats = crate::commands::gate::load_gate_stats(&config_dir).unwrap();
    assert_eq!(stats.total_observed, 1);
    assert_eq!(stats.blocked, 1);
    assert_eq!(stats.would_block, 0);

    // Gate receipt recorded in state.json
    let state_reloaded = State::load(&state_path).unwrap();
    assert_eq!(state_reloaded.gate_receipts.len(), 1);
    let receipt = state_reloaded
        .gate_receipts
        .get("missing-contract")
        .expect("receipt must exist");
    assert_eq!(receipt.decision, GateDecision::Blocked);
    assert_eq!(receipt.target_path, "src/commands/new_feat.rs");
    assert!(receipt
        .missing_artifacts
        .contains(&"proposal.md".to_string()));
    assert!(receipt.missing_artifacts.contains(&"spec.md".to_string()));
    assert!(receipt.missing_artifacts.contains(&"tasks.md".to_string()));
}

#[test]
fn test_run_gate_check_observe_mode_does_not_block() {
    use crate::commands::gate::{run_gate_check, GateCheckArgs, GateDecision};
    use crate::commands::Context;
    use crate::state::state::{State, WorkflowSource, WorkflowStage, WorkflowState};
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_dir = dir.path().join(".ce-ai");
    std::fs::create_dir_all(&config_dir).unwrap();
    let repo_root = dir.path().join("repo");
    std::fs::create_dir_all(&repo_root).unwrap();

    let ctx = Context {
        config_dir: config_dir.clone(),
        opencode_config_dir: config_dir.join("opencode"),
        workspace_root: Some(repo_root.clone()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let state_path = config_dir.join("state.json");
    let mut state = State::default();

    let wf = WorkflowState {
        stage: WorkflowStage::WorkTdd,
        task: "Writing code in observe mode".to_string(),
        feature_name: Some("observe-feature".to_string()),
        updated_at: chrono::Utc::now().to_rfc3339(),
        source: WorkflowSource::Manual,
        resolution: None,
        new_cycle: false,
        execution_mode: None,
    };
    let key = State::workspace_branch_key(&repo_root, None);
    state.workflows.insert(key, wf.clone());
    state.workflow = Some(wf);
    state.save(&state_path).unwrap();

    let args = GateCheckArgs {
        tool: Some("Write".to_string()),
        path: Some("src/observe.rs".to_string()),
        mode: Some("observe".to_string()),
        entry_point: None,
        execution_mode: None,
        disabled: false,
    };

    // In observe mode, returns Ok(()) without blocking
    let res = run_gate_check(&ctx, &args);
    assert!(res.is_ok());

    let stats = crate::commands::gate::load_gate_stats(&config_dir).unwrap();
    assert_eq!(stats.total_observed, 1);
    assert_eq!(stats.blocked, 0);
    assert_eq!(stats.would_block, 1);

    let state_reloaded = State::load(&state_path).unwrap();
    let receipt = state_reloaded
        .gate_receipts
        .get("observe-feature")
        .expect("receipt must exist");
    assert_eq!(receipt.decision, GateDecision::WouldBlock);
}

#[test]
fn test_run_gate_check_ce_debug_and_tier_minimal_exemptions_pass() {
    use crate::commands::gate::{run_gate_check, GateCheckArgs, GateDecision};
    use crate::commands::Context;
    use crate::state::state::{
        AdoptionTier, ProjectAdoptionEntry, State, WorkflowSource, WorkflowStage, WorkflowState,
    };
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_dir = dir.path().join(".ce-ai");
    std::fs::create_dir_all(&config_dir).unwrap();
    let repo_root = dir.path().join("repo");
    std::fs::create_dir_all(&repo_root).unwrap();

    let ctx = Context {
        config_dir: config_dir.clone(),
        opencode_config_dir: config_dir.join("opencode"),
        workspace_root: Some(repo_root.clone()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let state_path = config_dir.join("state.json");
    let mut state = State::default();

    // 1. ce-debug task text permits write in Stage 4 without OpenSpec contract
    let wf_debug = WorkflowState {
        stage: WorkflowStage::WorkTdd,
        task: "ce-debug: fix null pointer in auth handler".to_string(),
        feature_name: Some("debug-fix".to_string()),
        updated_at: chrono::Utc::now().to_rfc3339(),
        source: WorkflowSource::Manual,
        resolution: None,
        new_cycle: false,
        execution_mode: None,
    };
    let key = State::workspace_branch_key(&repo_root, None);
    state.workflows.insert(key.clone(), wf_debug.clone());
    state.workflow = Some(wf_debug);
    state.save(&state_path).unwrap();

    let args_debug = GateCheckArgs {
        tool: Some("Edit".to_string()),
        path: Some("src/auth.rs".to_string()),
        mode: Some("enforce".to_string()),
        entry_point: None,
        execution_mode: None,
        disabled: false,
    };
    assert!(run_gate_check(&ctx, &args_debug).is_ok());

    let stats1 = crate::commands::gate::load_gate_stats(&config_dir).unwrap();
    assert_eq!(stats1.pass, 1);
    assert_eq!(stats1.blocked, 0);

    // 2. Project adopted with AdoptionTier::Minimal permits write in Stage 4 without OpenSpec
    state.projects.push(ProjectAdoptionEntry {
        path: repo_root.clone(),
        file: "AGENTS.md".to_string(),
        tier: AdoptionTier::Minimal,
        block_version: 1,
        block_sha256: "dummy".to_string(),
        created_file: false,
        adopted_at: "2026-09-01T00:00:00Z".to_string(),
    });
    let wf_minimal = WorkflowState {
        stage: WorkflowStage::WorkTdd,
        task: "Ordinary work in minimal project".to_string(),
        feature_name: Some("minimal-work".to_string()),
        updated_at: chrono::Utc::now().to_rfc3339(),
        source: WorkflowSource::Manual,
        resolution: None,
        new_cycle: false,
        execution_mode: None,
    };
    state.workflows.insert(key, wf_minimal.clone());
    state.workflow = Some(wf_minimal);
    state.save(&state_path).unwrap();

    let args_minimal = GateCheckArgs {
        tool: Some("Write".to_string()),
        path: Some("src/minimal.rs".to_string()),
        mode: Some("enforce".to_string()),
        entry_point: None,
        execution_mode: None,
        disabled: false,
    };
    assert!(run_gate_check(&ctx, &args_minimal).is_ok());

    let stats2 = crate::commands::gate::load_gate_stats(&config_dir).unwrap();
    assert_eq!(stats2.pass, 2);
    assert_eq!(stats2.blocked, 0);

    // Verify receipt in state
    let state_reloaded = State::load(&state_path).unwrap();
    assert_eq!(
        state_reloaded
            .gate_receipts
            .get("minimal-work")
            .unwrap()
            .decision,
        GateDecision::Pass
    );
}

#[test]
fn test_run_gate_check_organic_mode_permits_write_and_advisory_on_diff_limit() {
    use crate::commands::gate::{run_gate_check, GateCheckArgs, GateDecision};
    use crate::commands::Context;
    use crate::state::state::{ExecutionMode, State, WorkflowSource, WorkflowStage, WorkflowState};
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_dir = dir.path().join(".ce-ai");
    std::fs::create_dir_all(&config_dir).unwrap();
    let repo_root = dir.path().join("repo");
    std::fs::create_dir_all(&repo_root).unwrap();

    let ctx = Context {
        config_dir: config_dir.clone(),
        opencode_config_dir: config_dir.join("opencode"),
        workspace_root: Some(repo_root.clone()),
        dry_run: false,
        verbose: false,
        quiet: false,
    };

    let state_path = config_dir.join("state.json");
    let mut state = State::default();

    // Workflow state explicitly set to Organic mode without any OpenSpec files
    let wf_organic = WorkflowState {
        stage: WorkflowStage::WorkTdd,
        task: "Fixing quick bug organically".to_string(),
        feature_name: Some("quick-fix".to_string()),
        updated_at: chrono::Utc::now().to_rfc3339(),
        source: WorkflowSource::Manual,
        resolution: None,
        new_cycle: false,
        execution_mode: Some(ExecutionMode::Organic),
    };
    let key = State::workspace_branch_key(&repo_root, None);
    state.workflows.insert(key, wf_organic.clone());
    state.workflow = Some(wf_organic);
    state.save(&state_path).unwrap();

    // 1. Tool write in Organic mode passes with exit code 0 even without OpenSpec contract
    let args = GateCheckArgs {
        tool: Some("Write".to_string()),
        path: Some("src/quick_fix.rs".to_string()),
        mode: Some("enforce".to_string()),
        entry_point: None,
        execution_mode: Some("organic".to_string()),
        disabled: false,
    };

    let res = run_gate_check(&ctx, &args);
    assert!(res.is_ok(), "gate check must pass in organic mode");

    let stats = crate::commands::gate::load_gate_stats(&config_dir).unwrap();
    assert_eq!(stats.pass, 1);
    assert_eq!(stats.blocked, 0);

    let state_reloaded = State::load(&state_path).unwrap();
    let receipt = state_reloaded
        .gate_receipts
        .get("quick-fix")
        .expect("receipt must exist for quick-fix");
    assert_eq!(receipt.decision, GateDecision::Pass);
    assert!(receipt
        .reason
        .contains("organic execution mode permits writes"));
}
