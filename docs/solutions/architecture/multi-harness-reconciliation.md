---
module: harness
tags: [harness, release, docs, audit, deepseek]
problem_type: architecture
---

# Solution: Multi-Harness Reconciliation, DeepSeek De-scope & Release Hardening

## Problem
Following the implementation of all 10 real native adapters (`OpenCode`, `Cursor`, `Claude`, `Codex`, `Copilot`, `Grok`, `Kimi`, `Agy`, `Pi`, `Fx`), four cross-cutting issues required reconciliation:
1. `README.md` and OpenSpec `multi_harness_support/spec.md` contained stale claims ("12 harnesses" without qualification, fictional JSON schemas) (Issues #155, #183).
2. DeepSeek Harness (`dsh`) launched in developer preview using `~/.dsh` YAML patch layers. Writing fictional JSON (`~/.config/deepseek/deepseek.json`) violated product invariants (Issue #180).
3. `ce-ai upgrade` hard-failed when the GitHub API returned HTTP 403 / 429 rate limit errors (Issue #202).
4. `ce-ai audit` labeled its score as "audit score" rather than "configuration coverage" (Issue #164).

## Solution
1. **README & OpenSpec Reconciliation**: Updated `README.md` and `openspec/changes/multi_harness_support/spec.md` to state the 10 real native harnesses and their exact native formats.
2. **DeepSeek De-scope**: CLI subcommands specifying `deepseek` return `CeError::Usage` (exit code 2) with clear notice that `dsh` uses `~/.dsh` YAML patch layers during preview, and excluded `deepseek` from host harness auto-detection.
3. **GitHub API 403 / Rate Limit Fallback**: Hardened `resolve_latest_release` in `src/source/release.rs` to catch network errors and non-success HTTP status codes (403/429), emitting stderr guidance and returning `Ok(None)` to fall back to `main_tarball_url()` (SF-2).
4. **Audit Configuration Coverage**: Updated `src/commands/audit.rs` output header, `--fail-under` docstring, and threshold error message to `configuration coverage`.

## Verification
- `cargo fmt --check` clean.
- `cargo clippy --all-targets --all-features -- -D warnings` clean.
- `cargo test` passing 100% across unit and integration tests (146 unit, 79 CLI integration).
