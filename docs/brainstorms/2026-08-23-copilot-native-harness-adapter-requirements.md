# Brainstorm: GitHub Copilot Native Harness Adapter Requirements (Issue #177)

- **Date**: 2026-08-23
- **Feature**: Native GitHub Copilot Harness Adapter (Umbrella #155 / Issue #177)
- **Goal**: Implement native harness adapter for GitHub Copilot CLI / extension (`~/.copilot/mcp-config.json`).

## 1. Problem Statement
Previously, `ce-ai` treated non-OpenCode harnesses uniformly using generic config path assumptions (`~/.config/github-copilot/config.json`). GitHub Copilot natively uses `~/.copilot/mcp-config.json` (JSON format) with `mcpServers` object, skills stored in `<harness_dir>/skills/`, and project instructions in `.github/copilot-instructions.md`.

## 2. Target Layout & Schema
- **Harness Directory**: `<harness_dir>` (`$COPILOT_CONFIG_DIR` if set, otherwise `$HOME/.copilot`).
- **Configuration File**: `<harness_dir>/mcp-config.json` (JSON file format).
- **Native MCP Schema**: `mcpServers` object containing `command`, `args`, `env`.
  ```json
  {
    "mcpServers": {
      "codegraph": {
        "command": "codegraph",
        "args": ["mcp"]
      },
      "engram": {
        "command": "engram",
        "args": ["serve"]
      }
    }
  }
  ```
- **User Config Preservation**: Preserves all top-level JSON keys and unmanaged `mcpServers` entries.
- **Zero OpenCode Leakage**: `plugin` and `skills.paths` keys are never written to `~/.copilot/mcp-config.json`.
- **Native Skills Placement**: Managed skills placed under `<harness_dir>/skills/<name>/SKILL.md`.
- **Project Rules**: Rules written to `.github/copilot-instructions.md` with demarcated `CE-AI MANAGED BLOCK`.

## 3. Lifecycle Integration
- `install`: Provision `<harness_dir>/mcp-config.json` with sidecars (`codegraph`, `engram`), copy skills to `<harness_dir>/skills/`.
- `tools install <tool>`: Register companion tool natively in `<harness_dir>/mcp-config.json`.
- `init-prj`: Adopt project instructions in `.github/copilot-instructions.md`.
- `deinit-prj`: Cleanly strip managed block from `.github/copilot-instructions.md`.
- `sync`: Reconcile Copilot MCP server definitions and skills drift.
- `uninstall`: Unregister `ce-ai` sidecars from `mcpServers` and clean skills directory, keeping user JSON configuration intact.
