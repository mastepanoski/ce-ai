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
        source: WorkflowSource::Manual,
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

#[test]
fn test_sanitize_feature_name_strips_traversal_and_prefixes() {
    assert_eq!(
        sanitize_feature_name("feature/my-cool-feature"),
        "my-cool-feature"
    );
    assert_eq!(sanitize_feature_name("feat/issue-296"), "issue-296");
    assert_eq!(sanitize_feature_name("fix/bug-fix"), "bug-fix");
    assert_eq!(
        sanitize_feature_name("refs/heads/feature/nested/name"),
        "nested-name"
    );
    // Directory traversal sequences must be neutralized
    assert_eq!(sanitize_feature_name("../../etc/passwd"), "etc-passwd");
    assert_eq!(
        sanitize_feature_name("..\\..\\windows\\system32"),
        "windows-system32"
    );
    // Empty / whitespace / non-alphanumeric fallback
    assert_eq!(sanitize_feature_name("///...///"), "default");
}

#[test]
fn test_is_transitory_git_state_detects_rebase_and_merge() {
    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path();
    let git_dir = repo_root.join(".git");
    std::fs::create_dir_all(&git_dir).unwrap();

    // Clean state
    assert!(!is_transitory_git_state(repo_root));

    // Rebase merge
    let rebase_merge = git_dir.join("rebase-merge");
    std::fs::create_dir_all(&rebase_merge).unwrap();
    assert!(is_transitory_git_state(repo_root));
    std::fs::remove_dir_all(&rebase_merge).unwrap();

    // Rebase apply
    let rebase_apply = git_dir.join("rebase-apply");
    std::fs::create_dir_all(&rebase_apply).unwrap();
    assert!(is_transitory_git_state(repo_root));
    std::fs::remove_dir_all(&rebase_apply).unwrap();

    // CHERRY_PICK_HEAD
    let cherry_pick = git_dir.join("CHERRY_PICK_HEAD");
    std::fs::write(&cherry_pick, "deadbeef").unwrap();
    assert!(is_transitory_git_state(repo_root));
    std::fs::remove_file(&cherry_pick).unwrap();

    // MERGE_HEAD
    let merge_head = git_dir.join("MERGE_HEAD");
    std::fs::write(&merge_head, "deadbeef").unwrap();
    assert!(is_transitory_git_state(repo_root));
}

#[test]
fn test_stage_inference_stages_1_to_5() {
    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path();

    // 1. Stage 1: Ideation (docs/brainstorms exists, no openspec)
    let brainstorms = repo_root.join("docs").join("brainstorms");
    std::fs::create_dir_all(&brainstorms).unwrap();
    std::fs::write(brainstorms.join("idea.md"), "# Big Idea").unwrap();

    let (stage1, _, feat1) =
        infer_stage_from_repo(repo_root, Some("main")).expect("must infer stage 1");
    assert_eq!(stage1, WorkflowStage::Ideation);
    assert_eq!(feat1, None);

    // 2. Stage 2: OpenSpec (proposal.md + spec.md, no tasks.md)
    let change_dir = repo_root.join("openspec").join("changes").join("test-feat");
    std::fs::create_dir_all(&change_dir).unwrap();
    std::fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();
    std::fs::write(change_dir.join("spec.md"), "# Spec").unwrap();

    let (stage2, _, feat2) =
        infer_stage_from_repo(repo_root, Some("feat/test-feat")).expect("must infer stage 2");
    assert_eq!(stage2, WorkflowStage::OpenSpec);
    assert_eq!(feat2.as_deref(), Some("test-feat"));

    // 3. Stage 3: Execution Plan (tasks.md with 0 completed)
    let tasks_file = change_dir.join("tasks.md");
    std::fs::write(&tasks_file, "- [ ] Task 1\n- [ ] Task 2\n").unwrap();

    let (stage3, _, feat3) =
        infer_stage_from_repo(repo_root, Some("feat/test-feat")).expect("must infer stage 3");
    assert_eq!(stage3, WorkflowStage::ExecutionPlan);
    assert_eq!(feat3.as_deref(), Some("test-feat"));

    // 4. Stage 4: Work/TDD (1/2 tasks completed)
    std::fs::write(&tasks_file, "- [x] Task 1\n- [ ] Task 2\n").unwrap();

    let (stage4, _, feat4) =
        infer_stage_from_repo(repo_root, Some("feat/test-feat")).expect("must infer stage 4");
    assert_eq!(stage4, WorkflowStage::WorkTdd);
    assert_eq!(feat4.as_deref(), Some("test-feat"));

    // 5. Stage 5: Verification (all tasks completed)
    std::fs::write(&tasks_file, "- [x] Task 1\n- [x] Task 2\n").unwrap();

    let (stage5, _, feat5) =
        infer_stage_from_repo(repo_root, Some("feat/test-feat")).expect("must infer stage 5");
    assert_eq!(stage5, WorkflowStage::Verification);
    assert_eq!(feat5.as_deref(), Some("test-feat"));
}

#[test]
fn test_maybe_auto_checkpoint_respects_opt_out_and_monotonic_guard() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    let state_path = config_dir.join("state.json");

    let repo_root = tmp.path().join("repo");
    let change_dir = repo_root.join("openspec").join("changes").join("feat-auto");
    std::fs::create_dir_all(&change_dir).unwrap();
    std::fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();
    std::fs::write(change_dir.join("spec.md"), "# Spec").unwrap();

    let ctx = Context::resolve(Some(config_dir), false, false, true).unwrap();

    let adoption_entry = crate::state::state::ProjectAdoptionEntry {
        path: repo_root.clone(),
        file: "AGENTS.md".into(),
        tier: crate::state::state::AdoptionTier::Full,
        block_version: 4,
        block_sha256: "fake-sha".into(),
        created_file: true,
        adopted_at: "2026-09-05T00:00:00Z".into(),
    };

    // 0. Non-adopted project skips auto-checkpoint
    let unadopted_state = State::new();
    unadopted_state.save(&state_path).unwrap();
    let unadopted_res = maybe_auto_checkpoint(&ctx, &repo_root, &state_path).unwrap();
    assert!(
        unadopted_res.is_none(),
        "non-adopted project must skip auto-checkpoint"
    );

    // 1. When auto_checkpoint is disabled in state.json, auto-checkpoint does not run
    let mut state = State::new();
    state.projects.push(adoption_entry.clone());
    state.auto_checkpoint = Some(false);
    state.save(&state_path).unwrap();

    let result = maybe_auto_checkpoint(&ctx, &repo_root, &state_path).unwrap();
    assert!(result.is_none());

    // 2. When auto_checkpoint is enabled (default), advancing from Ideation -> OpenSpec works
    let mut state2 = State::new();
    state2.projects.push(adoption_entry);
    state2.save(&state_path).unwrap();

    let res2 = maybe_auto_checkpoint(&ctx, &repo_root, &state_path).unwrap();
    assert!(res2.is_some());
    let saved_wf = res2.unwrap();
    assert_eq!(saved_wf.stage, WorkflowStage::OpenSpec);
    assert_eq!(saved_wf.source, WorkflowSource::Inferred);

    // 3. Monotonic provenance: If a manual checkpoint is saved at Stage 4, auto-checkpoint cannot regress to Stage 2
    let mut state3 = State::load(&state_path).unwrap();
    // Advance legally to Stage 4 manually
    state3
        .validate_and_set_workflow_for_branch(
            &repo_root,
            None,
            WorkflowStage::ExecutionPlan,
            "Manual plan",
            Some("feat-auto".into()),
            WorkflowSource::Manual,
        )
        .unwrap();
    state3
        .validate_and_set_workflow_for_branch(
            &repo_root,
            None,
            WorkflowStage::WorkTdd,
            "Manual work",
            Some("feat-auto".into()),
            WorkflowSource::Manual,
        )
        .unwrap();
    state3.save(&state_path).unwrap();

    // Even though repo files are at Stage 2 (no tasks.md), inferred stage cannot regress manual stage 4
    let res3 = maybe_auto_checkpoint(&ctx, &repo_root, &state_path).unwrap();
    assert!(res3.is_none());

    let state_after = State::load(&state_path).unwrap();
    let current = state_after
        .current_workflow_for_branch(&repo_root, None)
        .unwrap();
    assert_eq!(current.stage, WorkflowStage::WorkTdd);
    assert_eq!(current.source, WorkflowSource::Manual);
}

#[test]
fn test_extract_paths_from_task_text() {
    let text1 = "1. Author canonical skills/sequential-thinking/SKILL.md in repository root";
    let paths1 = extract_paths_from_task_text(text1);
    assert_eq!(paths1, vec!["skills/sequential-thinking/SKILL.md"]);

    let text2 = "2. Create `src/source/builtin_skills.rs` and update `tests/cli.rs`";
    let paths2 = extract_paths_from_task_text(text2);
    assert!(paths2.contains(&"src/source/builtin_skills.rs".to_string()));
    assert!(paths2.contains(&"tests/cli.rs".to_string()));

    let text3 = "Wire fallback seeding into `src/commands/install.rs` and `src/commands/sync.rs`";
    let paths3 = extract_paths_from_task_text(text3);
    assert!(paths3.contains(&"src/commands/install.rs".to_string()));
    assert!(paths3.contains(&"src/commands/sync.rs".to_string()));

    let text4 = "Setup database migration and configure redis cache";
    let paths4 = extract_paths_from_task_text(text4);
    assert!(paths4.is_empty());
}

#[test]
fn test_reconcile_tasks_with_git_exact_and_prefix_matches() {
    let tmp = TempDir::new().unwrap();
    let tasks_path = tmp.path().join("tasks.md");
    let content = "\
# Tasks
- [ ] 1. Implement feature in `src/commands/foo.rs`
- [ ] 2. Update `tests/cli.rs`
- [x] 3. Documentation in `docs/`
";
    std::fs::write(&tasks_path, content).unwrap();

    let touched = vec!["src/commands/foo.rs".to_string(), "Cargo.toml".to_string()];

    let report = reconcile_tasks_with_git(tmp.path(), "my-feat", &tasks_path, &touched).unwrap();
    assert!(report.has_desync());
    assert_eq!(report.completed_tasks, 1);
    assert_eq!(report.total_tasks, 3);
    assert_eq!(report.desynced_tasks.len(), 1);
    assert_eq!(report.desynced_tasks[0].task_index, 1);
    assert_eq!(
        report.desynced_tasks[0].matched_files,
        vec!["src/commands/foo.rs"]
    );

    let warn = report.warning_line();
    assert!(warn.contains("Tasks desync detected"));
    assert!(warn.contains("1 unchecked task(s) reference modified files"));
    assert!(warn.contains("1/3 completed"));
}

#[test]
fn test_reconcile_tasks_with_git_aggregate_fallback() {
    let tmp = TempDir::new().unwrap();
    let tasks_path = tmp.path().join("tasks.md");
    let content = "\
# Tasks
- [ ] 1. Do preliminary setup
- [ ] 2. Perform backend work
";
    std::fs::write(&tasks_path, content).unwrap();

    let touched = vec!["src/main.rs".to_string()];

    let report = reconcile_tasks_with_git(tmp.path(), "my-feat", &tasks_path, &touched).unwrap();
    assert!(report.has_desync());
    assert!(report.is_aggregate_desync);
    assert_eq!(report.completed_tasks, 0);
    assert_eq!(report.total_tasks, 2);

    let warn = report.warning_line();
    assert!(warn.contains("Tasks desync detected"));
    assert!(warn.contains("0/2 completed"));
}

#[test]
fn test_reconcile_tasks_with_git_all_completed_no_desync() {
    let tmp = TempDir::new().unwrap();
    let tasks_path = tmp.path().join("tasks.md");
    let content = "\
# Tasks
- [x] 1. Implement feature in `src/commands/foo.rs`
- [X] 2. Update `tests/cli.rs`
";
    std::fs::write(&tasks_path, content).unwrap();

    let touched = vec!["src/commands/foo.rs".to_string()];
    let report = reconcile_tasks_with_git(tmp.path(), "my-feat", &tasks_path, &touched);
    assert!(report.is_none());
}

#[test]
fn test_reconcile_tasks_with_git_graceful_degradation_r7() {
    let tmp = TempDir::new().unwrap();
    let non_existent = tmp.path().join("does_not_exist.md");
    let touched = vec!["src/commands/foo.rs".to_string()];

    // Non-existent tasks.md -> gracefully returns None
    assert!(reconcile_tasks_with_git(tmp.path(), "my-feat", &non_existent, &touched).is_none());

    // Empty touched files (e.g. git failure / non-git workspace) -> gracefully returns None
    let real_tasks = tmp.path().join("tasks.md");
    std::fs::write(&real_tasks, "- [ ] 1. Task in `src/foo.rs`").unwrap();
    assert!(reconcile_tasks_with_git(tmp.path(), "my-feat", &real_tasks, &[]).is_none());
}
