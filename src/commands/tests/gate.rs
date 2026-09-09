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
    };
    run_gate_check(&ctx, &args_disabled).unwrap();
    assert!(!crate::commands::gate::gate_events_log_path(&config_dir).exists());

    // 2. Non-target write bypass
    let args_non_target = GateCheckArgs {
        tool: Some("Write".to_string()),
        path: Some("README.md".to_string()),
        disabled: false,
    };
    run_gate_check(&ctx, &args_non_target).unwrap();
    assert!(!crate::commands::gate::gate_events_log_path(&config_dir).exists());

    // 3. Stage 4 without OpenSpec artifacts -> WouldBlock
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
        disabled: false,
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
    };
    let key = State::workspace_branch_key(&repo_root, None);
    state.workflows.insert(key.clone(), wf.clone());
    state.workflow = Some(wf);
    state.save(&state_path).unwrap();

    let args_target = GateCheckArgs {
        tool: Some("Write".to_string()),
        path: Some("src/lib.rs".to_string()),
        disabled: false,
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
