# Plan: Claude Code Native Harness Adapter (Issue #174)

## Overview
Implement native Claude Code harness support in `ce-ai`, mapping harness directory to `~/.claude` (respecting `CLAUDE_CONFIG_DIR`), default config path to `~/.claude.json` / `~/.claude/settings.json`, using native `mcpServers` stdio schema, zero OpenCode key leaks, project directive rules in `CLAUDE.md` / `.claude/CLAUDE.md`, and clean sidecar lifecycle management.

## Implementation Steps
1. **Module Creation (`src/harness/claude.rs`)**:
   - Define `ClaudeAdapter`, `ClaudeMcpConfig`, and `ClaudeMcpServer` structs.
   - Implement `register_claude_mcp_server`, `unregister_claude_mcp_server`, and `update_claude_md`.
2. **Harness Module Wiring (`src/harness/mod.rs`)**:
   - Update `HarnessKind::Claude.harness_dir` to respect `CLAUDE_CONFIG_DIR` or default to `~/.claude`.
   - Update `ClaudeAdapter::default_config_path` to `home.join(".claude.json")`.
3. **Lifecycle Commands Wiring**:
   - `install.rs`: Call `register_claude_mcp_server` for `codegraph` and `engram`.
   - `tools.rs`: Call `register_claude_mcp_server` on `ce-ai tools install <tool>`.
   - `init_prj.rs`: Write `CLAUDE.md` or `.claude/CLAUDE.md` with managed block.
   - `sync.rs`: Reconcile Claude `mcpServers` drift.
   - `uninstall.rs`: Call `unregister_claude_mcp_server` for `ce-ai` sidecars.
4. **Backup Tagging (`src/state/backups.rs`)**:
   - Recognize `.claude` or `claude` in path resolution.
5. **Testing**:
   - Unit tests in `src/harness/claude.rs`.
   - CLI integration tests in `tests/cli.rs`.
