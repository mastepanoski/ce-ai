---
module: src/harness/claude.rs
tags: [harness, claude, claude-code, mcp, skills]
problem_type: architecture
---

# Claude Code Native Harness Adapter Implementation

## What
Implemented native Claude Code harness support in `ce-ai`, mapping harness home directory to `~/.claude` (respecting `CLAUDE_CONFIG_DIR`), default user MCP configuration to `~/.claude.json` / `~/.claude/settings.json`, native skills to `~/.claude/skills/<name>/SKILL.md`, and project instructions to `CLAUDE.md` / `.claude/CLAUDE.md`.

## Why
Previously, non-OpenCode harnesses received synthetic copies of OpenCode's JSON schema (`plugin`, `skills.paths`), which Claude Code ignored completely. Claude Code requires `mcpServers` stdio configurations in `~/.claude.json` / `settings.json`, skills under `~/.claude/skills/`, and project instructions in `CLAUDE.md` or `.claude/CLAUDE.md`.

## Where
- `src/harness/claude.rs`: Native `ClaudeMcpConfig`, `ClaudeMcpServer` schemas, `register_claude_mcp_server`, `unregister_claude_mcp_server`, `update_claude_md`, and `strip_managed_block`.
- `src/harness/mod.rs`: Updated `HarnessKind::Claude.harness_dir` to respect `CLAUDE_CONFIG_DIR` or default to `~/.claude`. Updated `config_path` for Claude.
- `src/commands/install.rs`: Registered sidecar MCP servers (`codegraph`, `engram`) natively and copied skills to `~/.claude/skills/`.
- `src/commands/tools.rs`: Added Claude `mcpServers` registration on `ce-ai tools install <tool>`.
- `src/commands/init_prj.rs` & `src/commands/deinit_prj.rs`: Added `CLAUDE.md` / `.claude/CLAUDE.md` adoption and de-adoption.
- `src/commands/sync.rs`: Reconciled Claude `mcpServers` drift.
- `src/commands/uninstall.rs`: Restored user backups or removed `ce-ai` sidecars from `mcpServers`, keeping user-defined servers intact.
- `src/source/archive.rs`: Added `copy_dir_all` helper.
- `src/state/backups.rs`: Tagged Claude backups with `claude-` prefix.

## Learned & Gotchas
1. **Config Path Precedence**: Claude Code settings can live in `~/.claude/settings.json` or `~/.claude.json`. `ClaudeAdapter::default_config_path` checks if `~/.claude/settings.json` exists and contains `"mcpServers"`, otherwise defaults to `~/.claude.json`.
2. **Environment Variable Override**: `CLAUDE_CONFIG_DIR` overrides the default `~/.claude` directory when set.
3. **Optional Type Field in MCP Server**: Serializing `"type": "stdio"` on existing user stdio MCP servers mutates user definitions unnecessarily; making `r#type` an `Option<String>` with `skip_serializing_if = "Option::is_none"` keeps existing user MCP server entries pristine.
