# Execution Plan: Guaranteed Turn-0 Drift Delivery for OpenCode

## 1. Goal
Provide guaranteed Turn-0 `RepoState` drift synchronization for OpenCode sessions by shipping a native OpenCode plugin subscribed to `session.created` and `experimental.session.compacting`, accompanied by health diagnostics in `ce-ai doctor` and embedded fallback reliability.

## 2. Work Breakdown & Phased Execution

### Work Unit 1: Canonical OpenCode Plugin Implementation
- Target: `.opencode/plugins/compound-engineering.js`
- Responsibilities:
  - Dynamic skill loading and command registration (exact backward compatibility with upstream loader).
  - Executing `ce-ai workflow resume` in the session's workspace `directory` via `child_process.spawnSync`.
  - Subscribing to `session.created` and injecting context via `client.session.prompt` with `noReply: true`.
  - Subscribing to `experimental.session.compacting` and appending context to `output.context`.
  - Validating module syntax with Node.js.

### Work Unit 2: Rust Engine Integration & Embedded Loader
- Target: `src/opencode/plugins.rs` & `src/opencode/tests/plugins.rs`
- Responsibilities:
  - `pub const BUILTIN_LOADER: &str = include_str!("../../../.opencode/plugins/compound-engineering.js");`
  - Enhance `install_loader()` to verify presence of `session.created` and fall back to `BUILTIN_LOADER`.
  - Implement `has_session_start_plugin`, `ensure_session_start_plugin`, `remove_session_start_plugin`.
  - Add comprehensive unit tests.

### Work Unit 3: Doctor Health Check & CLI Integration Tests
- Target: `src/commands/doctor.rs` & `tests/cli.rs`
- Responsibilities:
  - Audit `opencode` harness in `doctor::run` when registered in `state.installed_harnesses`.
  - Add CLI integration tests verifying install, tamper detection, repair on sync/re-install, and doctor findings.

### Work Unit 4: Documentation Alignment & Version Bump
- Target: `docs/user-guide/zero-step-drift-recovery-explained.md`, `Cargo.toml`, `CHANGELOG.md`
- Responsibilities:
  - Update guides reflecting OpenCode automated lifecycle delivery.
  - Bump SemVer to `1.32.0`.
