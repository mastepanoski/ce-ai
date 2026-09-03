# Tasks: Guaranteed Turn-0 Drift Delivery for GitHub Copilot CLI

- [x] 1. Implement Copilot hook lifecycle functions in `src/harness/copilot.rs` (~120 LOC)
  - [x] Define `COPILOT_RESUME_COMMAND` (`"ce-ai workflow resume --json"`).
  - [x] Implement `has_session_start_hook(hooks_path: &Path) -> bool`.
  - [x] Implement `ensure_session_start_hook(hooks_path: &Path) -> Result<bool, CeError>`.
  - [x] Implement `remove_session_start_hook(hooks_path: &Path) -> Result<bool, CeError>`.
  - [x] Add unit tests in `src/harness/tests/copilot.rs`.

- [x] 2. Update `src/commands/workflow.rs` JSON resume output (~20 LOC)
  - [x] Add `"additionalContext": additional_context` to `Action::Resume { json: true }` output.
  - [x] Verify test suite and assertion compatibility.

- [x] 3. Wire Copilot hook in `init_prj.rs`, `deinit_prj.rs`, and `doctor.rs` (~40 LOC)
  - [x] In `src/commands/init_prj.rs`, call `ensure_session_start_hook` when `.github` is present.
  - [x] In `src/commands/deinit_prj.rs`, call `remove_session_start_hook`.
  - [x] In `src/commands/doctor.rs`, check `has_session_start_hook` for projects with `.github`.
  - [x] Add integration test in `tests/cli.rs`.

- [x] 4. Documentation & Versioning (~30 LOC)
  - [x] Update `docs/user-guide/zero-step-drift-recovery-explained.md`.
  - [x] Bump version in `Cargo.toml` to `1.33.0` and update `CHANGELOG.md`.

- [x] 5. Quality gates & Verification
  - [x] `cargo fmt --check`
  - [x] `cargo clippy --all-targets --all-features -- -D warnings`
  - [x] `cargo test`
  - [x] `make e2e`
