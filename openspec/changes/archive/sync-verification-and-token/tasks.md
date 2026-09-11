# Tasks: Fix `ce-ai upgrade` Verification Drift Error and GitHub Token Discovery

Work units carry per-unit changed-line estimates (~200 LOC target) so the PR-level forecast is derivable (CONTRIBUTING.md §4). Total forecast: ~80 lines.

- [x] **Task 1: GitHub Token Multi-Tiered Discovery Implementation & Unit Tests** (~35 LOC)
  - [x] Update `github_token_from_env()` in `src/source/release.rs` to check `CE_AI_GITHUB_TOKEN`, `GITHUB_TOKEN`, `GH_TOKEN`, and `gh auth token`.
  - [x] Add unit test in `src/source/tests/release.rs` testing `GITHUB_TOKEN` and `GH_TOKEN` resolution.
  - [x] Verification: `cargo test source::tests::release`

- [x] **Task 2: Native Harness Sync Verification Matrix Correction & Integration Tests** (~35 LOC)
  - [x] In `src/commands/sync.rs`, replace obsolete `verify_tree_against` check for native harnesses with `CheckStatus::NotVerified { reason: REASON_NO_MANAGED_SKILLS }`.
  - [x] Add integration test in `tests/cli.rs` verifying that `sync` succeeds with exit code 0 and reports `registered` when native harnesses (e.g. `claude`, `copilot`) exist on the host but are unadopted.
  - [x] Verification: `cargo test --test cli`

- [x] **Task 3: Release Version Bump & Changelog Documentation** (~10 LOC)
  - [x] Bump version from `1.29.0` to `1.29.1` in `Cargo.toml`.
  - [x] Update `CHANGELOG.md` with fix details following Keep a Changelog.
  - [x] Verification: `cargo check` and `cargo test`

- [x] **Task 4: Full Quality Gates & Empirical Verification** (~0 LOC)
  - [x] `cargo fmt --check`
  - [x] `cargo clippy --all-targets --all-features -- -D warnings`
  - [x] `cargo test`
  - [x] Real execution test: `ce-ai upgrade` on current environment
