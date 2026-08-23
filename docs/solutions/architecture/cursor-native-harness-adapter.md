---
module: src/harness/cursor.rs
tags: [harness, cursor, mcp, mdc, adapter]
problem_type: architecture
---

# Cursor Native Harness Adapter Implementation

## What
Implemented native Cursor harness support in `ce-ai`, replacing synthetic OpenCode JSON schema outputs with Cursor's official `mcpServers` stdio schema in `~/.cursor/mcp.json` and `.cursor/rules/compound-engineering.mdc` project rules format with YAML frontmatter.

## Why
Previously, non-OpenCode harnesses received synthetic copies of OpenCode's JSON schema (`plugin`, `skills.paths`), which host harnesses like Cursor ignored completely. Cursor requires `mcpServers` stdio configurations for MCP servers and `.cursor/rules/*.mdc` for project directives.

## Where
- `src/harness/cursor.rs`: Native `CursorMcpConfig`, `CursorMcpServer`, `CursorRuleFrontmatter` schemas, and atomic reader/writer functions.
- `src/harness/mod.rs`: Updated `CursorAdapter::default_config_path` to `home.join(".cursor").join("mcp.json")`.
- `src/commands/install.rs`: Registered Cursor sidecar MCP servers (`codegraph`, `engram`) natively.
- `src/commands/tools.rs`: Added Cursor `mcp.json` registration on `ce-ai tools install <tool>`.
- `src/commands/init_prj.rs`: Added `.cursor/rules/compound-engineering.mdc` creation with frontmatter on project adoption.
- `src/commands/sync.rs`: Reconciled Cursor `mcpServers` drift.
- `src/commands/uninstall.rs`: Restored user backups or removed `ce-ai` sidecar servers from `mcp.json`, preserving user-defined servers.
- `src/state/backups.rs`: Added harness path resolution to tag Cursor backups accurately.

## Learned & Gotchas
1. **Per-Server Attribute Preservation**: Cursor `mcpServers` entries can contain arbitrary custom fields (`disabled`, `timeout`, `autoApprove`, `url` for SSE). Using `#[serde(flatten)] pub extra: serde_json::Map<String, serde_json::Value>` on both root config and individual server entries is mandatory to prevent stripping user attributes.
2. **Legacy Rule Files**: Cursor officially deprecated `.cursorrules` in favor of Project Rules (`.cursor/rules/*.mdc`).
3. **Clean Uninstall**: `uninstall` should only delete `mcp.json` if `ce-ai` created it and no user-defined MCP servers remain. If pre-existing user tools exist, only `ce-ai` managed servers should be removed.
