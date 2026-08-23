---
module: harness
tags: [copilot, harness, adapter, json, mcp, rules, github]
problem_type: architectural_refactor
---

# Solution: Copilot Native Harness Adapter (Issue #177 & Audit Refinements)

## Problem
Previously, `ce-ai` treated GitHub Copilot CLI as a generic JSON harness at `~/.config/copilot/copilot.json`. Official GitHub documentation specifies that Copilot CLI reads MCP configuration from `~/.copilot/mcp-config.json` (`mcpServers` JSON object), stores skills in `~/.copilot/skills/`, and project instructions in `.github/copilot-instructions.md`.

## Refinements Applied
1. **Clean Env Object Replacement**: Updated `register_copilot_mcp_server` to overwrite `server_entry.env = env.clone()`. Serde's `#[serde(skip_serializing_if = "BTreeMap::is_empty")]` automatically removes the `env` key when `env` is empty, avoiding stale environment variables across updates.
2. **Warning Emission on Skills Cleanup**: Updated `src/commands/uninstall.rs` to log a warning on `stderr` when native skills directory removal fails due to permissions or IO errors instead of silencing with `let _ =`.
3. **Environment Override Documentation**: Documented `COPILOT_CONFIG_DIR` in `openspec/changes/copilot-native-harness-adapter/design.md` and `spec.md` as `ce-ai`'s environment variable convention for test and profile isolation.

## Verification
- Unit tests in `src/harness/copilot.rs` verifying `replaces_env_map_cleanly_on_re_registration`, zero OpenCode key leaks, and managed comment block injection/stripping.
- Integration tests in `tests/cli.rs`.
- 100% green unit & CLI integration test suite.
