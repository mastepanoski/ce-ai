# Tasks: Auto-configure RTK Hook Injection for Natively-Supported Harnesses

## Work Unit 1: RTK Adapter & Support Matrix (TDD First)
- **Estimated Changed Lines**: ~140 LOC
- [x] Create `src/harness/rtk.rs` defining support matrix, opt-out logic, command execution, and hook inspection
- [x] Register `pub mod rtk;` in `src/harness/mod.rs`
- [x] Create unit tests in `src/harness/tests/rtk.rs` for `is_rtk_supported`, `is_rtk_opted_out` (flags and env vars), and command construction
- [x] Connect unit tests via `#[cfg(test)]` in `src/harness/rtk.rs`
- [x] Run `cargo test --lib harness::rtk` to verify all unit tests pass

## Work Unit 2: CLI Flags & Install/Init-Prj/Uninstall Hook Injection
- **Estimated Changed Lines**: ~110 LOC
- [x] Add `--skip-rtk` and `--skip-companions` to `src/commands/install.rs` `Args`
- [x] Wire `configure_rtk_hook` into `src/commands/install.rs` for supported vs unsupported harnesses
- [x] Add `--skip-rtk` and `--skip-companions` to `src/commands/init_prj.rs` `Args`
- [x] Wire RTK hook reconciliation into `src/commands/init_prj.rs` for detected supported harnesses
- [x] Wire `unconfigure_rtk_hook` into `src/commands/uninstall.rs`
- [x] Add unit tests in `src/commands/tests/install.rs` and `src/commands/tests/init_prj.rs` verifying flag propagation

## Work Unit 3: Audit Severity Escalation & Doctor Diagnostics
- **Estimated Changed Lines**: ~90 LOC
- [x] Update `CliCompressionDetector` in `src/commands/audit.rs` to escalate missing RTK on supported harnesses from `Info` to `Warn`
- [x] Keep `Info` severity for unsupported harnesses in `src/commands/audit.rs`
- [x] Add RTK hook presence check and stdout filter limitation disclosure to `src/commands/doctor.rs`
- [x] Update unit tests in `src/commands/tests/audit.rs` and `src/commands/tests/doctor.rs`

## Work Unit 4: CLI Integration Tests
- **Estimated Changed Lines**: ~120 LOC
- [x] Add integration test in `tests/cli.rs` verifying RTK hook injection on install of supported harness
- [x] Add integration test verifying `--skip-rtk` and `CE_AI_SKIP_RTK=1` opt-out
- [x] Add integration test verifying unsupported harnesses execute as clean no-op
- [x] Add integration test verifying `audit` reports `Warn` for missing RTK on supported harnesses

## Work Unit 5: Versioning, Changelog & Quality Gates
- **Estimated Changed Lines**: ~35 LOC
- [x] Bump version to `1.42.0` in `Cargo.toml` and update `Cargo.lock`
- [x] Document all additions and the silent output filter limitation in `CHANGELOG.md`
- [x] Verify `README.md` length <= 100 lines
- [x] Run `cargo fmt --check`
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] Run `cargo test`
- [x] Run `make e2e`
