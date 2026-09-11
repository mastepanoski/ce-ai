# Proposal: Deterministic Detection of Unarchived Completed OpenSpec Changes

## Problem Statement
In `ce-ai v1.44.0` (Issue #313), a tasks-desync probe was introduced to reconcile modified code with OpenSpec task checkboxes. However, that probe was strictly scoped to a single active feature resolved by `probe_openspec_context_in`. It never scanned sibling directories under `openspec/changes/`.

Consequently, fully completed change folders (where 100% of tasks in `tasks.md` are marked `[x]`) can linger unarchived indefinitely without triggering any warnings or health checks. In a recent audit, 7 completed change folders accumulated in `openspec/changes/` unnoticed, requiring a manual sweep in PR #322.

A critical design requirement is that detection cannot be confined to `ce-ai doctor`. `ce-ai doctor` is a discretionary tool invoked manually by human developers or AI agents; non-compound-engineering sessions (such as quick hotfixes or targeted refactors) would continue to miss the ledger drift. Detection must be deterministic and ubiquitous, anchoring to the non-discretionary Turn-0 `ce-ai workflow resume` hook injected natively into session startups across harnesses, while also surfacing in `workflow status`, `workflow checkpoint`, and `ce-ai doctor`.

## In-Scope
1. **Shared Checkbox Counting Helper**: Extract `count_task_checkboxes(tasks_path: &Path) -> (usize, usize)` in `src/commands/workflow.rs`, returning `(completed, total)` and degrading gracefully on missing or unreadable files.
2. **Repo-Wide Completed Changes Probe**: Implement `probe_unarchived_completed_changes(repo_root: &Path) -> Vec<UnarchivedChange>`, scanning every non-archive subdirectory under `openspec/changes/` and detecting any with `total_tasks > 0 && completed_tasks == total_tasks`.
3. **RepoState Integration**: Extend `RepoState` with `pub unarchived_completed_changes: Vec<UnarchivedChange>`, populated during `probe_repo_state`.
4. **Turn-0 Resume Surface**: In `resume_lines`, emit a compact, single-line ledger summary in `"== [Environment State & Drift Status] =="` (`openspec ledger: clean (0 pending archival)` or `openspec ledger: ! N change(s) complete but not archived — run 'ce-ai doctor' for details`).
5. **Workflow Status & Checkpoint Surfaces**: In `status_lines` and `checkpoint_lines`, append a non-blocking warning when unarchived completed changes are detected.
6. **Detailed Doctor Diagnostic**: In `ce-ai doctor`, iterate through detected unarchived changes and emit verbose non-fatal warnings (`doctor-warn: openspec change '<feature>' is complete (N/N tasks) but not archived — see openspec/changes/archive/README.md`) exiting 0.
7. **Empirical Verification**: Unit tests for checkbox counting, TOCTOU graceful degradation, repo-wide probing, and CLI integration tests in `tests/cli.rs` covering `resume`, `resume --json`, `status`, and `doctor`.

## Out-of-Scope
- **Automatic Archiving**: `ce-ai` must never perform automated `git mv` operations on `openspec/changes/`. Moving folders to archive remains an explicit developer or agent action.
- **Modifying Active Feature Desync Behavior**: The existing active feature desync logic in `reconcile_tasks_with_git` remains functionally unchanged.
- **Inferring Criterion (2) (STATUS-verified shipped)**: Detecting whether an uncompleted task list has a valid human-attested STATUS header requires human governance and is not inferred automatically.

## Risk Evaluation & Mitigation
- **Risk (TOCTOU Filesystem Races):** A folder or file in `openspec/changes/` might be moved, deleted, or partially written while `probe_unarchived_completed_changes` runs.
  - *Mitigation:* Treat every directory and file read as fallible (`.ok()`, `.flatten()`). Missing or malformed files fail silently on that iteration without aborting or panicking.
- **Risk (Context Bloat in Turn-0 Resume):** If many completed folders linger, dumping every folder name could overwhelm model token context in `resume`.
  - *Mitigation:* Enforce a strict single-line summary in `resume_lines`, `status_lines`, and `checkpoint_lines`, directing users to `ce-ai doctor` for the granular folder list.
