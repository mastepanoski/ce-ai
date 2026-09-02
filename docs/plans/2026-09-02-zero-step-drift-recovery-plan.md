---
date: 2026-09-02
topic: zero-step-drift-recovery
status: draft
source: docs/brainstorms/2026-09-02-zero-step-drift-recovery-requirements.md
---

# Plan: Zero-Step Environment Drift Recovery via Live `RepoState` Sync

## Problem Frame & Scope

When human developers or tools modify workspace files, commit code, or checkout branches outside an AI agent's active turn, conversational agents hallucinate for 5–8 turns because stale chat history overpowers reality (arXiv:2608.26263v2).

This plan outlines the implementation of **Zero-Step Environment Drift Recovery** by establishing a live, fast (`<15ms`), and cryptographically exact `RepoState` projection inside `ce-ai workflow resume`, `ce-ai status`, and `ce-ai doctor`.

## Requirements Traceability

- **R1, R2, R3 (Data Model & Fast Probing Engine):** Covered in Unit 2.
- **R4, R5, R6, R7 (CLI & Workflow Integration):** Covered in Unit 3.
- **Acceptance Examples & Regression Verification:** Covered in Unit 4.
- **Address Existing Test Debt (`OpenSpecContextInfo` & `TreeDrift`):** Covered in Unit 1.

## Implementation Units

### Unit 1: Test Debt Coverage & Fixtures (`OpenSpecContextInfo` & `TreeDrift`)
- **Files:** `src/commands/tests/workflow.rs`, `src/commands/tests/sync.rs`
- **Target:** ~45 LOC
- **Approach:**
  - Create test fixtures simulating `openspec/changes/<feature>/` with proposals, specs, and tasks.
  - Verify `probe_openspec_context()` correctly calculates completed tasks count and handles missing directories without panicking.
  - Test `TreeDrift` calculation in `sync.rs` against controlled file sets.

### Unit 2: Data Model & Fast Probing Engine (`RepoState`)
- **Files:** `src/commands/workflow.rs`, `src/commands/mod.rs`
- **Target:** ~65 LOC
- **Approach:**
  - Define `RepoState` struct with `git_branch`, `head_sha`, `is_git_clean`, `modified_files`, `manifest_drift_count`, `agents_block_valid`, and `openspec_context`.
  - Implement `probe_repo_state(ctx: &Context, wf: &Option<WorkflowState>) -> RepoState`.
  - Execute git status inspection via `git status --porcelain=v1` with graceful non-git fallback.
  - Connect manifest diff calculation using `crate::state::diff::diff`.
  - Verify `AGENTS.md` adoption block SHA256 integrity against `state.projects`.

### Unit 3: Integration into `workflow resume` & `status`
- **Files:** `src/commands/workflow.rs`, `src/commands/status.rs`
- **Target:** ~50 LOC
- **Approach:**
  - Format `== [Environment State & Drift Status] ==` block in `resume_lines()`.
  - Serialize `repo_state` in `Action::Resume { json }`.
  - Surface git branch and working tree dirty indicator in `status_lines()`.
  - Print informative warning when `manifest_drift_count > 0` without blocking exit code.

### Unit 4: Integration & Regression Tests for Zero-Step Drift Recovery
- **Files:** `tests/cli.rs`
- **Target:** ~50 LOC
- **Approach:**
  - Add integration tests executing `ce-ai workflow resume` and `ce-ai workflow resume --json`.
  - Verify clean tree output.
  - Verify dirty tree (modified files, switched branch) accurately surfaces in Turn 0.
  - Verify manifest drift warning output.

### Unit 5: Version Bump, Changelog & Full Quality Gates
- **Files:** `Cargo.toml`, `CHANGELOG.md`
- **Target:** ~10 LOC
- **Approach:**
  - Bump SemVer in `Cargo.toml`.
  - Document changes in `CHANGELOG.md`.
  - Run full verification: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`.

---
*Composed 2026-09-02 by ce-plan from `docs/brainstorms/2026-09-02-zero-step-drift-recovery-requirements.md` and `openspec/changes/zero-step-drift-recovery/`*
