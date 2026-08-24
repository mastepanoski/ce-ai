# Proposal: Vercel Labs fx Adapter Audit Refinements

## Problem Statement
Audit findings across the native harness adapters identified specific documentation and behavior clarifications for the Vercel Labs `fx` adapter:
1. `default_config_path` in `src/harness/fx.rs` includes `home.join("mcp.json").exists()`, making path resolution filesystem-dependent. If a plain `$HOME` directory contains an unrelated `~/mcp.json`, `ce-ai` writes there instead of `~/.fx/mcp.json`.
2. `unregister_fx_mcp_server` uses `let _ = std::fs::remove_file(config_path);`, silently swallowing deletion IO errors (violating Invariant #5).
3. `FX_HOME` is a custom `ce-ai` relocation extension not present in official `FX_*` environment variables list.
4. Clean collision handling removes `type` from `extra` before re-inserting (`existing_extra.remove("type")`).

## Scope Boundaries
- **In Scope**:
  - Remove filesystem-dependent `.exists()` check in `FxAdapter::default_config_path`, keeping deterministic basename checks.
  - Propagate IO errors on `std::fs::remove_file` in `unregister_fx_mcp_server` (ignoring `NotFound`).
  - Document `FX_HOME` extension convention in OpenSpec `design.md`.
  - Document `type` key collision cleanup in OpenSpec `design.md`.
  - Add unit tests for deterministic path resolution and non-silent file deletion.
- **Out of Scope**:
  - Changing `fx` binary path conventions or breaking existing `mcp.json` schemas.
