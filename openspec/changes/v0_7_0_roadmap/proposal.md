# OpenSpec Proposal: Release v0.7.0 — Workspace Overrides & Complete Multi-Harness Uninstall

## Problem Statement
While `ce-ai` v0.6.0 established proactive workflow observability, live TUI gauges, and real-time sync watchers, two critical capabilities are missing for repository isolation and lifecycle maintenance:

1. **Lack of Repository-Local Overrides**: Model assignments and active profiles are currently strictly global (`~/.config/ce-ai/state.json`). Teams working on different repositories cannot specify project-specific model assignments (e.g. allocating `ce-work` to a specialized local LLM or distinct provider) within the project tree (`.ce-ai.json`).
2. **Incomplete Uninstall Parity (Issue #64)**: `ce-ai uninstall` currently only restores the latest `opencode.json` backup for OpenCode, leaving behind managed plugin loaders across other installed harnesses (`Claude Code`, `Cursor`, `Copilot`, `Pi`, `Antigravity`, etc.). Users have no supported command path to completely wipe managed loaders and skills across all installed harnesses (`--harness all --all`).

## In-Scope vs Out-of-Scope Boundaries

### In-Scope:
- **Workspace Configuration Overrides (`.ce-ai.json`)**:
  - Support `.ce-ai.json` located in current working directory or Git repository root.
  - Implement key-level fallback resolution: `.ce-ai.json` fields take precedence over global `~/.config/ce-ai/state.json`.
- **Complete Multi-Harness Uninstall (`ce-ai uninstall --harness <name|all> --all`)**:
  - Support `--harness <name|all>` flag in `ce-ai uninstall`.
  - Delete managed loader scripts and skill paths across all targeted harnesses when `--all` is specified.
  - Add `--yes` / `-y` flag for non-interactive automated script execution.
  - Preserve all unmanaged user configurations and custom skills.

### Out-of-Scope:
- Binary package distribution via Homebrew/WinGet/APT (deferred to `v0.8.0` — Issues #2, #3, #28).
- Penetration testing & ISO 27001 threat matrix audits (deferred to `v0.9.0`).

## Risk Evaluation
- **Config Drift / Overwrite Risk**: Low. `.ce-ai.json` is strictly read-only for runtime precedence resolution unless explicitly modified by `ce-ai`.
- **Destructive File Removal Risk**: Low. `ce-ai uninstall --all` only targets SHA256-verified managed assets listed in manifest files, protecting user-created plugins.

## Success Criteria
1. `ce-ai models list` reflects `.ce-ai.json` overrides when executed inside a repository containing `.ce-ai.json`.
2. `ce-ai uninstall --harness all --all --yes` completely removes all managed loaders across all 12 installed harnesses cleanly.
3. 100% test coverage with zero clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
