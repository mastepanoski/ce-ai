# Tasks: MCP-Configured Companion Tools & Skill Suggestions Detection

- [x] Unit 1: Add Shared MCP & Skill Detection Helpers in `src/source/tools_registry.rs` (~80 LOC)
  - [x] Implement `find_mcp_config_paths(ctx: &Context) -> Vec<PathBuf>` collecting candidate harness configs.
  - [x] Implement `is_mcp_server_configured(ctx: &Context, name: &str) -> bool` with key & definition matching.
  - [x] Implement `is_skill_configured(ctx: &Context, name: &str) -> bool` checking MCP configs and skill registries.
  - [x] Implement `detect_tool_freshness(ctx: &Context, tool_name: &str, info: &CompanionToolInfo) -> FreshnessStatus`.
  - [x] Add unit tests in `src/source/tests/` verifying detection logic.

- [x] Unit 2: Update `doctor.rs` and `tools.rs` to Use Shared Detection (~30 LOC)
  - [x] In `src/commands/doctor.rs:125-167`, use `detect_tool_freshness` and filter `registry.skills` via `is_skill_configured`.
  - [x] In `src/commands/tools.rs:38-74`, use `detect_tool_freshness` and filter `registry.skills` via `is_skill_configured`.

- [x] Unit 3: Add CLI Integration Tests in `tests/cli.rs` (~60 LOC)
  - [x] Add `doctor_detects_mcp_configured_context7_and_sequential_thinking` verifying that when `context7` and `sequential-thinking` are present in `opencode.json` under `mcpServers`, doctor outputs `context7 ... (ok)` and no `skill-suggestion: sequential-thinking` lines.
  - [x] Verify existing tests continue to pass when unconfigured.

- [x] Unit 4: Quality Gates and Release Preparation (~10 LOC)
  - [x] Bump version to `1.38.2` in `Cargo.toml`.
  - [x] Update `CHANGELOG.md` with 1.38.2 release notes.
  - [x] Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.
