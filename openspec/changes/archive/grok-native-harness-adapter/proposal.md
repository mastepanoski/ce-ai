# Proposal: Grok Native Harness Adapter (Issue #176)

- **Issue**: Issue #176 (Part of Umbrella #155)
- **Goal**: Implement native xAI Grok Build CLI harness adapter targeting `~/.grok/config.toml` (TOML format), `~/.grok/skills/`, and `.grok/rules/`.

## Problem Statement
Previously, `ce-ai` treated Grok as a generic JSON harness writing OpenCode schema to `~/.config/grok/grok.json`. Official xAI documentation confirms Grok Build CLI (`grok`) reads TOML configuration from `~/.grok/config.toml` (`[mcp_servers.<name>]` tables), stores skills in `~/.grok/skills/`, and loads rules from `.grok/rules/*.md`.

## Proposed Solution
1. Create `GrokAdapter` in `src/harness/grok.rs` implementing `HarnessAdapter`.
2. Support `GROK_HOME` environment variable override for harness directory resolution.
3. Parse and update `~/.grok/config.toml` using `toml::Table` / `toml::Value`, preserving user TOML sections (`[cli]`, `[marketplace]`, auth).
4. Register sidecar MCP servers (`codegraph`, `engram`) natively in `[mcp_servers.<name>]`.
5. Copy managed skills to `<harness_dir>/skills/<name>/SKILL.md`.
6. Adopt project rules in `.grok/rules/compound-engineering.md` with demarcated `CE-AI MANAGED BLOCK`.
7. Wire Grok across `install`, `tools`, `init-prj`, `deinit-prj`, `sync`, `uninstall`, `doctor`, `status`, and `backups` (`grok-` prefix).
