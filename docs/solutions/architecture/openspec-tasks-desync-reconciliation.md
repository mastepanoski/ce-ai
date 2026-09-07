---
title: "OpenSpec Tasks Checkbox Desync Reconciliation & Non-Blocking Warnings"
category: "architecture"
date: "2026-09-07"
tags:
  - workflow
  - openspec
  - fsm
  - tasks-desync
  - git-reconciliation
  - doctor
  - checkpoints
components:
  - commands::workflow
  - commands::doctor
applies_when: "Investigating why workflow FSM remains stuck at Stage 3 or Stage 4 despite completed code, diagnosing tasks.md checkbox desync, or understanding git diff reconciliation with OpenSpec checklists"
---

# OpenSpec Tasks Checkbox Desync Reconciliation & Non-Blocking Warnings

## Context & Problem

In Compound Engineering workflows, `/ce-plan` authors an executable task breakdown in `openspec/changes/<feature>/tasks.md`, and `/ce-work` is expected to execute each unit and mark the checkboxes (`- [x] Task 1`). The `ce-ai` workflow Finite State Machine (FSM) relies on `infer_stage_from_repo` to determine current development progress by parsing `- [x]` and `- [ ]` marks in `tasks.md`.

However, before Issue #313 (Part 1), if an AI agent or human developer implemented code changes or committed them to the feature branch without updating `tasks.md`:
1. `infer_stage_from_repo` saw `0/N` tasks completed and assumed the project was still at **Stage 3: Execution Plan** (or early Stage 4).
2. The workflow state remained silently desynced from actual repository reality.
3. No diagnostic warning was emitted by `ce-ai workflow resume`, `ce-ai workflow status`, `ce-ai workflow checkpoint`, or `ce-ai doctor`.

## Solution Architecture

Release v1.44.0 introduces git-diff to task-list reconciliation in `src/commands/workflow.rs` and diagnostic warnings across all workflow surfaces.

### 1. Git-Touched Files Extraction

`probe_feature_touched_files` aggregates all code and asset modifications associated with the active feature:
- **Working Tree Changes**: Extracted via `git status --porcelain=v1 -uall`, which ensures untracked directories expand into individual files rather than a single top-level directory token.
- **Branch Committed Changes**: Extracted by finding the merge-base with the target base branch (`origin/main`, falling back to `main`) and running `git diff --name-only <merge_base>...HEAD`.
- **Exclusion Filters**: System files (`.git/`), OpenSpec internal markdown files (`openspec/`), and lockfiles (`Cargo.lock`, `package-lock.json`, etc.) are filtered out to focus strictly on functional code modifications.

### 2. Multi-Level Task Reconciliation

`reconcile_tasks_with_git` matches extracted touched files against unchecked tasks (`- [ ]`) in `tasks.md`:
- **Path Extraction**: `extract_paths_from_task_text` extracts candidate file paths enclosed in markdown backticks (e.g. `` `src/commands/workflow.rs` ``) as well as bare path-like tokens containing directory slashes or standard file extensions.
- **Exact & Prefix Matching**: For each unchecked task, if any referenced path exactly matches a touched file, or matches as a directory prefix/suffix, a desync match (`TaskDesyncMatch`) is recorded.
- **Aggregate Fallback**: If no unchecked task has explicit path matches, but 0 tasks are marked completed (`0/N` with `total_tasks > 0`) and any implementation code file (under `src/`, `tests/`, or `skills/`) is touched, an aggregate desync match (`is_aggregate_desync = true`) is flagged.

### 3. Non-Blocking Diagnostic Warnings

When a desync is detected (`task_desync.is_some_and(|d| d.has_desync())`):
- **`ce-ai workflow resume`**: Injects a warning banner immediately below the `tasks progress` block in the context re-hydration prompt:
  ```markdown
  ! Warning: Tasks desync detected — 2 unchecked tasks reference modified code files:
    - [ ] 1. Implement reconciliation data structures (touches src/commands/workflow.rs)
  ```
- **`ce-ai workflow status`**: Surfaces the desync warning in the CLI output and TUI dashboard.
- **`ce-ai workflow checkpoint`**: Echoes the warning banner while preserving developer sovereignty (the manual checkpoint is saved successfully).
- **`ce-ai doctor`**: Emits `doctor-warn: openspec tasks desync in '<feature>': ...` without adding to fatal findings (exits with code 0).

### 4. FSM Auto-Checkpoint Guard

In `maybe_auto_checkpoint`, when a tasks desync is active, automatic checkpoint advancement is inhibited from advancing past Stage 4 (TDD Work) to Stage 5 (Verification), Stage 6 (Knowledge Capture), or Stage 7 (Git Shipping). This ensures the agent or developer is prompted to reconcile `tasks.md` before the automated FSM marks the feature as ready for verification or shipping.

### 5. Graceful Degradation (R7)

All git operations are designed for graceful degradation:
- Non-git workspaces, missing git binaries, or detached heads cleanly return `None` or empty collections.
- Reconciliation never panics or aborts workflow operations when git inspection is unavailable.
