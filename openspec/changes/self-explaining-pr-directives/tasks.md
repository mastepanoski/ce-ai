# Tasks: Self-Explaining PR Directives & Upfront Review Readiness

## Estimated Scope: ~250 LOC across 3 atomic work units (~80–90 LOC per work unit)

- [x] **Work Unit 1: Adoption Block Templates & BLOCK_VERSION Bump** (est. ~90 LOC)
  - [x] Update `render_block_content` in `src/commands/init_prj.rs` across `Full`, `Minimal`, and `Orchestrator` tiers with self-explaining PR and review readiness directives.
  - [x] Bump `pub const BLOCK_VERSION: u32 = 6;` in `src/commands/init_prj.rs`.
  - [x] Update adoption block unit tests in `src/commands/init_prj.rs`.
  - [x] TDD Verification: `cargo test init_prj`

- [x] **Work Unit 2: CLI Test Coordination & Drift Classification** (est. ~80 LOC)
  - [x] Update `CUR_BLOCK_VERSION: u32 = 6` in `tests/cli.rs`.
  - [x] Coordinate pinned test fixtures, stale version expectations, and SHA checks in `tests/cli.rs`.
  - [x] Verify `check_adoption_block_status` correctly marks v5 as `StaleVersion` and v6 as `Ok`.
  - [x] TDD Verification: `cargo test --test cli test_cli_adopt_project_`

- [x] **Work Unit 3: Governance Documentation, Version Bump & Gates** (est. ~80 LOC)
  - [x] Update root `AGENTS.md` with Stage 7 Self-Explaining PR directives and refreshed v6 managed block.
  - [x] Update `CONTRIBUTING.md` clarifying collapsible evidence blocks vs. strict 400 LOC review boundary.
  - [x] Refresh `GEMINI.md` managed block.
  - [x] Bump version to `1.69.0` in `Cargo.toml` and document changes in `CHANGELOG.md`.
  - [x] Run full project verification gates: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
