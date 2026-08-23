# Proposal: Cursor Native Harness Adapter

- **Change Name**: `cursor-native-harness-adapter`
- **Issue Reference**: #173 (Umbrella #155)
- **Target Version**: 1.9.0

## 1. Executive Summary

Implement native support for the Cursor AI harness in `ce-ai`. Rather than writing OpenCode JSON schemas into `~/.cursor/mcp.json`, `ce-ai` will parse and mutate Cursor's native `mcpServers` stdio schema, support `.cursor/rules/*.mdc` rules formatting, and perform byte-for-byte user content preservation during uninstall.

## 2. In-Scope & Boundaries

- Native `mcpServers` schema reader/writer for `~/.cursor/mcp.json`.
- `.cursor/rules/compound-engineering.mdc` rules formatting with frontmatter.
- Integration into `ce-ai install`, `ce-ai uninstall`, `ce-ai sync`, `ce-ai tools install --harness cursor`.
- Zero OpenCode keys (`plugin`, `skills.paths`) written to `~/.cursor/mcp.json`.

## 3. Risks & Mitigations

- **Risk**: Overwriting existing user MCP servers in `~/.cursor/mcp.json`.
  - *Mitigation*: Perform structured JSON deserialization and key-level merge; only touch keys managed by `ce-ai`.
- **Risk**: Path resolution drift between macOS, Linux, and Windows.
  - *Mitigation*: Use `HarnessKind::Cursor.harness_dir(home_dir)` mapping to `home_dir.join(".cursor")`.
