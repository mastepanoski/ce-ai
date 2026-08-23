# Proposal: Codex Adapter Audit Refinements

- **Issue / Audit Findings**: Audit findings on Issue #175 (Codex Native Adapter)
- **Goal**: Address audit findings in Codex adapter regarding environment variable standard (`CODEX_HOME`), spec alignment for `AGENTS.md` project rules, removal of dead legacy generic JSON code, and env map replacement consistency.

## Problem Statement
1. `ce-ai` checked `CODEX_CONFIG_DIR` instead of the official `CODEX_HOME` environment variable used by Codex CLI to relocate its configuration directory.
2. `spec.md` (R3) in `codex-native-harness-adapter` contained a minor spec drift regarding `AGENTS.md` vs `.codex/AGENTS.md` adoption logic.
3. `src/harness/generic_json.rs` retained a dead legacy mapping (`.codex/config.json`).
4. `register_codex_mcp_server` merged `env` keys individually instead of replacing the `env` table cleanly like Cursor/Claude/Copilot adapters.

## Proposed Solution
1. Update `HarnessKind::Codex.harness_dir` and `CodexAdapter::default_config_path` to use `CODEX_HOME` instead of `CODEX_CONFIG_DIR`.
2. Amend `openspec/changes/codex-native-harness-adapter/spec.md` (R3) and `openspec/changes/codex-adapter-audit-refinements/spec.md` to accurately reflect `.codex/AGENTS.md` adoption rules.
3. Remove legacy `HarnessKind::Codex` entry from `src/harness/generic_json.rs`.
4. Update `register_codex_mcp_server` in `src/harness/codex.rs` to replace the `env` table cleanly when registering.
