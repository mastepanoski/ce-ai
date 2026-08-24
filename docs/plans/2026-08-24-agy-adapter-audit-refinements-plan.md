# Implementation Plan: Google Antigravity (AGY) Adapter Audit Refinements

## Objective
Address and document findings from the Google Antigravity (`agy`) adapter audit across OpenSpec, unit tests, and documentation.

## User Review Required
None. Internal audit refinement.

## Proposed Changes

### OpenSpec & Documentation
- Update `openspec/changes/agy-adapter-audit-refinements/` (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`).
- Document `ANTIGRAVITY_CONFIG_DIR` and `GEMINI_HOME` extension conventions.
- Document Project Rules Architecture (`GEMINI.md` canonical instruction file, `.agents/rules/compound-engineering.md` derived stub).
- Document `serverUrl` collision resetting policy (`server_url = None` on managed tool collision while preserving non-colliding remote servers).
- Document `HarnessAdapter` zero-argument trait signatures (`canonical_instruction_file`, `derived_stub_files`).

### Code & Tests (`src/harness/agy.rs`)
- Add unit tests verifying `serverUrl` resetting to `None` when registering managed stdio server over pre-existing remote entry.
- Add unit tests verifying environment variable override resolution (`ANTIGRAVITY_CONFIG_DIR`, `GEMINI_HOME`).

## Verification Plan
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
