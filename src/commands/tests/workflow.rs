use super::*;
use crate::commands::Context;
use tempfile::TempDir;

fn ctx() -> (TempDir, Context) {
    let tmp = TempDir::new().unwrap();
    let ctx = Context::resolve(Some(tmp.path().join("ce-ai")), false, false, true).unwrap();
    (tmp, ctx)
}

#[test]
fn status_lists_stages_and_defaults_without_checkpoint() {
    let (_tmp, ctx) = ctx();
    let joined = status_lines(&ctx).unwrap().join("\n");
    assert!(joined.contains("[1: Ideation]"));
    assert!(joined.contains("[7: Ship]"));
}

#[test]
fn checkpoint_validates_transitions_and_saves_workflow_state() {
    let (_tmp, ctx) = ctx();

    // Stage 1 -> Stage 2: Valid
    let res = checkpoint_lines(
        &ctx,
        WorkflowStage::OpenSpec,
        "Authoring spec",
        Some("my-feat"),
    );
    assert!(res.is_ok());

    // Stage 2 -> Stage 5: Invalid jump (2 -> 5)
    let res_err = checkpoint_lines(&ctx, WorkflowStage::Verification, "Testing", None);
    assert!(res_err.is_err());
    assert!(res_err
        .unwrap_err()
        .to_string()
        .contains("invalid workflow transition"));

    // Stage 2 -> Stage 3: Valid advance
    let res3 = checkpoint_lines(&ctx, WorkflowStage::ExecutionPlan, "Planning tasks", None);
    assert!(res3.is_ok());
}

#[test]
fn probe_openspec_context_detects_features_and_counts_tasks() {
    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path();

    // No openspec dir -> None
    assert_eq!(probe_openspec_context_in(repo_root, &None), None);

    // Create openspec/changes/my-feature/
    let feat_dir = repo_root
        .join("openspec")
        .join("changes")
        .join("my-feature");
    std::fs::create_dir_all(&feat_dir).unwrap();
    std::fs::write(feat_dir.join("proposal.md"), "# Proposal").unwrap();
    std::fs::write(feat_dir.join("spec.md"), "# Spec").unwrap();
    std::fs::write(
        feat_dir.join("tasks.md"),
        "- [x] Task 1\n- [ ] Task 2\n- [X] Task 3\n- [ ] Task 4\n",
    )
    .unwrap();

    let wf = Some(WorkflowState {
        stage: WorkflowStage::WorkTdd,
        task: "Building feature".to_string(),
        feature_name: Some("my-feature".to_string()),
        updated_at: "2026-09-02T00:00:00Z".to_string(),
    });

    let info = probe_openspec_context_in(repo_root, &wf).expect("must detect feature");
    assert_eq!(info.feature, "my-feature");
    assert!(info.has_proposal);
    assert!(info.has_spec);
    assert!(info.has_tasks);
    assert_eq!(info.completed_tasks, 2);
    assert_eq!(info.total_tasks, 4);

    // Test fallback when feature_name is None
    let fallback_info =
        probe_openspec_context_in(repo_root, &None).expect("must fallback to directory");
    assert_eq!(fallback_info.feature, "my-feature");
}

#[test]
fn repo_state_serialization_and_resume_lines() {
    let (_tmp, ctx) = ctx();

    let lines = resume_lines(&ctx).unwrap().join("\n");
    assert!(lines.contains("== [Environment State & Drift Status] =="));
    assert!(lines.contains("git branch:"));
    assert!(lines.contains("working tree:"));
    assert!(lines.contains("manifest integrity:"));

    let repo_state = probe_repo_state(&ctx, &None);
    let serialized = serde_json::to_string(&repo_state).unwrap();
    let deserialized: RepoState = serde_json::from_str(&serialized).unwrap();
    assert_eq!(repo_state, deserialized);
}

#[test]
fn pre_invocation_deduplicates_by_conversation_id() {
    let tmp = TempDir::new().unwrap();
    let marker_dir = tmp.path();

    let input1 = r#"{"conversationId": "session-abc-123", "invocationNum": 0}"#;
    assert!(should_inject_pre_invocation(input1, marker_dir));

    // Second turn in same session must be deduplicated
    let input2 = r#"{"conversationId": "session-abc-123", "invocationNum": 1}"#;
    assert!(!should_inject_pre_invocation(input2, marker_dir));

    // Different session ID must inject
    let input3 = r#"{"conversationId": "session-xyz-789", "invocationNum": 0}"#;
    assert!(should_inject_pre_invocation(input3, marker_dir));
}

#[test]
fn pre_invocation_supports_session_id_fallback_and_invocation_num() {
    let tmp = TempDir::new().unwrap();
    let marker_dir = tmp.path();

    // Uses sessionId if conversationId not present
    let input1 = r#"{"sessionId": "sess-456", "invocationNum": 0}"#;
    assert!(should_inject_pre_invocation(input1, marker_dir));
    assert!(!should_inject_pre_invocation(input1, marker_dir));

    // When no ID is present, relies on invocationNum
    assert!(should_inject_pre_invocation(
        r#"{"invocationNum": 0}"#,
        marker_dir
    ));
    assert!(!should_inject_pre_invocation(
        r#"{"invocationNum": 1}"#,
        marker_dir
    ));
}
