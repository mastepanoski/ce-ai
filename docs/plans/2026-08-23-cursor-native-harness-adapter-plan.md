# Implementation Plan: Cursor Native Harness Adapter (Issue #173)

- **Date**: 2026-08-23
- **Status**: Ready
- **Target Release**: v1.9.0
- **Feature**: Native Cursor Adapter for `ce-ai`

---

## 1. Architectural Changes

1. **`src/harness/mod.rs` & `src/harness/cursor.rs`**:
   - Update `CursorAdapter::default_config_path` in `src/harness/cursor.rs` to return `home.join(".cursor").join("mcp.json")` (replacing deprecated `.cursorrules`).
   - Implement `CursorMcpConfig`, `CursorMcpServer`, and `CursorRuleFrontmatter` structs for parsing and serializing `~/.cursor/mcp.json`.
   - Implement `register_cursor_mcp_server` and `unregister_cursor_mcp_server` with structured JSON merge & atomic write.
   - Update rule block formatting to produce `.cursor/rules/compound-engineering.mdc` with valid frontmatter.

2. **`src/commands/install.rs` & `src/commands/tools.rs`**:
   - Dispatch `HarnessKind::Cursor` to `cursor::install_cursor_harness` or format `mcpServers` correctly.
   - Ensure `ce-ai tools install <tool> --harness cursor` updates `mcpServers` in `~/.cursor/mcp.json`.

3. **`src/commands/init_prj.rs` & `src/commands/sync.rs`**:
   - Update `init_prj` to write `.cursor/rules/compound-engineering.mdc` when `--harness cursor` is targeted.
   - Update `sync` to inspect and repair `~/.cursor/mcp.json` `mcpServers` drift.

4. **`src/commands/uninstall.rs`**:
   - Dispatch `HarnessKind::Cursor` to strip `ce-ai` managed servers from `mcp.json` or restore backups via `crate::state::backups`.

5. **`tests/cli.rs`**:
   - Add integration tests verifying native `mcpServers` schema in `~/.cursor/mcp.json`, zero OpenCode keys (`plugin`, `skills.paths`), `.cursor/rules/*.mdc` creation, and clean uninstall.

---

## 2. Execution Phases

- **Phase 1**: OpenSpec Contract Definition (`openspec/changes/cursor-native-harness-adapter/`)
- **Phase 2**: OpenSpec Review (`ce-doc-review`)
- **Phase 3**: TDD & Implementation (`src/harness/cursor.rs`, `src/commands/install.rs`, `src/commands/uninstall.rs`, `src/commands/tools.rs`)
- **Phase 4**: Verification (`cargo fmt`, `cargo clippy`, `cargo test`, `make e2e`)
- **Phase 5**: Code Review & Security Audit (`ce-code-review`)
- **Phase 6**: Knowledge Capture (`ce-compound`)
- **Phase 7**: Release Shipping (SemVer `1.9.0`, `CHANGELOG.md`, PR, Release tag `v1.9.0`)
