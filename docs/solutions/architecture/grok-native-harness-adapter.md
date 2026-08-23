---
module: harness
tags: [grok, harness, adapter, toml, mcp, rules, xai]
problem_type: architectural_refactor
---

# Solution: Grok Native Harness Adapter (Issue #176)

## Problem
Previously, `ce-ai` treated xAI Grok Build CLI (`grok`) as a generic JSON harness, writing OpenCode schema configuration to `~/.config/grok/grok.json`. Official xAI documentation specifies that Grok Build CLI reads TOML configuration from `~/.grok/config.toml` (`[mcp_servers.<name>]` tables), loads skills under `~/.grok/skills/`, and project rules from `.grok/rules/compound-engineering.md`.

## Solution Details
1. **Native TOML Configuration**: Implemented `GrokAdapter` in `src/harness/grok.rs` that reads and mutates `~/.grok/config.toml` using `toml::Table`.
2. **Environment Variable Relocation**: Supported `$GROK_HOME` environment variable override for `<harness_dir>` resolution.
3. **Structured TOML Server Registration**:
   - `register_grok_mcp_server` inserts/updates `[mcp_servers.<name>]` tables with `command`, `args`, and `env`.
   - `unregister_grok_mcp_server` removes specified server entries.
   - Preserves all top-level user TOML tables (`[cli]`, `[marketplace]`, auth).
4. **Project Rule Adoption**: `init-prj` creates or updates `.grok/rules/compound-engineering.md` with demarcated `CE-AI MANAGED BLOCK`.
5. **Clean Uninstallation**: Unregisters `ce-ai` sidecars and removes managed skills without deleting `config.toml` or user custom skills.

## Verification
- Unit tests in `src/harness/grok.rs` verifying `[mcp_servers]` TOML schema manipulation, zero OpenCode key leaks, and managed comment block injection/stripping.
- Integration tests in `tests/cli.rs` (`install_grok_harness_writes_to_native_dir_and_leaves_opencode_pristine`, `init_prj_grok_preserves_preexisting_rules_md_on_deinit`, `uninstall_grok_harness_clean_install_lifecycle`, `uninstall_grok_harness_cleans_native_dir_artifacts_and_preserves_user_configs`).
- 100% green unit & CLI integration test suite.
