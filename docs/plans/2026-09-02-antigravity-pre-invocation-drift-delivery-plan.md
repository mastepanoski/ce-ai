# Plan: Antigravity PreInvocation Turn-0 Drift Delivery

## Phase 1: CLI Flag & Handler
- Add `--pre-invocation` flag to `Action::Resume` in `src/commands/workflow.rs`.
- Implement `handle_pre_invocation` reading `stdin`, checking session marker, and returning `injectSteps` with `ephemeralMessage`.

## Phase 2: Antigravity Hook Helpers
- Implement `has_pre_invocation_hook`, `ensure_pre_invocation_hook`, `remove_pre_invocation_hook` in `src/harness/agy.rs`.
- Add unit tests in `src/harness/tests/agy.rs`.

## Phase 3: Integration & Testing
- Wire in `src/commands/init_prj.rs`.
- Wire in `src/commands/deinit_prj.rs`.
- Wire in `src/commands/doctor.rs`.
- Add CLI integration test in `tests/cli.rs`.

## Phase 4: Documentation & Release
- Update `docs/user-guide/zero-step-drift-recovery-explained.md`.
- Capture solution in `docs/solutions/architecture/`.
- Bump version to `1.37.0` and update `CHANGELOG.md`.
