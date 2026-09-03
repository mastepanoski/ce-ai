# Tasks: Guaranteed Turn-0 Drift Delivery for Pi Coding Agent

- [ ] 1. Implement Pi extension lifecycle helpers in `src/harness/pi.rs` (~70 LOC)
  - [ ] Define `PI_EXTENSION_FILENAME` and `PI_EXTENSION_CONTENT`.
  - [ ] Implement `has_session_start_hook(extension_path: &Path) -> bool`.
  - [ ] Implement `ensure_session_start_hook(extension_path: &Path) -> Result<bool, CeError>`.
  - [ ] Implement `remove_session_start_hook(extension_path: &Path) -> Result<bool, CeError>`.
  - [ ] Add unit tests in `src/harness/tests/pi.rs`.

- [ ] 2. Wire Pi extension in `init_prj.rs`, `deinit_prj.rs`, and `doctor.rs` (~35 LOC)
  - [ ] In `src/commands/init_prj.rs`, call `ensure_session_start_hook` when `.pi` is present.
  - [ ] In `src/commands/deinit_prj.rs`, call `remove_session_start_hook`.
  - [ ] In `src/commands/doctor.rs`, check `has_session_start_hook` for projects with `.pi`.
  - [ ] Add integration test in `tests/cli.rs`.

- [ ] 3. Documentation & Versioning (~25 LOC)
  - [ ] Update `docs/user-guide/zero-step-drift-recovery-explained.md`.
  - [ ] Bump version in `Cargo.toml` to `1.35.0` and update `CHANGELOG.md`.

- [ ] 4. Quality gates & Verification
  - [ ] `cargo fmt --check`
  - [ ] `cargo clippy --all-targets --all-features -- -D warnings`
  - [ ] `cargo test`
  - [ ] `make e2e`
