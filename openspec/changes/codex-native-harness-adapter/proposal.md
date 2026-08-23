# Proposal: Codex Native Harness Adapter (Issue #175)

## 1. Problem
`ce-ai` currently lacks native support for the Codex AI harness. Non-OpenCode harnesses were defaulting to synthetic JSON paths. Codex requires native TOML configuration in `~/.codex/config.toml` using `[mcp_servers.<name>]` tables, skills stored in `~/.codex/skills/`, and project instructions in `AGENTS.md` / `.codex/AGENTS.md`.

## 2. Solution Summary
Implement `CodexAdapter` in `src/harness/codex.rs` using `toml` serialization (`toml::Value` / `toml::Table`) for native `~/.codex/config.toml` management, skills placement under `~/.codex/skills/`, and project instructions in `AGENTS.md` or `.codex/AGENTS.md`.

## 3. In-Scope / Out-of-Scope
- **In-Scope**:
  - Native `~/.codex/config.toml` TOML reader/writer (`[mcp_servers]` table).
  - Environment variable support for `CODEX_CONFIG_DIR`.
  - Preservation of unmanaged user TOML tables and settings.
  - Zero OpenCode key leakage (`plugin`, `skills.paths`).
  - Native skills placement under `~/.codex/skills/<name>/SKILL.md`.
  - `AGENTS.md` / `.codex/AGENTS.md` project rules adoption and de-adoption.
  - Full lifecycle integration (`install`, `sync`, `tools install`, `init-prj`, `deinit-prj`, `uninstall`).
- **Out-of-Scope**:
  - Cloud synchronization of Codex credentials.

## 4. Risks & Mitigation
- **Risk**: TOML formatting clobbering user comments or formatting.
- **Mitigation**: Parse `config.toml` into structured TOML document/value, mutate only `mcp_servers` sub-tables, and preserve all other top-level keys.
