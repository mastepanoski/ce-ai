# Proposal: Cursor sessionStart Lifecycle Hook Integration

## Problem Statement
When developing in Cursor (both desktop IDE and Cursor CLI v0.45+), agent sessions may experience Turn-0 drift if the LLM overlooks or forgets instructions in `AGENTS.md` or `.cursor/rules/compound-engineering.mdc`. Without an automated, synchronous lifecycle hook, `ce-ai workflow resume` is not invoked deterministically at the start of composer sessions.

## In-Scope Boundaries
- Implement `has_session_start_hook`, `ensure_session_start_hook`, and `remove_session_start_hook` in `src/harness/cursor.rs` managing `<project>/.cursor/hooks.json`.
- Configure `sessionStart` hook executing `ce-ai workflow resume --json`.
- Enhance `ce-ai workflow resume --json` payload with `additional_context` field matching Cursor's expected schema (`{ "additional_context": "..." }`).
- Wire hook installation in `src/commands/init_prj.rs`.
- Wire hook and rules removal in `src/commands/deinit_prj.rs`.
- Add diagnostic health probe in `src/commands/doctor.rs`.
- Add unit tests in `src/harness/tests/cursor.rs` and CLI integration test in `tests/cli.rs`.
- Document zero-step drift recovery in `docs/user-guide/zero-step-drift-recovery-explained.md`.

## Out-of-Scope Boundaries
- Modifying Cursor IDE internal binary or proprietary extensions.
- Overwriting non-managed hooks or user settings in `.cursor/hooks.json`.

## Risk Evaluation & Mitigation
- **Risk:** User config clobbering in `.cursor/hooks.json`.
  - **Mitigation:** Parse full JSON AST, insert only under `hooks.sessionStart`, preserve all other properties, and write atomically using `write_atomic`.
- **Risk:** Stale hook entries on project de-adoption.
  - **Mitigation:** `deinit-prj` surgically strips the managed `sessionStart` entry and deletes the file only if no custom hooks remain.

## Success Criteria
- `ce-ai init-prj` creates `.cursor/hooks.json` containing `sessionStart` hook when `.cursor` exists.
- `ce-ai doctor` verifies the hook and flags missing configuration with a remediation hint.
- `ce-ai deinit-prj` cleans up the managed hook and rule files without touching user configs.
- 100% tests pass (`cargo test`), `make e2e` passes, and CI matrix is green.
