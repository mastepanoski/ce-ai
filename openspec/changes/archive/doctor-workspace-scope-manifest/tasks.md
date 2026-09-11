# Tasks: Workspace-Scoped OpenCode Manifest Resolution

- [x] Unit 1: Add Scope & Target Directory to `state.installed_harnesses` in `install.rs` (~30 LOC)
  - [x] In `src/commands/install.rs:340-350`, add `"scope"` and `"target_dir"` fields to the `installed_harnesses` entry.
  - [x] Verify unit tests pass in `src/commands/tests/install.rs`.

- [x] Unit 2: Implement `Context::resolve_opencode_dir` in `src/commands/mod.rs` (~35 LOC)
  - [x] Add `resolve_opencode_dir(&self, state: &State) -> PathBuf` to `Context` in `src/commands/mod.rs`.
  - [x] Add unit test verifying resolution logic for workspace-scoped vs global-scoped states.

- [x] Unit 3: Update `doctor.rs`, `status.rs`, and `sync.rs` to Use `resolve_opencode_dir` (~65 LOC)
  - [x] In `src/commands/doctor.rs:50-105`, resolve `opencode_dir` from state and use it for manifest loading, drift diffing, and session plugin verification.
  - [x] In `src/commands/status.rs:94-117`, resolve `opencode_dir` for manifest loading and drift diffing.
  - [x] In `src/commands/sync.rs:37-175`, resolve `opencode_dir` for manifest loading and synchronization.

- [x] Unit 4: Add Integration Test in `tests/cli.rs` and Unit Test in `src/commands/tests/doctor.rs` (~70 LOC)
  - [x] In `tests/cli.rs`, add `doctor_workspace_scope_opencode_install_has_no_false_positive_findings` verifying `ce-ai install --harness opencode --scope workspace` followed by `ce-ai doctor` succeeds without findings.
  - [x] In `src/commands/tests/doctor.rs`, add unit test verifying that real missing manifest triggers `state-inconsistent`.
