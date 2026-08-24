# Proposal: Copilot Adapter Audit Refinements

- **Goal**: Refine the native GitHub Copilot CLI adapter based on audit feedback.

## Audit Findings Addressed
1. **Clean Env Object Replacement**: In `src/harness/copilot.rs`, update `register_copilot_mcp_server` to cleanly overwrite `server_entry.env = env.clone()`. Combined with `#[serde(skip_serializing_if = "BTreeMap::is_empty")]`, an empty `env` map automatically removes the `env` key from `~/.copilot/mcp-config.json`, preventing stale environment variables from persisting across updates.
2. **Warning Emission on Skills Cleanup**: In `src/commands/uninstall.rs`, log warnings if deleting native skills directories (`skills/`) encounters permission or IO errors instead of silencing with `let _ =`. Because skills directory removal is shared across native harnesses, this warning emission improves error visibility for Claude, Codex, Copilot, and Grok.
3. **Environment Override Documentation**: Document `COPILOT_CONFIG_DIR` in `openspec/changes/copilot-native-harness-adapter/design.md` and `openspec/changes/copilot-native-harness-adapter/spec.md` as `ce-ai`'s environment override convention for test and profile isolation.
