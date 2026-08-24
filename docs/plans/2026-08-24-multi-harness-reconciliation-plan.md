# Implementation Plan: Multi-Harness Reconciliation & Release Hardening

## Objective
Reconcile multi-harness documentation/specs (Issues #155, #183), de-scope DeepSeek with an actionable usage error (Issue #180), harden release resolution against GitHub API 403 rate limits (Issue #202), and rename audit score to configuration coverage (Issue #164).

## User Review Required
None.

## Proposed Changes

### Documentation & Specs
- Update `README.md` to state 10 native AI agent harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `fx`) and detail native directory locations.
- Reconcile `openspec/changes/multi_harness_support/spec.md` with native adapter architecture.

### Code & Handlers
- In `src/source/release.rs`: Fall back to `main_tarball_url()` (SF-2) when GitHub API returns HTTP 403 / 429 or encounters network send errors.
- In `src/harness/generic_json.rs`: Remove `Deepseek` arm.
- In `src/harness/mod.rs`: Filter out `Deepseek` from `detect_installed_harnesses` and `detect_ce_installed_harnesses`.
- In `src/commands/install.rs`, `uninstall.rs`, `sync.rs`, `init_prj.rs`, `deinit_prj.rs`, `tools.rs`: Reject `deepseek` with `CeError::Usage` (exit code 2) explaining `dsh` developer-preview status and guiding users to supported native harnesses.
- In `src/commands/audit.rs`: Change `score:` to `configuration coverage:` in output header, `--fail-under` flag description, and threshold error message.
- Bump version to `1.18.0` in `Cargo.toml`, `Formula/ce-ai.rb`, and `CHANGELOG.md`.

## Verification Plan
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
