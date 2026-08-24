# Proposal: Multi-Harness Reconciliation, DeepSeek De-scope & Release Fallback Hardening

## Problem Statement
Following the implementation and release of all 10 real native adapters (`OpenCode`, `Cursor`, `Claude`, `Codex`, `Copilot`, `Grok`, `Kimi`, `Agy`, `Pi`, `Fx`), four cross-cutting issues require reconciliation:
1. **Issue #155 & #183**: `README.md` and OpenSpec `multi_harness_support/spec.md` contain stale claims and spec drift ("12 harnesses" without qualification, fictional JSON schemas).
2. **Issue #180**: `DeepSeek` (`dsh`) launched in developer preview using YAML patch layers under `~/.dsh`. The current fictional JSON path `~/.config/deepseek/deepseek.json` must be de-scoped with an actionable usage error notice.
3. **Issue #202**: `ce-ai upgrade` hard-fails when the GitHub API returns HTTP 403 (unauthenticated rate limit), bypassing the main branch tarball (SF-2) fallback.
4. **Issue #164**: `ce-ai audit` labels its score as "audit score" rather than "configuration coverage".

## Scope Boundaries
- **In Scope**:
  - Qualify `README.md` to state 10 native AI agent harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `fx`) plus custom mode.
  - Reconcile OpenSpec `multi_harness_support/spec.md` with the shipped native adapter architecture.
  - De-scope `deepseek` cleanly: `ce-ai install|uninstall|sync|init-prj --harness deepseek` returns `CeError::Usage` (exit code 2) explaining that `dsh` uses `~/.dsh` YAML patch layers during developer preview.
  - Harden `resolve_latest_release` in `src/source/release.rs` to fall back to `main_tarball_url()` (SF-2) on HTTP 403 / rate limit errors while emitting an informative notice.
  - Update `src/commands/audit.rs` output to display `configuration coverage: X%`.
- **Out of Scope**:
  - Modifying external `dsh` binaries or creating unsupported YAML patch engine layers.
