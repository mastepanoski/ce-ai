# Execution Plan: Guaranteed Turn-0 Drift Delivery for GitHub Copilot CLI

## 1. Goal
Provide guaranteed Turn-0 `RepoState` drift synchronization for GitHub Copilot CLI by managing `.github/hooks/hooks.json` `sessionStart` hooks and enhancing `ce-ai workflow resume --json` with `additionalContext`.

## 2. Work Breakdown

### Work Unit 1: Copilot Hook Functions in `src/harness/copilot.rs`
- Target: `src/harness/copilot.rs` & `src/harness/tests/copilot.rs`
- Responsibilities:
  - `COPILOT_RESUME_COMMAND = "ce-ai workflow resume --json"`
  - `has_session_start_hook`
  - `ensure_session_start_hook` (atomic, non-destructive JSON merge)
  - `remove_session_start_hook` (surgical removal and clean pruning)
  - Unit tests verifying creation, idempotency, user preservation, and removal.

### Work Unit 2: Workflow JSON Output Enhancement
- Target: `src/commands/workflow.rs`
- Responsibilities:
  - Add `"additionalContext"` to `workflow resume --json` payload.
  - Verify existing CLI tests still pass.

### Work Unit 3: CLI Subcommand Wiring & Integration Tests
- Target: `src/commands/init_prj.rs`, `src/commands/deinit_prj.rs`, `src/commands/doctor.rs`, `tests/cli.rs`
- Responsibilities:
  - `init_prj`: call `ensure_session_start_hook`.
  - `deinit_prj`: call `remove_session_start_hook`.
  - `doctor`: report finding when hook is missing in projects with `.github`.
  - Integration test verifying end-to-end `init-prj` -> `doctor` -> `deinit-prj` lifecycle for Copilot.

### Work Unit 4: Documentation & SemVer Bump
- Target: `docs/user-guide/zero-step-drift-recovery-explained.md`, `Cargo.toml`, `CHANGELOG.md`
- Responsibilities:
  - Update user guide.
  - Bump SemVer to `1.33.0`.
