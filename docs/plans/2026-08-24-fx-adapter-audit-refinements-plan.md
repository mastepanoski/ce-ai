# Implementation Plan: Vercel Labs fx Adapter Audit Refinements

## Objective
Address and document findings from the Vercel Labs `fx` adapter audit across OpenSpec, unit tests, and source code.

## User Review Required
None. Internal audit refinement.

## Proposed Changes

### OpenSpec & Documentation
- Update `openspec/changes/fx-adapter-audit-refinements/` (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`).
- Document `FX_HOME` environment variable extension.
- Document `type` key extra map collision cleanup.

### Code & Tests (`src/harness/fx.rs`)
- Remove `home.join("mcp.json").exists()` from `default_config_path`.
- Handle `remove_file` errors in `unregister_fx_mcp_server` without silencing (ignoring `NotFound`).
- Add unit tests verifying deterministic path resolution regardless of pre-existing `mcp.json` at root home directory.

## Verification Plan
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
