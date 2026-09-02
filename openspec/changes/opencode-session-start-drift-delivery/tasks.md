# Tasks: OpenCode Session-Start Drift Delivery Implementation

- [x] 1. Create `.opencode/plugins/compound-engineering.js` source file (~100 LOC)
  - [x] Implement skill command discovery + registration.
  - [x] Implement `getRepoState(cwd)` running `ce-ai workflow resume`.
  - [x] Implement `event` listener with `session.created` handling.
  - [x] Implement `experimental.session.compacting` hook.
  - [x] Verify syntax and module loading via Node.js / Bun.

- [x] 2. Update `src/opencode/plugins.rs` with embedded loader and lifecycle helpers (~90 LOC)
  - [x] Embed `BUILTIN_LOADER` using `include_str!`.
  - [x] Update `install_loader()` to use `BUILTIN_LOADER` when source loader lacks `session.created`.
  - [x] Implement `has_session_start_plugin(config_dir: &Path) -> bool`.
  - [x] Implement `ensure_session_start_plugin(config_dir: &Path) -> Result<bool, CeError>`.
  - [x] Implement `remove_session_start_plugin(config_dir: &Path) -> Result<bool, CeError>`.
  - [x] Add unit tests in `src/opencode/tests/plugins.rs`.

- [x] 3. Wire `ce-ai doctor` health diagnostic finding (~30 LOC)
  - [x] In `src/commands/doctor.rs`, audit `opencode` harness when present in `state.installed_harnesses`.
  - [x] Report `opencode: SessionStart plugin missing or outdated` if missing or tampered.
  - [x] Add integration test in `tests/cli.rs`.

- [x] 4. Update documentation (~50 LOC)
  - [x] Update `docs/user-guide/zero-step-drift-recovery-explained.md`.
  - [x] Update `docs/user-guide/harnesses-loops-and-context-masterclass.md`.
  - [x] Bump version in `Cargo.toml` to `1.32.0` and update `CHANGELOG.md`.

- [x] 5. Quality gates & Verification
  - [x] `cargo fmt --check`
  - [x] `cargo clippy --all-targets --all-features -- -D warnings`
  - [x] `cargo test`
  - [x] `make e2e`
