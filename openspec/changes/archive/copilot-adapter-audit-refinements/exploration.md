# Exploration: Copilot Adapter Audit Refinements

## 1. Env Map Replacement Behavior
In `src/harness/copilot.rs`, setting `server_entry.env = env.clone();` on `CopilotMcpServer` ensures that non-empty maps are serialized into `env`, while empty maps trigger Serde's `#[serde(skip_serializing_if = "BTreeMap::is_empty")]` annotation, removing the `env` key from `~/.copilot/mcp-config.json`.

## 2. Skills Directory Cleanup Warnings
In `src/commands/uninstall.rs`, replacing `let _ = std::fs::remove_dir_all(&skills_dir);` with explicit warning logging ensures users are informed if native skills directory cleanup fails due to locked files or permissions.

## 3. Environment Override Documentation
Updating `openspec/changes/copilot-native-harness-adapter/design.md` and `openspec/changes/copilot-native-harness-adapter/spec.md` clarifies that `COPILOT_CONFIG_DIR` is a `ce-ai` test isolation convention.
