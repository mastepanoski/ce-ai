# Exploration: GitHub Copilot Native Harness Adapter

## Options Evaluated

1. **Generic Config Fallback (Status Quo)**:
   - *Pros*: Zero new code.
   - *Cons*: Fails completely. GitHub Copilot reads `~/.copilot/mcp-config.json` (JSON format), not generic files in `~/.config/github-copilot/`.

2. **Native JSON Adapter (`src/harness/copilot.rs`) (Selected)**:
   - *Pros*: Direct integration with GitHub Copilot CLI's native configuration format (`mcpServers`), skills layout (`~/.copilot/skills/`), and project directives (`.github/copilot-instructions.md`).
   - *Cons*: Requires custom JSON serialization and field preservation logic.

## Technical Architectural Choices
- `serde_json` is standard across the codebase.
- `CopilotMcpConfig` uses `#[serde(flatten)] extra: BTreeMap<String, serde_json::Value>` to preserve all unmanaged top-level keys.
- Atomic file writes via `crate::state::write_atomic`.
