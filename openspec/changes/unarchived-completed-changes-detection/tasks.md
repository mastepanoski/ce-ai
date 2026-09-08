# Tasks: Deterministic Detection of Unarchived Completed OpenSpec Changes

## Work Unit 1: Shared Task Checkbox Counting & Degradation (~35 LOC)
- [x] 1.1 Extract `count_task_checkboxes(tasks_path: &Path) -> (usize, usize)` in `src/commands/workflow.rs`.
- [x] 1.2 Refactor `probe_openspec_context_in` in `src/commands/workflow.rs` to reuse `count_task_checkboxes`.
- [x] 1.3 Add unit test `test_count_task_checkboxes_parsing_and_toctou_degradation` in `src/commands/tests/workflow.rs`.

## Work Unit 2: Repo-Wide Probe & `RepoState` Integration (~55 LOC)
- [x] 2.1 Define `UnarchivedChange` struct with `feature: String`, `completed_tasks: usize`, `total_tasks: usize` deriving `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`.
- [x] 2.2 Add `unarchived_completed_changes: Vec<UnarchivedChange>` to `RepoState`.
- [x] 2.3 Implement `probe_unarchived_completed_changes(repo_root: &Path) -> Vec<UnarchivedChange>`.
- [x] 2.4 Wire `probe_unarchived_completed_changes` into `probe_repo_state`.
- [x] 2.5 Add unit test `test_probe_unarchived_completed_changes_multi_directory` in `src/commands/tests/workflow.rs`.

## Work Unit 3: Surface Integration (`resume`, `status`, `checkpoint`, `doctor`) (~45 LOC)
- [x] 3.1 In `resume_lines`, add `openspec ledger:` compact line to `"== [Environment State & Drift Status] =="`.
- [x] 3.2 In `status_lines`, append warning line when `unarchived_completed_changes` is non-empty.
- [x] 3.3 In `checkpoint_lines`, append warning line when `unarchived_completed_changes` is non-empty.
- [x] 3.4 In `src/commands/doctor.rs`, iterate unarchived changes and emit `doctor-warn: ...` (non-fatal, exit 0).

## Work Unit 4: CLI Integration Tests (~65 LOC)
- [x] 4.1 Implement `workflow_and_doctor_detect_unarchived_completed_changes` in `tests/cli.rs` verifying:
  - `ce-ai workflow resume` plain text contains `openspec ledger: ! 1 change(s) complete but not archived`
  - `ce-ai workflow resume --json` contains the summary in `additionalContext` and `repo_state.unarchived_completed_changes`
  - `ce-ai workflow status` contains the warning line
  - `ce-ai doctor` prints `doctor-warn: openspec change 'feat-done' is complete (2/2 tasks) but not archived` and exits 0.

## Work Unit 5: Versioning & Documentation (~20 LOC)
- [x] 5.1 Bump SemVer to `1.45.0` in `Cargo.toml` and update `Cargo.lock`.
- [x] 5.2 Add `1.45.0` release notes to `CHANGELOG.md`.
