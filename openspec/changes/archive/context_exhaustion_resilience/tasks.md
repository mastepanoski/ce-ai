# Task Checklist: Context-Exhaustion Resilience Implementation

- [x] **Stage 1: Automated Scripting & Pre-Commit Hook Extension**
  - [x] Create `scripts/protect-branch.sh` to configure GitHub API branch protection on `main`.
  - [x] Add `core.hooksPath` configuration hint to `make hooks`.

- [x] **Stage 2: `ce-ai doctor` Diagnostic Probes**
  - [x] Implement `check_branch_protection_health` in `src/commands/doctor.rs`.
  - [x] Implement `check_git_hooks_health` in `src/commands/doctor.rs`.
  - [x] Add integration tests in `tests/cli.rs` verifying doctor findings.

- [x] **Stage 3: Compact Invariant Index in `AGENTS.md`**
  - [x] Add concise, high-density ~25-line Invariant Index block at the top of `AGENTS.md`.
  - [x] Verify line budget (<= 30 lines) and markdown formatting.

- [x] **Stage 4: Verification & Governance**
  - [x] Run `cargo fmt --check`.
  - [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Run `cargo test`.
  - [x] Open PR, verify 100% green CI matrix, and merge to `main`.

> STATUS (v1.21.0): Completed & verified. R1 probe shipped with two
> documented deviations from the original spec text: (a) a missing `gh` CLI
> or non-GitHub remote degrades to an info notice instead of a failing
> finding (cannot claim "missing protection" without evidence); (b)
> `required_pull_request_reviews` absence is an advisory info (the shipped
> `protect-branch.sh` intentionally sets it null for single-developer flow),
> while missing `required_status_checks` remains a hard finding per spec.
