# Brainstorm: Claude Code Native Harness Adapter Requirements (Issue #174)

## Context & Problem Statement
Currently, `ce-ai` treats `claude` harness as writing synthetic OpenCode schema (`plugin`, `skills.paths`) into `~/.config/claude/claude.json`.
Claude Code (Anthropic) does not read `~/.config/claude/claude.json` or OpenCode keys. Its real home directory is `~/.claude` (respecting `CLAUDE_CONFIG_DIR` if set), user settings live in `~/.claude/settings.json`, user MCP servers live under `mcpServers` stdio schema in `~/.claude.json` (or `~/.claude/settings.json`), project instructions live in `CLAUDE.md` or `.claude/CLAUDE.md`, and skills live in `~/.claude/skills/<name>/SKILL.md`.

## Target Native Layout
1. **Harness Directory**:
   - `harness_dir(Claude)` -> `std::env::var_os("CLAUDE_CONFIG_DIR").map(PathBuf::from).unwrap_or_else(|| home.join(".claude"))`.
2. **Configuration Paths & MCP Servers**:
   - Primary user config / settings: `~/.claude/settings.json`
   - User MCP server config: `~/.claude.json` (or `~/.claude/settings.json` with root key `mcpServers`).
   - Stdio MCP Server Schema: `{"mcpServers": {"<name>": {"command": "<cmd>", "args": [...], "env": {...}}}}`.
   - Preserve unmanaged top-level keys (`userSetting`, `trustedFolders`, etc.) and extra per-server fields via Serde `#[serde(flatten)] pub extra`.
   - Zero OpenCode key leakage: `plugin` and `skills.paths` are never written to Claude Code config files.
3. **Project Instructions (`init-prj`)**:
   - Write project directives to `./CLAUDE.md` or `.claude/CLAUDE.md` with demarcated `CE-AI MANAGED BLOCK` comment tags and optional `@AGENTS.md` import line.
4. **Skills Location**:
   - Managed skills directory: `~/.claude/skills/`.
5. **Lifecycle Integration**:
   - `install`: Copy loader & skills into `~/.claude/skills/` and register `mcpServers` (`codegraph`, `engram`) in `~/.claude.json` / `settings.json`.
   - `tools install`: Register companion tools (`context7`, `rtk`, `codegraph`, `engram`) in Claude MCP config.
   - `sync`: Reconcile `mcpServers` drift.
   - `uninstall`: Restore user backup or unregister `ce-ai` MCP servers; delete file ONLY if empty and created by `ce-ai`.

## Safety & Non-negotiable Invariants
- `write_atomic` with `tempfile` and atomic rename for all mutations targeting `~/.claude.json` or `~/.claude/settings.json`.
- Preserve existing user-defined MCP servers and settings intact.
- Propagate all IO and Serde errors with `?` (no `let _ =` error swallowing).
