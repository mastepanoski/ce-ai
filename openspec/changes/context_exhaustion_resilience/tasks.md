# Task Checklist: Context-Exhaustion Resilience Implementation

- [ ] **Stage 1: Automated Scripting & Pre-Commit Hook Extension**
  - [ ] Create `scripts/protect-branch.sh` to configure GitHub API branch protection on `main`.
  - [ ] Add `core.hooksPath` configuration hint to `make hooks`.

- [ ] **Stage 2: `ce-ai doctor` Diagnostic Probes**
  - [ ] Implement `check_branch_protection_health` in `src/commands/doctor.rs`.
  - [ ] Implement `check_git_hooks_health` in `src/commands/doctor.rs`.
  - [ ] Add integration tests in `tests/cli.rs` verifying doctor findings.

- [ ] **Stage 3: Compact Invariant Index in `AGENTS.md`**
  - [ ] Add concise, high-density ~25-line Invariant Index block at the top of `AGENTS.md`.
  - [ ] Verify line budget (<= 30 lines) and markdown formatting.

- [ ] **Stage 4: Verification & Governance**
  - [ ] Run `cargo fmt --check`.
  - [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [ ] Run `cargo test`.
  - [ ] Open PR, verify 100% green CI matrix, and merge to `main`.
