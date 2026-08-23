# Exploration: Codex Native Harness Adapter

## Options Evaluated

1. **Synthetic JSON fallback (Status Quo)**:
   - *Pros*: Zero new code.
   - *Cons*: Fails completely. Codex reads `~/.codex/config.toml` (TOML format), not JSON files in `~/.config/codex/`.

2. **Native TOML Adapter (`src/harness/codex.rs`) (Selected)**:
   - *Pros*: Direct integration with Codex CLI's native configuration format (`[mcp_servers]`), skills layout (`~/.codex/skills/`), and project directives (`AGENTS.md`).
   - *Cons*: Requires TOML parsing and serialization using crate `toml`.

## Technical Architectural Choices
- `toml = "0.8"` dependency added to `Cargo.toml` `[dependencies]`.
- `CodexMcpConfig` uses `toml::Table` / `toml::Value` to parse and serialize `~/.codex/config.toml`, preserving all unmanaged top-level keys and `[mcp_servers]` tables.
- Atomic file writes via `crate::state::write_atomic`.
