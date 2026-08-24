# Specification: Codex Adapter Audit Refinements

## Acceptance Criteria

### R1: CODEX_HOME Environment Variable Support
- WHEN `CODEX_HOME` is set in the environment THEN `harness_dir(HarnessKind::Codex)` and `default_config_path` MUST resolve to `$CODEX_HOME` and `$CODEX_HOME/config.toml`.
- WHEN `CODEX_HOME` is not set THEN `harness_dir` MUST default to `$HOME/.codex` and `default_config_path` MUST default to `$HOME/.codex/config.toml` (or `home/config.toml` if home points to `.codex` or `config.toml`).

### R2: AGENTS.md Adoption Contract Clarification
- WHEN `ce-ai init-prj` runs in a Codex-enabled project with `.codex/` directory THEN `.codex/AGENTS.md` MUST be updated with the demarcated `CE-AI MANAGED BLOCK`.
- WHEN `AGENTS.md` exists at project root as the primary `ce-ai` directive source THEN it MUST NOT be injected with a managed block to avoid self-referential redundancy.

### R3: Legacy Generic JSON Removal
- WHEN resolving generic JSON adapters THEN `HarnessKind::Codex` MUST NOT map to `.codex/config.json`.

### R4: Clean Env Table Replacement
- WHEN `register_codex_mcp_server` updates an existing server entry THEN the `env` sub-table MUST be cleanly replaced with the provided `env` map (or removed if empty).
