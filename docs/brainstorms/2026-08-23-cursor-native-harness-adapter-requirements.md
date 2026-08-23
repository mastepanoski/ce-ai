# Brainstorm: Cursor Native Harness Adapter Requirements (Issue #173)

- **Date**: 2026-08-23
- **Status**: Complete
- **Feature**: Real Cursor Native Harness Adapter Implementation
- **Issue Reference**: #173 (Umbrella #155)

---

## 1. Problem Statement

`ce-ai` currently advertises support for 12 harnesses, but non-OpenCode harnesses (including `cursor`) receive a synthetic copy of OpenCode's JSON schema (`plugin`, `skills.paths`) written into host paths, and uninstall leaves synthetic files behind.

For Cursor specifically:
- `~/.cursor/mcp.json` receives OpenCode JSON schema keys (`plugin`, `skills.paths`) which Cursor's MCP parser ignores.
- Cursor requires `{"mcpServers": {"<name>": {"type": "stdio", "command": "...", "args": [...]}}}`.
- Instruction rules targeting `.cursorrules` are legacy/deprecated by Cursor. Cursor uses `.cursor/rules/*.mdc` with frontmatter (`description`, `globs`, `alwaysApply`) or standard `AGENTS.md`.

---

## 2. In-Scope & Out-of-Scope

### In-Scope
1. **Native MCP Schema Support**:
   - Structured JSON parser and writer for `~/.cursor/mcp.json` producing `mcpServers.<tool>` objects matching Cursor's official `stdio` format.
   - Preservation of existing user `mcpServers` entries during install/sync/uninstall.
   - Exclusion of OpenCode `plugin` / `skills.paths` keys from `~/.cursor/mcp.json`.
2. **Native Instruction Rule Formatting**:
   - Deprecate `.cursorrules` as an auto-generated target.
   - Target `.cursor/rules/compound-engineering.mdc` with valid frontmatter (`description`, `globs`, `alwaysApply`) and demarcated `CE-AI MANAGED BLOCK`.
3. **Native Directory Resolution**:
   - `harness_dir`: `~/.cursor`
   - `config_path`: `~/.cursor/mcp.json`
   - `managed_dir`: `~/.cursor/compound-engineering`
4. **Lifecycle Verification**:
   - Full lifecycle test matrix: `install → dry-run → drift/sync → backup → uninstall`.

### Out-of-Scope
- Refactoring non-Cursor harnesses (`claude`, `pi`, `codex`, etc.), which are tracked in separate per-harness issues (#174–#182).

---

## 3. Success Criteria

1. `ce-ai install --harness cursor` writes `~/.cursor/mcp.json` containing `mcpServers` with valid `stdio` configurations for companion tools.
2. `ce-ai uninstall --harness cursor` restores `~/.cursor/mcp.json` to user snapshot (or strips ce-ai entries) and removes `~/.cursor/compound-engineering/` cleanly.
3. Unit and CLI integration tests prove zero OpenCode keys exist in `~/.cursor/mcp.json`.
