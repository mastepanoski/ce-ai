# Tasks: Automated Background Update Notifier for ce-ai CLI & Harness Releases

## Work Unit 1: State Configuration & Cache Data Model
- [x] Add `UpdateNotifierConfig` struct to `src/state/state.rs` with `enabled` (default `true`) and `interval_hours` (default `24`). (~40 LOC)
- [x] Add `pub update_notifier: Option<UpdateNotifierConfig>` field to `State` in `src/state/state.rs`. (~10 LOC)
- [x] Unit tests for `UpdateNotifierConfig` serialization/deserialization. (~20 LOC)
*Estimated changed lines: ~70 LOC*

## Work Unit 2: Core Update Notifier Module & Unit Tests
- [x] Create `src/source/update_notifier.rs` implementing `UpdateCheckCache`, `read_cache`, `is_cache_stale`, `format_update_banner`, `should_check_updates`. (~110 LOC)
- [x] Implement `check_and_update_cache_sync` and `spawn_background_check` using `write_atomic` and fail-closed error handling. (~60 LOC)
- [x] Add comprehensive unit tests in `src/source/tests/update_notifier.rs` covering caching, expiration arithmetic, banner rendering, and suppression conditions. (~90 LOC)
*Estimated changed lines: ~260 LOC*

## Work Unit 3: CLI Runtime Integration (`main.rs`, `doctor.rs`, `self_update.rs`)
- [x] Integrate background check trigger and exit banner hook into `src/main.rs`. (~35 LOC)
- [x] Update `src/commands/doctor.rs` to inspect update notifier status and report `doctor-info: update-notifier: ...`. (~35 LOC)
- [x] Update `src/commands/self_update.rs` to refresh `<config_dir>/cache/update_check.json` during explicit checks. (~25 LOC)
*Estimated changed lines: ~95 LOC*

## Work Unit 4: CLI End-to-End Integration Tests & Verification
- [x] Add CLI integration tests in `tests/cli.rs` verifying banner appearance on stderr, silence on stdout, suppression via `CE_NO_UPDATE_NOTIFIER=1` and `CI=true`. (~90 LOC)
- [x] Verify `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`. (~20 LOC)
*Estimated changed lines: ~110 LOC*

*Total PR forecast: ~535 LOC*
