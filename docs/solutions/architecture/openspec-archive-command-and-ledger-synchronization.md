---
title: "OpenSpec Change Archival CLI Command and Safe Ledger Synchronization"
category: "architecture"
date: "2026-09-11"
tags:
  - openspec
  - workflow
  - archive
  - safe-mover
  - git
  - ledger-hygiene
components:
  - commands::workflow
  - commands::registry
applies_when: "Implementing or invoking OpenSpec change package archival, transitioning completed or rescoped features to archive/, managing ledger synchronization, or preventing dormant specification sprawl"
---

# OpenSpec Change Archival CLI Command and Safe Ledger Synchronization

## Problem & Context
While Issue #323 implemented deterministic Turn-0 detection of completed OpenSpec changes via `probe_unarchived_completed_changes`, moving those packages into `openspec/changes/archive/` was left as a manual convention described in documentation. Over time, 33 completed changes accumulated across active worktrees, triggering persistent warnings in `ce-ai doctor` and `ce-ai workflow resume`.

Leaving archival to manual developer discretion without dedicated CLI tooling caused:
1. **Accumulation of Dormant Changes**: Completed features lingered indefinitely because manual `git mv` and editing `archive/README.md` disrupted development flow.
2. **Inflexibility on Scope Reductions**: Features shipped with abandoned or rescoped tasks could not mechanically achieve 100% `[x]` completion, leaving developers in a dilemma between falsely checking undone tasks or leaving the folder in permanent unarchived dormancy.
3. **Active Feature Pointer Desynchronization**: Manual folder movement left `state.json` pointing to an archived or nonexistent feature, causing `ce-ai workflow resume` context errors.

## Solution Architecture

### 1. Unified CLI Surface
Exposed both a top-level alias `ce-ai archive [feature]` and subcommand `ce-ai workflow archive [feature]`, supporting:
- `[feature]`: Named feature to archive. When omitted without `--all`, automatically defaults to the active feature recorded in the workspace branch state in `state.json`.
- `--all`: Batch sweep mode that discovers all unarchived completed changes via `probe_unarchived_completed_changes` and archives them in a single command.
- `--status "<evidence>"`: Supplies release evidence for Criterion 2 archival.
- `--dry-run`: Previews candidate moves and destination paths with zero filesystem or git mutations.

### 2. Dual-Criteria Archival Evaluation
- **Criterion 1 (Mechanical 100% Checkboxes)**: When all task checkboxes (`- [x]` / `- [X]`) in `tasks.md` are marked complete (`total > 0 && completed == total`), archival proceeds without requiring status annotations.
- **Criterion 2 (STATUS-Attested Rescoping)**: When tasks were cut, deferred, or superseded, supplying `--status "<evidence>"` prepends a formal `> STATUS: <evidence>\n\n` header to `tasks.md`. This satisfies the archival contract documented in `openspec/changes/archive/README.md` while declaring any residual unchecked tasks as unaudited by the header itself.
- **Fail-Closed Verification (Exit Code 6)**: Attempting to archive an incomplete feature without `--status` is rejected with `CeError::Verification`.

### 3. Safe Git-Aware Mover & Collision Guard
- **Collision Rejection (Exit Code 3)**: Checks if `openspec/changes/archive/<feature>` already exists before touching any files.
- **Dirty-Tree Guard**: Probes `git status --porcelain` in the target folder. If uncommitted modifications exist in non-tasks files (e.g. uncommitted changes in `proposal.md` or `spec.md`), the command fails closed with `CeError::Verification` (exit code 6) to prevent archiving unstaged work.
- **Resilient Moving**: Attempts `git mv openspec/changes/<feature> openspec/changes/archive/<feature>`. If `git mv` cannot execute (e.g., untracked files or non-standard worktree state), safely falls back to `std::fs::rename` combined with `git add -A openspec/changes`.

### 4. Ledger & State Reconciliation
- **Audit Ledger Synchronization**: Appends dated entries to `openspec/changes/archive/README.md` (e.g. single-feature entry with task counts/status or batch sweep summary) and stages the updated README in git.
- **Active State Clearing**: Inspects `state.json` and clears `feature_name` (setting it to `None`) if the active feature was archived, preventing stale pointers on subsequent `ce-ai workflow resume` invocations.

## Key Learnings & Gotchas
1. **Dirty Tree Guard Importance**: An AI agent or developer might attempt to archive a spec folder while still having uncommitted draft edits. Verifying git status prevents archiving half-written files.
2. **Git Mover Fallback**: In test environments and temporary checkouts, git index tracking may differ from standard worktrees. Attempting `git mv` with atomic `rename` + `git add` fallback provides maximum cross-platform portability.
3. **Status Attestation Length**: Status strings must carry substantive evidence (e.g., minimum 5 characters citing PR, release, or symbol) to prevent trivial `--status "x"` bypasses.
