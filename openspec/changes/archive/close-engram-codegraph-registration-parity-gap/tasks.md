# Tasks: Close engram/codegraph registration parity gap

## Work Unit 1: Registration Characterization Tests (TDD First)
- **Estimated Changed Lines**: ~90 LOC
- [x] Create `src/harness/tests/registration.rs` with characterization tests for `registration_spec` and `RegistrationSpec`
- [x] Connect tests to `src/harness/registration.rs` via `#[cfg(test)]`
- [x] Run `cargo test --lib harness::registration` to verify baseline pass

## Work Unit 2: OpenCode Companion Auto-Registration
- **Estimated Changed Lines**: ~60 LOC
- [x] Add `crate::opencode::config::register_companions` in `src/opencode/config.rs`
- [x] Invoke `register_companions` in `src/commands/install.rs` for `HarnessKind::Opencode`
- [x] Invoke `register_companions` in `src/commands/sync.rs` for `HarnessKind::Opencode`
- [x] Update `find_mcp_config_paths` in `src/source/tools_registry.rs` to include `opencode.json`
- [x] Add unit test verifying OpenCode companions auto-registration in `src/opencode/tests/config.rs`

## Work Unit 3: Custom Harness MCP File Support
- **Estimated Changed Lines**: ~95 LOC
- [x] Extend `CustomHarnessConfig` and `CustomConfigFlags` in `src/harness/custom.rs` with `mcp_file: Option<PathBuf>`
- [x] Add `--mcp-file` CLI flag in `src/commands/install.rs` and `src/commands/uninstall.rs`
- [x] Implement `register_custom_mcp_server` in `src/harness/custom.rs`
- [x] Wire companion registration in `install.rs` and `sync.rs` for `HarnessKind::Custom`
- [x] Add unit tests in `src/harness/tests/custom.rs` for custom MCP registration

## Work Unit 4: Pi & Deepseek Parity & Documentation
- **Estimated Changed Lines**: ~35 LOC
- [x] Document Deepseek preview rationale and Pi No-MCP CLI delivery contract in `src/harness/registration.rs`
- [x] Verify Pi companion integration diagnostics in `src/commands/tools.rs` and `src/commands/doctor.rs`
- [x] Add CLI integration test in `tests/cli.rs` verifying companion registration across OpenCode, Custom, and Pi

## Work Unit 5: Versioning, Changelog & Quality Gates
- **Estimated Changed Lines**: ~25 LOC
- [x] Bump version to `1.41.0` in `Cargo.toml` and update `Cargo.lock`
- [x] Document additions in `CHANGELOG.md`
- [x] Verify `README.md` length <= 100 lines
- [x] Run `cargo fmt --check`
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] Run `cargo test`
- [x] Run `make e2e`
