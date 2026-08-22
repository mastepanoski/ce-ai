# Implementation Plan: Context-Exhaustion Resilience & Deterministic Workflow Invariants

**Date**: 2026-08-22  
**Target Feature**: Context-Exhaustion Resilience (Issue #97)  
**OpenSpec Location**: `openspec/changes/context_exhaustion_resilience/`  

---

## Stage Breakdown

### Stage 1: Automated Scripting & Pre-Commit Hook Extension
- Create `scripts/protect-branch.sh` to configure GitHub API branch protection on `main` using `gh api`.
- Update `.githooks/pre-commit` and `Makefile` (`make hooks`) to ensure `core.hooksPath` is set.

### Stage 2: `ce-ai doctor` Health Probes
- Add `check_branch_protection_health` in `src/commands/doctor.rs`.
- Add `check_git_hooks_health` in `src/commands/doctor.rs`.
- Add unit/integration tests in `tests/cli.rs`.

### Stage 3: Compact Invariant Index in `AGENTS.md`
- Insert a high-density, concise ~25-line Hard-Gate Invariant Index at the top of `AGENTS.md`.

### Stage 4: Verification & Shipping
- Execute `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
- Submit PR, verify 100% green CI matrix, and merge into `main`.
