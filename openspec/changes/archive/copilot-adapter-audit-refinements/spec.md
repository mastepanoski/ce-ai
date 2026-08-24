# Specification: Copilot Adapter Audit Refinements

## Acceptance Criteria

### R1: Clean Env Object Replacement
- WHEN `register_copilot_mcp_server` is called with an empty `env` map THEN it MUST remove any existing `env` key from `mcpServers.<name>` in `~/.copilot/mcp-config.json`.
- WHEN `register_copilot_mcp_server` is called with a non-empty `env` map THEN it MUST replace the `env` object cleanly without retaining stale environment variables.

### R2: Skills Removal Warning Emission
- WHEN `uninstall` runs for a native harness (Claude, Codex, Copilot, Grok) and removing `<harness_dir>/skills/` fails THEN `ce-ai` MUST emit a warning to `stderr` instead of silently ignoring the error.

### R3: Environment Override Documentation
- WHEN documentation for Copilot native adapter is reviewed THEN `COPILOT_CONFIG_DIR` MUST be documented as `ce-ai`'s environment variable convention for test and profile isolation.
