# Execution Plan: Guaranteed Turn-0 Drift Delivery for Pi Coding Agent

## 1. Goal
Provide guaranteed Turn-0 `RepoState` drift synchronization and context injection for Mario Zechner's Pi coding agent (`pi.dev`) via `.pi/extensions/compound-engineering.ts`.

## 2. Work Breakdown

### Work Unit 1: Pi Extension Helpers in `src/harness/pi.rs`
- Target: `src/harness/pi.rs` & `src/harness/tests/pi.rs`
- Responsibilities:
  - `PI_EXTENSION_FILENAME = "compound-engineering.ts"`
  - `PI_EXTENSION_CONTENT`: TypeScript extension subscribing to `session_start` and `before_agent_start`.
  - `has_session_start_hook`
  - `ensure_session_start_hook`
  - `remove_session_start_hook`
  - Unit tests verifying creation, idempotency, content integrity, and removal.

### Work Unit 2: CLI Subcommand Wiring & Integration Tests
- Target: `src/commands/init_prj.rs`, `src/commands/deinit_prj.rs`, `src/commands/doctor.rs`, `tests/cli.rs`
- Responsibilities:
  - `init_prj`: call `ensure_session_start_hook` when `.pi/` exists.
  - `deinit_prj`: call `remove_session_start_hook`.
  - `doctor`: report finding when extension is missing in projects with `.pi/`.
  - Integration test verifying end-to-end `init-prj` -> `doctor` -> `deinit-prj` lifecycle for Pi.

### Work Unit 3: Documentation & SemVer Bump
- Target: `docs/user-guide/zero-step-drift-recovery-explained.md`, `Cargo.toml`, `CHANGELOG.md`
- Responsibilities:
  - Update user guide.
  - Bump SemVer to `1.35.0`.
