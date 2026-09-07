# Tasks: OpenSpec Tasks Checkbox Desync Reconciliation & Warning

- [ ] 1. Implement reconciliation data structures and extraction logic in `src/commands/workflow.rs` (~120 LOC)
  - Define `TaskDesyncMatch` and `TaskDesyncReport` with `has_desync()` and `warning_line()`.
  - Implement `extract_paths_from_task_text` extracting backticked paths and path-like tokens.
  - Implement `probe_branch_committed_files` and `probe_feature_touched_files` collecting working tree and branch changes excluding `openspec/` and lockfiles.
  - Implement `reconcile_tasks_with_git(repo_root, feature, tasks_path, touched_files) -> Option<TaskDesyncReport>` with exact, prefix, suffix, and aggregate matching.
- [ ] 2. Integrate reconciliation into `RepoState`, `resume_lines`, `status_lines`, and `checkpoint_lines` (~50 LOC)
  - Add `task_desync: Option<TaskDesyncReport>` to `RepoState`.
  - Wire warning banner injection in `resume_lines` under the `tasks progress` block.
  - Wire warning banner in `status_lines` and `checkpoint_lines`.
  - Wire FSM guard in `maybe_auto_checkpoint` to prevent auto-advancement to `Verification`/`Ship` when tasks are desynced.
- [ ] 3. Add diagnostic probe in `src/commands/doctor.rs` (~30 LOC)
  - Check active workspace OpenSpec change for tasks desync.
  - Print `doctor-warn: openspec tasks desync in '<feature>': ...` without adding to fatal `findings` (exit code 0).
- [ ] 4. Unit tests in `src/commands/tests/workflow.rs` (~100 LOC)
  - Test `extract_paths_from_task_text` with backticks, raw file paths, and noisy text.
  - Test `reconcile_tasks_with_git` with exact file match, directory prefix match, aggregate fallback, and all-completed pass.
  - Test `warning_line` formatting.
- [ ] 5. CLI integration tests in `tests/cli.rs` (~80 LOC)
  - Test `ce-ai workflow resume` outputs desync warning when code is touched but `tasks.md` has 0/N completed.
  - Test `ce-ai workflow status` outputs desync warning.
  - Test `ce-ai doctor` emits `doctor-warn:` and exits 0 on desync.
  - Test `ce-ai workflow checkpoint` saves checkpoint and echoes warning without failing.
- [ ] 6. Documentation and Solution Architecture capture (~40 LOC)
  - Document tasks desync diagnostics in `docs/user-guide/` or relevant command guides.
  - Capture solution in `docs/solutions/architecture/openspec-tasks-desync-reconciliation.md`.
- [ ] 7. SemVer version bump and CHANGELOG update (~15 LOC)
  - Bump version to `1.44.0` in `Cargo.toml`.
  - Document changes in `CHANGELOG.md` following Keep a Changelog standard.

Total estimated LOC: ~435 lines.
