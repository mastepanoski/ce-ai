# Tasks: Post-Merge OpenSpec Archival & Turn-0 Prescriptions

## Estimated Scope: ~380 LOC across 4 atomic work units (~90–100 LOC per work unit)

- [x] **Work Unit 1: Governance Directives & Adoption Block Update** (est. ~90 LOC)
  - [x] Update Hard Invariant #10 in `AGENTS.md` with post-merge `workflow status` verification and automated `ce-ai archive` requirement.
  - [x] Update `render_block_content` in `src/commands/init_prj.rs` to include post-merge archival guidance in adoption blocks.
  - [x] Update existing adoption block unit tests in `src/commands/tests/init_prj.rs` or `init_prj.rs`.
  - [x] TDD Verification: `cargo test init_prj`

- [x] **Work Unit 2: Turn-0 `ce-ai workflow resume` Prescriptive Guidance** (est. ~100 LOC)
  - [x] Extend `handle_resume` in `src/commands/workflow.rs` to scan for completed unarchived OpenSpec packages.
  - [x] Emit `! Action Required: OpenSpec change '{change}' is complete ({completed}/{total} tasks). Run 'ce-ai archive {change}' to seal the change package.`
  - [x] Add unit test verifying that `resume` surfaces the action directive when completed changes exist.
  - [x] TDD Verification: `cargo test workflow::tests::test_resume`

- [x] **Work Unit 3: Actionable Doctor & Status Diagnostics** (est. ~90 LOC)
  - [x] Update `ce-ai doctor` in `src/commands/doctor.rs` to replace `see openspec/changes/archive/README.md` with `run 'ce-ai archive <feature>'`.
  - [x] Update `ce-ai workflow status` in `src/commands/workflow.rs` to output `run 'ce-ai archive <feature>'`.
  - [x] Update corresponding assertions in existing doctor and status unit/CLI tests.
  - [x] TDD Verification: `cargo test doctor`

- [x] **Work Unit 4: CLI Integration Tests & Quality Gates** (est. ~100 LOC)
  - [x] Add CLI integration test in `tests/cli.rs` validating `workflow resume` and `doctor` output on completed unarchived changes.
  - [x] Run full project quality gates (`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`).
  - [x] Bump SemVer to `v1.67.0` in `Cargo.toml` and update `CHANGELOG.md`.
  - [x] TDD Verification: `cargo test --test cli test_cli_resume_completed_openspec_prescription`
