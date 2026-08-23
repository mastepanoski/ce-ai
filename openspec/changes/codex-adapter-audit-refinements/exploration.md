# Exploration: Codex Adapter Audit Refinements

## 1. Environment Variable Standard (`CODEX_HOME`)
- **Current state**: `src/harness/mod.rs` and `src/harness/codex.rs` check `std::env::var_os("CODEX_CONFIG_DIR")`.
- **Codex CLI spec**: Official Codex CLI uses `CODEX_HOME` to relocate its home directory (defaults to `~/.codex`).
- **Fix**: Replace all occurrences of `CODEX_CONFIG_DIR` with `CODEX_HOME`.

## 2. OpenSpec Contract Alignment (R3)
- **Current state**: `spec.md` (R3) for Codex mentioned `AGENTS.md` generically without clarifying why root `AGENTS.md` is not injected with `CE-AI MANAGED BLOCK`.
- **Fix**: Update R3 to state that project adoption target for Codex is `.codex/AGENTS.md` when `.codex/` directory exists.

## 3. Generic JSON Legacy Code Removal
- **Current state**: `src/harness/generic_json.rs:26` retains `HarnessKind::Codex => base_dir.join(".codex").join("config.json")`.
- **Fix**: Remove `HarnessKind::Codex` from `generic_json.rs` and update unit test.

## 4. Env Map Replacement Consistency
- **Current state**: `register_codex_mcp_server` mutated `env` key-by-key in an existing `mcp_servers` entry.
- **Fix**: Replace `env` map cleanly with the provided `env` parameter (unless empty and existing is non-empty, or replacing entirely) matching Cursor, Claude, and Copilot adapters.
