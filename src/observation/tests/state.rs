use std::fs;
use tempfile::tempdir;

use crate::observation::state::{
    observe_handoff, observe_knowledge, observe_openspec, observe_plan, ObservableWorkflowState,
};

#[test]
fn test_empty_repository_observation() {
    let temp_repo = tempdir().expect("tempdir");
    let temp_config = tempdir().expect("tempdir");

    let state = ObservableWorkflowState::observe(temp_repo.path(), temp_config.path())
        .expect("observe state");

    assert!(!state.active_work);
    assert_eq!(state.plan, None);
    assert_eq!(state.openspec, None);
    assert!(!state.verification.has_evidence);
    assert!(!state.verification.review_receipt_stamped);
    assert_eq!(state.verification.uncommitted_changes, 0);
    assert_eq!(state.handoff, None);
    assert!(!state.knowledge_capture.required);
    assert!(!state.knowledge_capture.doc_detected);
}

#[test]
fn test_plan_detection_and_checkbox_counting() {
    let temp_repo = tempdir().expect("tempdir");
    let plans_dir = temp_repo.path().join("docs").join("plans");
    fs::create_dir_all(&plans_dir).expect("create plans dir");

    let plan_content = r#"# Sample Feature Plan

## Tasks
- [x] First completed unit
- [ ] Second in progress unit
- [x] Third completed unit
"#;
    let plan_path = plans_dir.join("2026-10-05-sample-feature-plan.md");
    fs::write(&plan_path, plan_content).expect("write plan");

    let plan_obs = observe_plan(temp_repo.path(), Some("feat/sample-feature"))
        .expect("plan should be observed");

    assert_eq!(plan_obs.title, Some("Sample Feature Plan".to_string()));
    assert_eq!(plan_obs.completed_items, 2);
    assert_eq!(plan_obs.total_items, 3);
    assert!(!plan_obs.is_requirements_only);
    assert_eq!(plan_obs.path, plan_path);
}

#[test]
fn test_plan_requirements_only() {
    let temp_repo = tempdir().expect("tempdir");
    let plans_dir = temp_repo.path().join("docs").join("plans");
    fs::create_dir_all(&plans_dir).expect("create plans dir");

    let plan_content = r#"# Requirements Framing
This document details user story requirements and problem statement.
No checkboxes are defined yet.
"#;
    let plan_path = plans_dir.join("2026-10-05-requirements-only-plan.md");
    fs::write(&plan_path, plan_content).expect("write plan");

    let plan_obs = observe_plan(temp_repo.path(), None).expect("plan should be observed");

    assert_eq!(plan_obs.title, Some("Requirements Framing".to_string()));
    assert_eq!(plan_obs.completed_items, 0);
    assert_eq!(plan_obs.total_items, 0);
    assert!(plan_obs.is_requirements_only);
}

#[test]
fn test_openspec_detection_optional_and_sealed() {
    let temp_repo = tempdir().expect("tempdir");
    let openspec_dir = temp_repo.path().join("openspec").join("changes");
    fs::create_dir_all(&openspec_dir).expect("create openspec dir");

    // Initially no active change directory
    assert_eq!(observe_openspec(temp_repo.path(), None), None);

    // Create an archive folder (should be ignored)
    let archive_dir = openspec_dir.join("archive").join("old-feat");
    fs::create_dir_all(&archive_dir).expect("create archive");
    assert_eq!(observe_openspec(temp_repo.path(), None), None);

    // Create an active feature directory with uncompleted tasks
    let feat_dir = openspec_dir.join("my-cool-feat");
    fs::create_dir_all(&feat_dir).expect("create feat dir");
    fs::write(feat_dir.join("tasks.md"), "- [x] Task 1\n- [ ] Task 2\n").expect("write tasks");

    let obs = observe_openspec(temp_repo.path(), Some("feat/my-cool-feat"))
        .expect("should observe active change");
    assert_eq!(obs.feature, "my-cool-feat");
    assert!(!obs.is_sealed);

    // Now seal all tasks
    fs::write(feat_dir.join("tasks.md"), "- [x] Task 1\n- [x] Task 2\n")
        .expect("write sealed tasks");

    let obs_sealed = observe_openspec(temp_repo.path(), Some("feat/my-cool-feat"))
        .expect("should observe active change");
    assert!(obs_sealed.is_sealed);
}

#[test]
fn test_handoff_detection() {
    let temp_repo = tempdir().expect("tempdir");
    assert_eq!(observe_handoff(temp_repo.path()), None);

    let ce_dir = temp_repo.path().join(".compound-engineering");
    fs::create_dir_all(&ce_dir).expect("create ce dir");
    let handoff_file = ce_dir.join("handoff.md");
    fs::write(&handoff_file, "# Handoff notes\n").expect("write handoff");

    let handoff = observe_handoff(temp_repo.path()).expect("handoff should exist");
    assert!(handoff.exists);
    assert_eq!(handoff.path, handoff_file);
    assert!(handoff.last_modified.is_some());
}

#[test]
fn test_knowledge_detection() {
    let temp_repo = tempdir().expect("tempdir");
    let modified = vec!["src/main.rs".to_string()];

    // Initially no solution doc
    let k1 = observe_knowledge(temp_repo.path(), &modified);
    assert!(k1.required);
    assert!(!k1.doc_detected);
    assert_eq!(k1.doc_path, None);

    // Create solution doc in subcategory
    let sol_dir = temp_repo
        .path()
        .join("docs")
        .join("solutions")
        .join("architecture");
    fs::create_dir_all(&sol_dir).expect("create sol dir");
    let sol_file = sol_dir.join("v2-migration.md");
    fs::write(&sol_file, "# V2 Migration Solution\n").expect("write sol");

    let k2 = observe_knowledge(temp_repo.path(), &modified);
    assert!(k2.required);
    assert!(k2.doc_detected);
    assert_eq!(k2.doc_path, Some(sol_file));
}
