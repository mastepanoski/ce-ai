# Tasks: Condition Doctor Git-Hooks Probe on Project Adoption of .githooks Convention

- [x] Unit 1: Condition Git-Hooks Probe on `.githooks` Directory Presence in `doctor.rs` (~25 LOC)
  - [x] In `src/commands/doctor.rs:297-333`, check `let uses_githooks_convention = root_path.join(".githooks").exists()`.
  - [x] If `points_to_githooks`, check `.githooks/pre-commit` exists.
  - [x] If not `points_to_githooks` but `uses_githooks_convention`, record drift finding.
  - [x] If not `points_to_githooks` and not `uses_githooks_convention`, emit informational notice skipping the check.

- [x] Unit 2: Update Existing Git-Hooks Drift CLI Integration Test in `tests/cli.rs` (~10 LOC)
  - [x] In `tests/cli.rs:doctor_reports_git_hooks_misconfigured_finding`, ensure `.githooks` directory is created so genuine drift is validated.

- [x] Unit 3: Add Integration Test for Non-Adopted Repositories in `tests/cli.rs` (~35 LOC)
  - [x] In `tests/cli.rs`, add `doctor_ignores_non_githooks_hooks_path_when_not_adopted` verifying that Husky-style hooks (`.husky/_`) without `.githooks/` directory are ignored.

- [x] Unit 4: Verification and Quality Gates (~0 LOC)
  - [x] Run `cargo fmt --check`.
  - [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Run `cargo test`.
