# Execution Plan: Guaranteed Turn-0 Drift Delivery for OpenAI Codex CLI

## 1. Goal
Provide guaranteed Turn-0 `RepoState` drift synchronization and compaction resilience for OpenAI Codex CLI by managing `.codex/config.toml` `[[hooks.SessionStart]]` hooks running `ce-ai workflow resume`.

## 2. Work Breakdown

### Work Unit 1: Codex Hook Functions in `src/harness/codex.rs`
- Target: `src/harness/codex.rs` & `src/harness/tests/codex.rs`
- Responsibilities:
  - `CODEX_RESUME_COMMAND = "ce-ai workflow resume"`
  - `has_session_start_hook`
  - `ensure_session_start_hook` (atomic, non-destructive TOML table merge)
  - `remove_session_start_hook` (surgical removal and clean file pruning)
  - Unit tests verifying creation, idempotency, user preservation, and removal.

### Work Unit 2: CLI Subcommand Wiring & Integration Tests
- Target: `src/commands/init_prj.rs`, `src/commands/deinit_prj.rs`, `src/commands/doctor.rs`, `tests/cli.rs`
- Responsibilities:
  - `init_prj`: call `ensure_session_start_hook` when `.codex` exists.
  - `deinit_prj`: call `remove_session_start_hook`.
  - `doctor`: report finding when hook is missing in projects with `.codex`.
  - Integration test verifying end-to-end `init-prj` -> `doctor` -> `deinit-prj` lifecycle for Codex.

### Work Unit 3: Documentation & SemVer Bump
- Target: `docs/user-guide/zero-step-drift-recovery-explained.md`, `Cargo.toml`, `CHANGELOG.md`
- Responsibilities:
  - Update user guide.
  - Bump SemVer to `1.34.0`.
