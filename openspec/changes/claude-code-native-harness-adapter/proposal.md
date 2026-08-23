# Proposal: Claude Code Native Harness Adapter (Issue #174)

## Problem Statement
The current `claude` harness adapter incorrectly points to `~/.config/claude/claude.json` and writes OpenCode-specific JSON keys (`plugin`, `skills.paths`). Claude Code (Anthropic) does not read `~/.config/claude` or OpenCode keys. Its real home directory is `~/.claude` (respecting `CLAUDE_CONFIG_DIR`), user MCP servers live under `mcpServers` in `~/.claude.json` or `~/.claude/settings.json`, and project instructions live in `CLAUDE.md` or `.claude/CLAUDE.md`.

## Proposed Solution
Transform `claude` into a native harness adapter:
1. Map `harness_dir(Claude)` to `~/.claude` (honoring `CLAUDE_CONFIG_DIR`).
2. Map default config path to `~/.claude.json` / `~/.claude/settings.json`.
3. Implement `ClaudeMcpConfig` and `ClaudeMcpServer` structs with `mcpServers` stdio schema (`type: "stdio"`, `command`, `args`, `env`), url fallback, and `#[serde(flatten)] pub extra` for user property preservation.
4. Support project rule adoption in `./CLAUDE.md` / `.claude/CLAUDE.md`.
5. Integrate across `install`, `sync`, `tools install`, `init-prj`, and `uninstall`.

## Out of Scope
- Direct mutation of app-managed internal state in `~/.claude.json` beyond `mcpServers` and top-level user configuration keys.

## Success Criteria
- Zero OpenCode key leakage into Claude Code configuration files.
- User-defined MCP servers and custom properties are preserved byte-for-byte during merges.
- Full lifecycle tests (`install`, `sync`, `init-prj`, `uninstall`) pass green across Linux, macOS, and Windows.
