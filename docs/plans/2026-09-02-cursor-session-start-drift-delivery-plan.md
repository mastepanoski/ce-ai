# Implementation Plan: Cursor sessionStart Drift Delivery

## Phase 1: Workflow Resume JSON Output
- Add `"additional_context": additional_context` in `src/commands/workflow.rs`.

## Phase 2: Cursor Harness Helpers
- Implement `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` in `src/harness/cursor.rs`.
- Add unit tests in `src/harness/tests/cursor.rs`.

## Phase 3: CLI Wiring
- Wire in `src/commands/init_prj.rs`.
- Wire in `src/commands/deinit_prj.rs`.
- Wire in `src/commands/doctor.rs`.
- Add integration test in `tests/cli.rs`.

## Phase 4: Documentation & Release
- Update `docs/user-guide/zero-step-drift-recovery-explained.md`.
- Capture solution in `docs/solutions/architecture/`.
- Bump version to `1.36.0` and update `CHANGELOG.md`.
