# Tasks: Inferred-Stage Monotonicity Guard Hardening

## Work Unit 1: State Invariant Hardening & Unit Tests
- **Estimated Changed Lines**: ~40 LOC
- [x] Implement Inferred non-regression guard in `State::validate_and_set_workflow_for_branch` (`src/state/state.rs`)
- [x] Add unit test `inferred_checkpoint_cannot_regress_previous_inferred_checkpoint` in `src/state/tests/state.rs`
- [x] Run `cargo test --bin ce-ai state::tests` to verify TDD pass

## Work Unit 2: Versioning, Changelog & Quality Gates
- **Estimated Changed Lines**: ~25 LOC
- [x] Bump version to `1.40.1` in `Cargo.toml` and update `Cargo.lock`
- [x] Add entry for `1.40.1` in `CHANGELOG.md`
- [x] Run `cargo fmt --check`
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] Run `cargo test` (all 411+ tests)
- [x] Run `make e2e` (containerized Docker E2E gate)

