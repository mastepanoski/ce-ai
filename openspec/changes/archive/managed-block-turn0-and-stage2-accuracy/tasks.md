# Tasks: Managed Adoption Block Turn-0 Directives & Progressive OpenSpec Accuracy

- [x] **Work Unit 1: Adoption Block Template Updates & BLOCK_VERSION Bump** (est. ~50 LOC)
  - [x] Replace Turn-0 unconditional directive in `render_block_content(AdoptionTier::Full)` in `src/commands/init_prj.rs` with hook-aware conditional text.
  - [x] Replace Stage 2 5-file requirement in `render_block_content(AdoptionTier::Full)` in `src/commands/init_prj.rs` with progressive OpenSpec definition.
  - [x] Bump `pub const BLOCK_VERSION: u32 = 7;` in `src/commands/init_prj.rs`.
  - [x] Add unit test `test_render_block_content_turn_0_and_progressive_openspec` in `src/commands/tests/init_prj.rs`.
  - [x] Verify unit tests pass: `cargo test --bin ce-ai commands::init_prj::tests`.

- [x] **Work Unit 2: Test Suite Synchronization & Stale v6 Upgrade Test** (est. ~60 LOC)
  - [x] Update `const CUR_BLOCK_VERSION: u32 = 7;` in `tests/cli.rs`.
  - [x] Replace hardcoded `6` assertion in `init_prj_upgrades_stale_v5_block_to_v6_with_self_explaining_directives` with `CUR_BLOCK_VERSION`.
  - [x] Add new integration test `init_prj_upgrades_stale_v6_block_to_v7_with_turn0_and_progressive_openspec` in `tests/cli.rs`.
  - [x] Verify full test suite passes: `cargo test`.

- [x] **Work Unit 3: Versioning, CHANGELOG & Quality Gates** (est. ~40 LOC)
  - [x] Bump SemVer to `1.72.1` in `Cargo.toml`.
  - [x] Add entry in `CHANGELOG.md` under `[1.72.1] - 2026-09-26`.
  - [x] Run formatting and linter checks (`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`).
  - [x] Run containerized E2E gate (`make e2e`).
  - [x] Run documentation hygiene check (`cargo run -- doc lint --strict` and `python3 scripts/validate-concepts.py`).
