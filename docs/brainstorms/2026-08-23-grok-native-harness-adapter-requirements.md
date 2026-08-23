# Brainstorm: Grok Native Harness Adapter Requirements (Issue #176)

- **Date**: 2026-08-23
- **Feature**: Native xAI Grok Build Harness Adapter (Umbrella #155 / Issue #176)
- **Goal**: Implement native harness adapter for Grok Build CLI (`~/.grok/config.toml`).

## 1. Problem Statement
Previously, `ce-ai` treated Grok as a generic JSON harness at `~/.config/grok/grok.json`. Official xAI Grok Build CLI documentation specifies that Grok uses `~/.grok/config.toml` (TOML format) with `[mcp_servers.<name>]` tables, skills stored under `<harness_dir>/skills/`, and project rules in `.grok/rules/<name>.md` or `AGENTS.md`.

## 2. Target Layout & Schema
- **Harness Directory**: `<harness_dir>` (`$GROK_HOME` if set, otherwise `$HOME/.grok`).
- **Configuration File**: `<harness_dir>/config.toml` (TOML format).
- **Native MCP Schema**: `[mcp_servers.<name>]` tables in `config.toml` containing `command`, `args`, `env`.
  ```toml
  [mcp_servers.codegraph]
  command = "codegraph"
  args = ["mcp"]

  [mcp_servers.engram]
  command = "engram"
  args = ["serve"]
  ```
- **User Config Preservation**: Preserves all top-level TOML tables (`[cli]`, `[marketplace]`, auth) and unmanaged `[mcp_servers]` entries.
- **Zero OpenCode Leakage**: `plugin` and `skills.paths` keys are never written to `~/.grok/config.toml`.
- **Native Skills Placement**: Managed skills placed under `<harness_dir>/skills/<name>/SKILL.md`.
- **Project Rules**: Rules written to `.grok/rules/compound-engineering.md` with demarcated `CE-AI MANAGED BLOCK`.

## 3. Lifecycle Integration
- `install`: Provision `<harness_dir>/config.toml` with sidecars (`codegraph`, `engram`), copy skills to `<harness_dir>/skills/`.
- `tools install <tool>`: Register companion tool natively in `<harness_dir>/config.toml`.
- `init-prj`: Adopt project instructions in `.grok/rules/compound-engineering.md`.
- `deinit-prj`: Cleanly strip managed block from `.grok/rules/compound-engineering.md`.
- `sync`: Reconcile Grok MCP server definitions and skills drift.
- `uninstall`: Unregister `ce-ai` sidecars from `[mcp_servers]` and clean skills directory, keeping user TOML configuration intact.
