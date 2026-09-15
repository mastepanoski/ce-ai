---
title: "OpenSpec Task Checkbox Reconciliation and Ledger Hygiene"
category: "architecture"
problem_type: "architecture"
date: "2026-09-15"
applies_when: "Diagnosing OpenSpec tasks desynchronization with git changes, preventing completed changes from lingering unarchived, or understanding Turn-0 ledger delivery"
tags:
  - openspec
  - workflow
  - doctor
  - tasks-desync
  - git-reconciliation
  - turn-0
  - ledger-hygiene
components:
  - src/commands/workflow.rs
  - src/commands/doctor.rs
---

# OpenSpec Task Checkbox Reconciliation and Ledger Hygiene

## Problem Statement
In Compound Engineering workflows, `/ce-plan` generates an executable task checklist in `openspec/changes/<feature>/tasks.md`, and `/ce-work` is expected to check off items as implementation proceeds (`- [x] Task 1`). Over time, two symmetric failure modes emerged when human developers or autonomous AI agents developed code without keeping task checklists synchronized:

1. **In-Flight Task Desynchronization (Code Modified, Checkboxes Stale)**:
   - Changes are staged, modified, or committed on a feature branch, but `tasks.md` remains `0/N` or partially checked.
   - The workflow Finite State Machine (FSM) infers that the project is still at Stage 3 (Plan) or early Stage 4, causing automated tools to misjudge progression.
2. **Post-Completion Archival Blind Spot (Checkboxes Done, Folder Lingering)**:
   - All tasks in `tasks.md` are marked complete (`N/N`), and PRs are merged to `main`, but the change folder remains unarchived in `openspec/changes/`.
   - Sibling completed folders accumulate silently, polluting agent context windows and file tree scans.

## Unified Architecture: Two-Sided Task Lifecycle Governance

`ce-ai` solves both problems deterministically by anchoring task-to-git reconciliation and repo-wide ledger audits into non-discretionary Turn-0 delivery mechanisms across all supported harnesses.

### 1. In-Flight Reconciliation (`reconcile_tasks_with_git`)
When a feature is active, `ce-ai` correlates real git disk modifications with unchecked tasks (`- [ ]`):

- **Git-Touched Files Extraction (`probe_feature_touched_files`)**:
  - Aggregates untracked and working tree changes via `git status --porcelain=v1 -uall` (expanding untracked directory trees into individual file paths).
  - Inspects branch-committed changes by resolving the merge-base with the default branch (`git diff --name-only <merge_base>...HEAD`).
  - Filters out system directories (`.git/`), OpenSpec markdown files (`openspec/`), and lockfiles (`Cargo.lock`, `package-lock.json`).
- **Path Matching & Fallback**:
  - Extracts code paths enclosed in backticks or path tokens from task descriptions (`extract_paths_from_task_text`).
  - Matches paths against touched files (exact match, prefix match, or suffix match).
  - If 0 tasks are checked but implementation code files under `src/`, `tests/`, or `skills/` are modified, an aggregate desync match (`is_aggregate_desync = true`) is flagged.
- **FSM Auto-Checkpoint Guard**:
  - In `maybe_auto_checkpoint`, an active tasks desync inhibits automatic stage advancement past Stage 4 (TDD Work) to Stage 5 (Verification), Stage 6 (Compound Learning), or Stage 7 (Git Shipping). This prevents automated transitions until the operator or agent reconciles `tasks.md`.

### 2. Post-Completion Ledger Auditing (`probe_unarchived_completed_changes`)
When features reach completion, `ce-ai` audits the entire change inventory:

- **Shared Checkbox Counting Primitives (`count_task_checkboxes`)**:
  - Robust regex parsing that handles `- [x]`, `- [X]`, and `- [ ]` lines, gracefully returning `(completed_tasks, total_tasks)` with zero-panic fallback on missing files or I/O errors.
- **Repo-Wide Inventory Scan**:
  - Iterates across all direct child directories of `openspec/changes/` (excluding `archive/` and hidden folders).
  - Flags any folder where `total_tasks > 0 && completed_tasks == total_tasks`.
- **`RepoState` Structured Snapshot**:
  - Records unarchived changes in `repo_state.unarchived_completed_changes`, making findings available to programmatic callers, JSON serializers (`--json`), and CLI formatters.

### 3. Multi-Surface Non-Blocking Delivery Matrix

Both in-flight desyncs and unarchived completions are delivered across the standard workflow surfaces without blocking routine execution:

| Surface | In-Flight Desync Behavior | Unarchived Completed Behavior |
| :--- | :--- | :--- |
| **`ce-ai workflow resume`** (Turn-0) | Injects prominent warning banner below task progress block | Emits compact ledger summary line: `openspec ledger: ! N change(s) complete but not archived` |
| **`ce-ai workflow status` / TUI** | Displays desync warning and highlights affected tasks | Surfaces unarchived completed changes in dashboard |
| **`ce-ai workflow checkpoint`** | Echoes desync banner while honoring manual checkpoint recording | Echoes unarchived completed warning |
| **`ce-ai doctor`** | Non-fatal diagnostic: `doctor-warn: openspec tasks desync in '<feature>'` | Non-fatal diagnostic: `doctor-warn: openspec change '<feature>' is complete (N/N tasks) but not archived` |

### 4. Graceful Degradation
All git operations degrade safely:
- In environments without a git binary, outside a git repository, or in detached HEAD states, git inspection functions return `None` or empty vectors without crashing.
- Diagnostic surfaces report `[git: n/a]` or fallback to filesystem metadata rather than emitting false positives.

## Verification & Key Learnings
- **Unit Tests**: In `src/commands/tests/workflow.rs`, tests verify `reconcile_tasks_with_git`, path extraction, aggregate fallbacks, and checkbox counting across corrupted and valid task files.
- **Integration Tests**: In `tests/cli.rs`, tests verify that `workflow resume`, `status`, and `doctor` surface both desync warnings and unarchived change notices.
- **Turn-0 Invariant**: Relying solely on `doctor` for task hygiene fails because developers do not run doctor during ordinary coding turns. Embedding diagnostics into `workflow resume` (fired by harness `SessionStart` hooks) provides non-discretionary observability at zero cognitive overhead.
