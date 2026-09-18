# Tasks: Fix Companion MCP Server Invocation Arguments for CodeGraph and Engram

- [x] **Work Unit 1: Update Core Registration Engines and Tools Spec** (~40 LOC)
  - [x] Update `src/harness/registration.rs` to register `codegraph` with `&["serve", "--mcp"]` and `engram` with `&["mcp", "--tools=agent"]`.
  - [x] Update `src/opencode/config.rs` to register `codegraph` with `["serve", "--mcp"]` and `engram` with `["mcp", "--tools=agent"]`.
  - [x] Update `src/harness/custom.rs` to register `codegraph` with `&["serve", "--mcp"]` and `engram` with `&["mcp", "--tools=agent"]`.
  - [x] Update `src/commands/tools.rs` to map `codegraph` to `&["serve", "--mcp"]` and `engram` to `&["mcp", "--tools=agent"]`.
  - [x] Verification: `cargo check`.

- [x] **Work Unit 2: Update Harness and Adapter Unit Tests** (~120 LOC)
  - [x] Update `src/harness/tests/registration.rs` fixtures and assertions.
  - [x] Update `src/opencode/tests/config.rs` fixtures and assertions.
  - [x] Update `src/harness/tests/custom.rs` fixtures and assertions.
  - [x] Update `src/harness/tests/claude.rs` fixtures and assertions.
  - [x] Update `src/harness/tests/cursor.rs` fixtures and assertions.
  - [x] Update `src/harness/tests/copilot.rs` fixtures and assertions.
  - [x] Update `src/harness/tests/codex.rs` fixtures and assertions.
  - [x] Update `src/harness/tests/grok.rs` fixtures and assertions.
  - [x] Update `src/harness/tests/kimi.rs` fixtures and assertions.
  - [x] Update `src/harness/tests/agy.rs` fixtures and assertions.
  - [x] Update `src/harness/tests/fx.rs` fixtures and assertions.
  - [x] Verification: `cargo test`.

- [x] **Work Unit 3: Quality Gates, Versioning & CHANGELOG** (~25 LOC)
  - [x] Run `cargo fmt --check`.
  - [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] Run `cargo test`.
  - [x] Bump patch version in `Cargo.toml` (`1.57.1`).
  - [x] Update `CHANGELOG.md` documenting the fix.
  - [x] Verification: `cargo test`.
