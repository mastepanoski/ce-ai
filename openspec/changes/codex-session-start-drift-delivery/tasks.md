# Tasks: Guaranteed Turn-0 Drift Delivery for OpenAI Codex CLI

- [ ] 1. Implement Codex hook lifecycle functions in `src/harness/codex.rs` (~120 LOC)
  - [ ] Define `CODEX_RESUME_COMMAND` (`"ce-ai workflow resume"`).
  - [ ] Implement `has_session_start_hook(config_path: &Path) -> bool`.
  - [ ] Implement `ensure_session_start_hook(config_path: &Path) -> Result<bool, CeError>`.
  - [ ] Implement `remove_session_start_hook(config_path: &Path) -> Result<bool, CeError>`.
  - [ ] Add unit tests in `src/harness/tests/codex.rs`.

- [ ] 2. Wire Codex hook in `init_prj.rs`, `deinit_prj.rs`, and `doctor.rs` (~40 LOC)
  - [ ] In `src/commands/init_prj.rs`, call `ensure_session_start_hook` when `.codex` is present.
  - [ ] In `src/commands/deinit_prj.rs`, call `remove_session_start_hook`.
  - [ ] In `src/commands/doctor.rs`, check `has_session_start_hook` for projects with `.codex`.
  - [ ] Add integration test in `tests/cli.rs`.

- [ ] 3. Documentation & Versioning (~30 LOC)
  - [ ] Update `docs/user-guide/zero-step-drift-recovery-explained.md`.
  - [ ] Bump version in `Cargo.toml` to `1.34.0` and update `CHANGELOG.md`.

- [ ] 4. Quality gates & Verification
  - [ ] `cargo fmt --check`
  - [ ] `cargo clippy --all-targets --all-features -- -D warnings`
  - [ ] `cargo test`
  - [ ] `make e2e`
