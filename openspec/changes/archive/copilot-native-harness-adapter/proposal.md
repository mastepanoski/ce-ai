# Proposal: GitHub Copilot Native Harness Adapter (Issue #177)

## 1. Problem
`ce-ai` currently lacks native support for the GitHub Copilot CLI / extension harness. Non-OpenCode harnesses were defaulting to generic paths. GitHub Copilot requires native JSON configuration in `~/.copilot/mcp-config.json` using `mcpServers` object, skills stored in `~/.copilot/skills/`, and project instructions in `.github/copilot-instructions.md`.

## 2. Solution Summary
Implement `CopilotAdapter` in `src/harness/copilot.rs` using `serde_json` for native `~/.copilot/mcp-config.json` management, skills placement under `~/.copilot/skills/`, and project instructions in `.github/copilot-instructions.md`.

## 3. In-Scope / Out-of-Scope
- **In-Scope**:
  - Native `~/.copilot/mcp-config.json` JSON reader/writer (`mcpServers` object).
  - Environment variable support for `COPILOT_CONFIG_DIR`.
  - Preservation of unmanaged user JSON keys and settings.
  - Zero OpenCode key leakage (`plugin`, `skills.paths`).
  - Native skills placement under `~/.copilot/skills/<name>/SKILL.md`.
  - `.github/copilot-instructions.md` project rules adoption and de-adoption.
  - Full lifecycle integration (`install`, `sync`, `tools install`, `init-prj`, `deinit-prj`, `uninstall`, `doctor`, `status`).
- **Out-of-Scope**:
  - Managing GitHub Copilot token credentials.

## 4. Risks & Mitigation
- **Risk**: Overwriting user settings in `mcp-config.json`.
- **Mitigation**: Parse `mcp-config.json` into structured `CopilotMcpConfig` struct with `serde_json::Value` extra map, mutate only `mcpServers` sub-objects, and preserve all other top-level keys.
