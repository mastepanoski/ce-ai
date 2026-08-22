---
type: feat
origin: openspec/changes/context_exhaustion_resilience/
date: 2026-08-22
issue: 97
---

# Implementation Plan: Context-Exhaustion Resilience & Deterministic Workflow Invariants

## Implementation Units

### U1. Automated Scripting & Pre-Commit Hook Extension
- **Goal**: Provide automated scripts to configure GitHub branch protection on `main` and enforce local git hooks.
- **Files**: `scripts/protect-branch.sh`, `.githooks/pre-commit`, `Makefile`.
- **Approach**:
  - Create `scripts/protect-branch.sh` using `#!/usr/bin/env bash`. Use `gh repo view --json nameWithOwner` to auto-detect `{owner}/{repo}` and send complete `PUT /repos/{owner}/{repo}/branches/main/protection` payload with status check contexts.
  - Update `Makefile` (`make hooks`) to execute `git config core.hooksPath .githooks` and verify executable permissions (`chmod +x .githooks/pre-commit`).
- **Test scenarios**: Test `scripts/protect-branch.sh` execution when unauthenticated (graceful error message) and verify `git config --get core.hooksPath` returns `.githooks`.
- **Verification**: Run `make hooks` and verify `.githooks/pre-commit` fires before git commit.

### U2. `ce-ai doctor` Health Probes
- **Goal**: Implement diagnostic probes in `ce-ai doctor` for GitHub branch protection, git hooks, and invariant index SHA drift.
- **Files**: `src/commands/doctor.rs`, `tests/cli.rs`.
- **Approach**:
  - Implement `check_branch_protection_health`: Check for `gh` CLI availability and remote origin before running `gh api`. If offline or unauthenticated, emit `doctor-info: gh api unavailable, skipping branch protection probe` instead of a hard error.
  - Implement `check_git_hooks_health`: Normalize `core.hooksPath` using `PathBuf` relative resolution and verify `.githooks/pre-commit` exists and is executable.
  - Implement `check_invariant_index_health`: Verify `AGENTS.md` header invariant index block SHA hash.
- **Test scenarios**: Hermetic tests in `tests/cli.rs` (`doctor_reports_git_hooks_unconfigured` and `doctor_reports_git_hooks_ok`) inside synthetic `TempDir` repos.
- **Verification**: Run `cargo test doctor` and `ce-ai doctor`.

### U3. Compact Invariant Index in `AGENTS.md`
- **Goal**: Add a high-density, concise Hard-Gate Invariant Index (~25 lines, maximum <= 30 lines) at the top of `AGENTS.md`.
- **Files**: `AGENTS.md`.
- **Approach**: Insert imperative non-negotiable rules (never push to main, wait for CI green, atomic writes, no dummy fallbacks) at the top of `AGENTS.md` above detailed architecture sections.
- **Test scenarios**: Line count verification (`wc -l` header section <= 30 lines).
- **Verification**: Verify header rendering in GFM and check line count budget.

### U4. Verification & Shipping
- **Goal**: Execute 100% clean verification matrix across Linux, macOS, and Windows.
- **Files**: Workspace-wide.
- **Approach**: Run formatting, clippy linting, and full unit/CLI integration test suite.
- **Test scenarios**: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
- **Verification**: Submit PR, achieve 100% green CI matrix status, and merge into `main`.
