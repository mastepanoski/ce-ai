---
title: "Deterministic Detection of Unarchived Completed OpenSpec Changes"
category: "architecture"
date: "2026-09-07"
tags:
  - openspec
  - workflow
  - doctor
  - turn-0
  - ledger-hygiene
  - repo-state
components:
  - commands::workflow
  - commands::doctor
applies_when: "Preventing completed OpenSpec changes from lingering unarchived in openspec/changes/, ensuring repo-wide task ledger cleanliness, or architecting non-discretionary Turn-0 health probes"
---

# Deterministic Detection of Unarchived Completed OpenSpec Changes

## Problem & Context
When features are implemented, reviewed, and merged into `main`, their corresponding OpenSpec change folders in `openspec/changes/<feature>/` should be archived to `openspec/changes/archive/`. Previously, `ce-ai doctor` only probed the single active feature (`probe_openspec_context_in`), leaving a blind spot where sibling folders with 100% completed tasks remained dormant indefinitely.

A critical design finding is that placing diagnostic probes solely in `ce-ai doctor` is insufficient for deterministic enforcement:
1. **Developer / Agent Workflow Discretion:** In non-compound development loops (such as bugfixes or maintenance), developers and AI agents do not proactively run `ce-ai doctor`.
2. **Turn-0 Invariant:** To be truly deterministic, detection must attach to the non-discretionary Turn-0 delivery mechanism that fires automatically on every session startup across all supported harnesses (`ce-ai workflow resume` via `SessionStart` and `PreInvocation` hooks).

## Solution Architecture
1. **Shared Checkbox Counting (`count_task_checkboxes`)**:
   Extracted pure checkbox counting logic from `probe_openspec_context_in` into a shared, robust helper that parses `- [x]`, `- [X]`, and `- [ ]` lines, returning `(completed_tasks, total_tasks)` with graceful `(0, 0)` degradation on missing files or I/O errors.
2. **Repo-Wide Probe (`probe_unarchived_completed_changes`)**:
   Iterates through all direct subdirectories in `openspec/changes/` (excluding `archive`), identifying any directory whose `tasks.md` has `total > 0 && completed == total`.
3. **`RepoState` Integration**:
   Populates `repo_state.unarchived_completed_changes: Vec<UnarchivedChange>`, ensuring the information is available to both plain-text rendering and JSON payloads (`--json`).
4. **Multi-Surface Diagnostic Delivery**:
   - **`ce-ai workflow resume`**: Emits a compact, single-line ledger summary in `"== [Environment State & Drift Status] =="` (`openspec ledger: clean (0 pending archival)` or `openspec ledger: ! N change(s) complete but not archived — run 'ce-ai doctor' for details`).
   - **`ce-ai workflow status` & `checkpoint`**: Emits a non-blocking warning when unarchived completed changes exist.
   - **`ce-ai doctor`**: Emits granular `doctor-warn:` lines per feature with `(N/N tasks)` without elevating to fatal findings (exits 0).
