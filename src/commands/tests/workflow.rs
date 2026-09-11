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
        resolution: None,
        new_cycle: false,
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

    let (stage1, _, feat1, _) =
        infer_stage_from_repo(repo_root, Some("main")).expect("must infer stage 1");
    assert_eq!(stage1, WorkflowStage::Ideation);
    assert_eq!(feat1, None);

    // 2. Stage 2: OpenSpec (proposal.md + spec.md, no tasks.md)
    let change_dir = repo_root.join("openspec").join("changes").join("test-feat");
    std::fs::create_dir_all(&change_dir).unwrap();
    std::fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();
    std::fs::write(change_dir.join("spec.md"), "# Spec").unwrap();

    let (stage2, _, feat2, _) =
        infer_stage_from_repo(repo_root, Some("feat/test-feat")).expect("must infer stage 2");
    assert_eq!(stage2, WorkflowStage::OpenSpec);
    assert_eq!(feat2.as_deref(), Some("test-feat"));

    // 3. Stage 3: Execution Plan (tasks.md with 0 completed)
    let tasks_file = change_dir.join("tasks.md");
    std::fs::write(&tasks_file, "- [ ] Task 1\n- [ ] Task 2\n").unwrap();

    let (stage3, _, feat3, _) =
        infer_stage_from_repo(repo_root, Some("feat/test-feat")).expect("must infer stage 3");
    assert_eq!(stage3, WorkflowStage::ExecutionPlan);
    assert_eq!(feat3.as_deref(), Some("test-feat"));

    // 4. Stage 4: Work/TDD (1/2 tasks completed)
    std::fs::write(&tasks_file, "- [x] Task 1\n- [ ] Task 2\n").unwrap();

    let (stage4, _, feat4, _) =
        infer_stage_from_repo(repo_root, Some("feat/test-feat")).expect("must infer stage 4");
    assert_eq!(stage4, WorkflowStage::WorkTdd);
    assert_eq!(feat4.as_deref(), Some("test-feat"));

    // 5. Stage 5: Verification (all tasks completed)
    std::fs::write(&tasks_file, "- [x] Task 1\n- [x] Task 2\n").unwrap();

    let (stage5, _, feat5, _) =
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

#[test]
fn test_count_task_checkboxes_parsing_and_toctou_degradation() {
    let tmp = TempDir::new().unwrap();
    let tasks_path = tmp.path().join("tasks.md");

    // Case 1: Valid content with mixed checkmarks, plain text, and whitespace
    let content = "\
# Tasks: Feature Foo
- [x] Task 1 completed
  - [x] Subtask 1.1 completed
- [X] Task 2 completed with capital X
- [ ] Task 3 pending
  - [ ] Subtask 3.1 pending
Plain notes without checkbox
- Bullet without box
";
    std::fs::write(&tasks_path, content).unwrap();
    let (completed, total) = count_task_checkboxes(&tasks_path);
    assert_eq!(completed, 3);
    assert_eq!(total, 5);

    // Case 2: Non-existent path -> gracefully returns (0, 0) without panic
    let non_existent = tmp.path().join("does_not_exist.md");
    let (completed, total) = count_task_checkboxes(&non_existent);
    assert_eq!(completed, 0);
    assert_eq!(total, 0);

    // Case 3: Directory passed instead of file -> gracefully returns (0, 0) without panic
    let dir_path = tmp.path().join("a_dir");
    std::fs::create_dir_all(&dir_path).unwrap();
    let (completed, total) = count_task_checkboxes(&dir_path);
    assert_eq!(completed, 0);
    assert_eq!(total, 0);
}

#[test]
fn test_probe_unarchived_completed_changes_multi_directory() {
    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path();
    let changes_dir = repo_root.join("openspec").join("changes");

    // 1. Fully completed change outside archive/ -> must be detected
    let complete_dir = changes_dir.join("feat-alpha-done");
    std::fs::create_dir_all(&complete_dir).unwrap();
    std::fs::write(
        complete_dir.join("tasks.md"),
        "- [x] Task 1\n- [X] Task 2\n",
    )
    .unwrap();

    // 2. Fully completed second change -> must be detected and sorted alphabetically
    let complete_dir_2 = changes_dir.join("feat-beta-done");
    std::fs::create_dir_all(&complete_dir_2).unwrap();
    std::fs::write(
        complete_dir_2.join("tasks.md"),
        "- [x] Unit 1\n- [x] Unit 2\n- [x] Unit 3\n",
    )
    .unwrap();

    // 3. Incomplete change (1/2) -> must NOT be detected
    let incomplete_dir = changes_dir.join("feat-gamma-wip");
    std::fs::create_dir_all(&incomplete_dir).unwrap();
    std::fs::write(
        incomplete_dir.join("tasks.md"),
        "- [x] Task 1\n- [ ] Task 2\n",
    )
    .unwrap();

    // 4. Change with 0 tasks -> must NOT be detected
    let empty_tasks_dir = changes_dir.join("feat-empty");
    std::fs::create_dir_all(&empty_tasks_dir).unwrap();
    std::fs::write(empty_tasks_dir.join("tasks.md"), "# Tasks without boxes\n").unwrap();

    // 5. Change with no tasks.md file -> must NOT be detected
    let no_tasks_dir = changes_dir.join("feat-no-tasks-file");
    std::fs::create_dir_all(&no_tasks_dir).unwrap();
    std::fs::write(no_tasks_dir.join("proposal.md"), "# Proposal\n").unwrap();

    // 6. Fully completed change inside archive/ -> must NOT be detected
    let archived_dir = changes_dir.join("archive").join("feat-archived-done");
    std::fs::create_dir_all(&archived_dir).unwrap();
    std::fs::write(
        archived_dir.join("tasks.md"),
        "- [x] Arch 1\n- [x] Arch 2\n",
    )
    .unwrap();

    // 7. Regular file directly inside openspec/changes -> must NOT cause panic or be detected
    std::fs::write(changes_dir.join("notes.txt"), "some notes\n").unwrap();

    let unarchived = probe_unarchived_completed_changes(repo_root);
    assert_eq!(unarchived.len(), 2);
    assert_eq!(unarchived[0].feature, "feat-alpha-done");
    assert_eq!(unarchived[0].completed_tasks, 2);
    assert_eq!(unarchived[0].total_tasks, 2);

    assert_eq!(unarchived[1].feature, "feat-beta-done");
    assert_eq!(unarchived[1].completed_tasks, 3);
    assert_eq!(unarchived[1].total_tasks, 3);
}

#[test]
fn test_probe_unarchived_completed_changes_non_existent_openspec_dir() {
    let tmp = TempDir::new().unwrap();
    let unarchived = probe_unarchived_completed_changes(tmp.path());
    assert!(unarchived.is_empty());
}

#[test]
fn test_stage_inference_resolution_branch_vs_mtime_fallback() {
    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path();

    let change_dir = repo_root
        .join("openspec")
        .join("changes")
        .join("feat-resolution");
    std::fs::create_dir_all(&change_dir).unwrap();
    std::fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();
    std::fs::write(change_dir.join("spec.md"), "# Spec").unwrap();

    // 1. Branch matching openspec directory resolves as FeatureResolution::Branch
    let (stage_branch, _, feat_branch, res_branch) =
        infer_stage_from_repo(repo_root, Some("feat/feat-resolution")).expect("must infer stage");
    assert_eq!(stage_branch, WorkflowStage::OpenSpec);
    assert_eq!(feat_branch.as_deref(), Some("feat-resolution"));
    assert_eq!(
        res_branch,
        Some(crate::state::state::FeatureResolution::Branch)
    );

    // 2. Branch not matching (or None / no git) falls back to mtime -> FeatureResolution::MtimeFallback
    let (stage_fallback, _, feat_fallback, res_fallback) =
        infer_stage_from_repo(repo_root, Some("main")).expect("must infer stage via fallback");
    assert_eq!(stage_fallback, WorkflowStage::OpenSpec);
    assert_eq!(feat_fallback.as_deref(), Some("feat-resolution"));
    assert_eq!(
        res_fallback,
        Some(crate::state::state::FeatureResolution::MtimeFallback)
    );
}

#[test]
fn test_probe_openspec_uncommitted_detection() {
    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path();

    // Initialize real git repo hermetically
    let mut git_init = std::process::Command::new("git");
    for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        git_init.env_remove(var);
    }
    git_init
        .args(["init"])
        .current_dir(repo_root)
        .output()
        .unwrap();

    let change_dir = repo_root.join("openspec").join("changes").join("feat-git");
    std::fs::create_dir_all(&change_dir).unwrap();
    std::fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();

    // 1. Untracked file -> detected as uncommitted
    assert!(probe_openspec_has_uncommitted(repo_root, "feat-git"));

    // 2. Committed file -> clean, not uncommitted
    let mut git_add = std::process::Command::new("git");
    for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        git_add.env_remove(var);
    }
    git_add
        .args(["add", "."])
        .current_dir(repo_root)
        .output()
        .unwrap();

    let mut git_commit = std::process::Command::new("git");
    for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        git_commit.env_remove(var);
    }
    git_commit
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "commit spec",
        ])
        .current_dir(repo_root)
        .output()
        .unwrap();

    assert!(!probe_openspec_has_uncommitted(repo_root, "feat-git"));

    // 3. New untracked tasks.md added -> detected as uncommitted
    std::fs::write(change_dir.join("tasks.md"), "- [ ] Task 1\n").unwrap();
    assert!(probe_openspec_has_uncommitted(repo_root, "feat-git"));
}

#[test]
fn test_maybe_auto_checkpoint_second_cycle_on_same_branch() {
    let tmp = TempDir::new().unwrap();
    let config_dir = tmp.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    let state_path = config_dir.join("state.json");

    let repo_root = tmp.path().join("repo");
    std::fs::create_dir_all(&repo_root).unwrap();

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

    let mut state = State::new();
    state.projects.push(adoption_entry);
    // Cycle 1 is at Stage 7 (GitShipping) on feature "feat-first"
    let init_wf = WorkflowState {
        stage: WorkflowStage::GitShipping,
        task: "Shipped feat-first".into(),
        feature_name: Some("feat-first".into()),
        updated_at: chrono::Utc::now().to_rfc3339(),
        source: WorkflowSource::Manual,
        resolution: Some(FeatureResolution::Branch),
        new_cycle: false,
    };
    let key = State::workspace_branch_key(&repo_root, None);
    state.workflows.insert(key, init_wf.clone());
    state.workflow = Some(init_wf);
    state.save(&state_path).unwrap();

    // Start Cycle 2: docs/brainstorms/idea.md exists, no openspec dir
    let brainstorms = repo_root.join("docs").join("brainstorms");
    std::fs::create_dir_all(&brainstorms).unwrap();
    std::fs::write(brainstorms.join("idea2.md"), "# Big Idea 2").unwrap();

    // Auto-checkpoint must succeed (not blocked by monotonic guard) because it's a new cycle (feature changed)
    let res = maybe_auto_checkpoint(&ctx, &repo_root, &state_path).unwrap();
    assert!(
        res.is_some(),
        "new cycle must not be blocked by monotonic guard"
    );
    let wf = res.unwrap();
    assert_eq!(wf.stage, WorkflowStage::Ideation);
    assert_eq!(wf.source, WorkflowSource::Inferred);
    assert!(
        wf.new_cycle,
        "new_cycle flag must be set to true on WorkflowState"
    );
    assert!(
        wf.task.to_lowercase().contains("nuevo ciclo detectado"),
        "task should indicate new cycle detected, got: {}",
        wf.task
    );
}

// --- Ship-readiness observe-only gate (Issue #354) ---

fn repo_state_for(
    commits_ahead: u32,
    stage6: bool,
    receipt: Option<ReviewReceipt>,
    head: Option<&str>,
) -> RepoState {
    RepoState {
        git_branch: Some("feat/x".into()),
        head_sha: Some("abc1234".into()),
        head_full_sha: head.map(String::from),
        is_git_clean: true,
        modified_files: vec![],
        manifest_drift_count: 0,
        adoption_status: Some(AdoptionBlockStatus::Ok),
        openspec_context: None,
        task_desync: None,
        unarchived_completed_changes: vec![],
        commits_ahead,
        stage6_artifact_present: stage6,
        review_receipt: receipt,
    }
}

#[test]
fn ship_readiness_evaluator_truth_table() {
    use ShipReadinessGap::*;
    let receipt = ReviewReceipt {
        head_sha: "abc".into(),
        recorded_at: "2026-09-11T00:00:00Z".into(),
        override_reason: None,
    };

    // Nothing ahead -> no gaps even when everything is missing.
    assert!(
        evaluate_ship_readiness(0, false, None, Some("abc"), WorkflowStage::WorkTdd).is_empty()
    );

    // Everything satisfied at Ship.
    assert!(evaluate_ship_readiness(
        2,
        true,
        Some(&receipt),
        Some("abc"),
        WorkflowStage::GitShipping
    )
    .is_empty());

    // Missing Stage 6 artifact.
    assert_eq!(
        evaluate_ship_readiness(
            1,
            false,
            Some(&receipt),
            Some("abc"),
            WorkflowStage::GitShipping
        ),
        vec![Stage6Missing]
    );

    // Missing receipt.
    assert_eq!(
        evaluate_ship_readiness(1, true, None, Some("abc"), WorkflowStage::GitShipping),
        vec![ReviewReceiptMissing]
    );

    // Stale receipt (different head).
    assert_eq!(
        evaluate_ship_readiness(
            1,
            true,
            Some(&receipt),
            Some("def"),
            WorkflowStage::GitShipping
        ),
        vec![ReviewReceiptStale]
    );

    // Early stage while commits are ahead, gap ordering preserved.
    assert_eq!(
        evaluate_ship_readiness(1, false, None, Some("abc"), WorkflowStage::WorkTdd),
        vec![Stage6Missing, ReviewReceiptMissing, EarlyStageWithCommits]
    );
}

#[test]
fn ship_readiness_lines_reports_signals_and_gap_descriptions() {
    let rs = repo_state_for(3, false, None, Some("abc"));
    let text = ship_readiness_lines(&rs, WorkflowStage::WorkTdd).join("\n");
    assert!(text.contains("== [Ship Readiness (observe-only, issue #354)] =="));
    assert!(text.contains("commits ahead of base: 3"));
    assert!(text.contains("stage 6 artifact (docs/solutions/): MISSING"));
    assert!(text.contains("code-review receipt: MISSING"));
    // The information block must not duplicate the gap warnings (status owns those).
    assert!(!text.contains("! Warning:"));

    // Not adopted -> no block.
    let mut not_adopted = repo_state_for(3, false, None, Some("abc"));
    not_adopted.adoption_status = None;
    assert!(ship_readiness_lines(&not_adopted, WorkflowStage::WorkTdd).is_empty());

    // Nothing ahead -> no block.
    let none_ahead = repo_state_for(0, false, None, Some("abc"));
    assert!(ship_readiness_lines(&none_ahead, WorkflowStage::WorkTdd).is_empty());

    // Gap descriptions are actionable.
    assert!(ShipReadinessGap::Stage6Missing
        .describe()
        .contains("docs/solutions"));
    assert!(ShipReadinessGap::ReviewReceiptMissing
        .describe()
        .contains("review-receipt"));
    assert!(ShipReadinessGap::ReviewReceiptStale
        .describe()
        .contains("stale"));
    assert!(ShipReadinessGap::EarlyStageWithCommits
        .describe()
        .contains("already committed"));
}

#[test]
fn review_receipt_lines_records_and_dry_run_writes_nothing() {
    let (tmp, ctx) = ctx();
    let repo_root = ctx.repo_root();
    let branch = probe_git_branch(&repo_root);
    let head_full = probe_git_head_full_sha(&repo_root).unwrap();

    let lines = review_receipt_lines(&ctx, &repo_root, branch.as_deref(), None, None).unwrap();
    assert!(lines.join("\n").contains(&format!("head: {head_full}")));

    let state = State::load(&ctx.config_dir.join("state.json")).unwrap();
    let receipt = state
        .review_receipt_for_branch(&repo_root, branch.as_deref())
        .unwrap();
    assert_eq!(receipt.head_sha, head_full);
    assert!(receipt.override_reason.is_none());

    // Override reason is persisted verbatim.
    review_receipt_lines(
        &ctx,
        &repo_root,
        branch.as_deref(),
        None,
        Some("skipped for spike".to_string()),
    )
    .unwrap();
    let state = State::load(&ctx.config_dir.join("state.json")).unwrap();
    let receipt = state
        .review_receipt_for_branch(&repo_root, branch.as_deref())
        .unwrap();
    assert_eq!(
        receipt.override_reason.as_deref(),
        Some("skipped for spike")
    );

    // Dry-run records nothing.
    let dry_config = tmp.path().join("dry-ce-ai");
    let dry_ctx = Context::resolve(Some(dry_config.clone()), true, false, true).unwrap();
    review_receipt_lines(&dry_ctx, &repo_root, branch.as_deref(), None, None).unwrap();
    assert!(!dry_config.join("state.json").exists());
}

#[test]
fn review_receipt_rejects_unresolvable_head() {
    let (_tmp, ctx) = ctx();
    let repo_root = ctx.repo_root();
    let err = review_receipt_lines(
        &ctx,
        &repo_root,
        Some("feat/x"),
        Some("this-is-not-a-git-object".to_string()),
        None,
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 2);
}

fn run_git(repo: &std::path::Path, args: &[&str]) {
    let mut cmd = std::process::Command::new("git");
    for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        cmd.env_remove(var);
    }
    cmd.args(args).current_dir(repo).output().unwrap();
}

#[test]
fn ship_readiness_probes_real_git_commits_and_solutions() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();
    run_git(repo, &["init", "-b", "main"]);
    std::fs::write(repo.join("README.md"), "base\n").unwrap();
    run_git(repo, &["add", "."]);
    run_git(
        repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "base",
        ],
    );

    run_git(repo, &["checkout", "-b", "feat/x"]);
    std::fs::write(repo.join("src.rs"), "fn main() {}\n").unwrap();
    run_git(repo, &["add", "."]);
    run_git(
        repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "feat",
        ],
    );

    assert_eq!(probe_commits_ahead(repo), 1);
    assert!(!probe_stage6_artifact(repo, &[]));
    assert_eq!(
        evaluate_ship_readiness(1, false, None, None, WorkflowStage::Verification),
        vec![
            ShipReadinessGap::Stage6Missing,
            ShipReadinessGap::ReviewReceiptMissing
        ]
    );

    // Add a Stage 6 learning and confirm detection.
    std::fs::create_dir_all(repo.join("docs").join("solutions").join("bugfixes")).unwrap();
    std::fs::write(
        repo.join("docs")
            .join("solutions")
            .join("bugfixes")
            .join("x.md"),
        "# learning\n",
    )
    .unwrap();
    run_git(repo, &["add", "."]);
    run_git(
        repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "docs",
        ],
    );
    assert!(probe_stage6_artifact(repo, &[]));
    assert_eq!(probe_commits_ahead(repo), 2);
}

#[test]
fn stage6_detection_ignores_deletions() {
    // A branch whose only docs/solutions change is a deletion must NOT count as
    // a Stage 6 artifact (else it silently reports "ready").
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();
    run_git(repo, &["init", "-b", "main"]);
    std::fs::create_dir_all(repo.join("docs").join("solutions").join("bugfixes")).unwrap();
    std::fs::write(
        repo.join("docs")
            .join("solutions")
            .join("bugfixes")
            .join("a.md"),
        "# a\n",
    )
    .unwrap();
    run_git(repo, &["add", "."]);
    run_git(
        repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "base",
        ],
    );

    run_git(repo, &["checkout", "-b", "feat/del"]);
    run_git(repo, &["rm", "docs/solutions/bugfixes/a.md"]);
    run_git(
        repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "remove",
        ],
    );

    assert!(
        !probe_stage6_artifact(repo, &[]),
        "a deleted docs/solutions file must not satisfy Stage 6"
    );
}

#[test]
fn review_receipt_clears_gap_and_goes_stale_after_new_commit() {
    let tmp = TempDir::new().unwrap();
    let repo = tmp.path();
    run_git(repo, &["init", "-b", "main"]);
    std::fs::write(repo.join("README.md"), "base\n").unwrap();
    run_git(repo, &["add", "."]);
    run_git(
        repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "base",
        ],
    );
    run_git(repo, &["checkout", "-b", "feat/x"]);
    std::fs::write(repo.join("src.rs"), "fn main() {}\n").unwrap();
    run_git(repo, &["add", "."]);
    run_git(
        repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "feat",
        ],
    );

    let head_full = probe_git_head_full_sha(repo).unwrap();
    let mut state = State::default();
    state.record_review_receipt(repo, Some("feat/x"), &head_full, None);
    let receipt = state.review_receipt_for_branch(repo, Some("feat/x"));
    assert_eq!(receipt.unwrap().head_sha, head_full);

    // Matching HEAD + Stage 6 present -> no gaps.
    assert!(evaluate_ship_readiness(
        1,
        true,
        receipt,
        Some(&head_full),
        WorkflowStage::GitShipping
    )
    .is_empty());

    // A new commit makes the receipt stale.
    std::fs::write(repo.join("src2.rs"), "x\n").unwrap();
    run_git(repo, &["add", "."]);
    run_git(
        repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-m",
            "more",
        ],
    );
    let new_head = probe_git_head_full_sha(repo).unwrap();
    assert_ne!(head_full, new_head);
    assert_eq!(
        evaluate_ship_readiness(
            2,
            true,
            state.review_receipt_for_branch(repo, Some("feat/x")),
            Some(&new_head),
            WorkflowStage::GitShipping
        ),
        vec![ShipReadinessGap::ReviewReceiptStale]
    );
}
