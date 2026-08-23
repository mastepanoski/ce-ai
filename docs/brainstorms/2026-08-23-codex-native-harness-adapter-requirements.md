# Brainstorm: Codex Native Harness Adapter Requirements (Issue #175)

- **Date**: 2026-08-23
- **Feature**: Native Codex AI Harness Adapter (Umbrella #155 / Issue #175)
- **Goal**: Implement native harness adapter for OpenAI Codex CLI / harness (`~/.codex/config.toml`).

## 1. Problem Statement
Previously, `ce-ai` treated non-OpenCode harnesses uniformly using fictional JSON structures (`~/.config/codex/codex.json`). Codex natively uses `~/.codex/config.toml` (TOML format) with `[mcp_servers.<name>]` tables, skills stored in `~/.codex/skills/`, and project instructions in `AGENTS.md` / `.codex/AGENTS.md`.

## 2. Target Layout & Schema
- **Harness Directory**: `~/.codex` (honors `CODEX_CONFIG_DIR` environment variable when set).
- **Configuration File**: `~/.codex/config.toml` (TOML file format).
- **Native MCP Schema**: `[mcp_servers.<name>]` tables containing `command`, `args`, `env`.
  ```toml
  [mcp_servers.codegraph]
  command = "codegraph"
  args = ["mcp"]

  [mcp_servers.engram]
  command = "engram"
  args = ["serve"]
  ```
- **User Config Preservation**: Preserves all top-level TOML keys and unmanaged `[mcp_servers]` tables.
- **Zero OpenCode Leakage**: `plugin` and `skills.paths` keys are never written to `~/.codex/config.toml`.
- **Native Skills Placement**: Managed skills placed under `~/.codex/skills/<name>/SKILL.md`.
- **Project Rules**: Rules written to `AGENTS.md` or `.codex/AGENTS.md` with demarcated `CE-AI MANAGED BLOCK`.

## 3. Lifecycle Integration
- `install`: Provision `~/.codex/config.toml` with sidecars (`codegraph`, `engram`), copy skills to `~/.codex/skills/`.
- `tools install <tool>`: Register companion tool natively in `~/.codex/config.toml`.
- `init-prj`: Adopt project instructions in `AGENTS.md` or `.codex/AGENTS.md`.
- `deinit-prj`: Cleanly strip managed block from `AGENTS.md` / `.codex/AGENTS.md`.
- `sync`: Reconcile Codex MCP server definitions and skills drift.
- `uninstall`: Unregister `ce-ai` sidecars from `[mcp_servers]`, keeping user TOML configuration intact.
