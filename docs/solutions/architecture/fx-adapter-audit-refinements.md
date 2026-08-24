---
title: "Vercel Labs fx Adapter Audit Refinements"
category: "architecture"
tags: ["fx", "vercel-labs", "harness-adapter", "audit", "mcp-json"]
date: "2026-08-24"
problem_type: "audit_refinements"
---

# Vercel Labs fx Adapter Audit Refinements

## Problem Statement
The cross-adapter audit suite identified four specific audit findings for Vercel Labs' `fx` native harness adapter:
1. `default_config_path` checked `home.join("mcp.json").exists()`, making path resolution dependent on filesystem state. Passing `$HOME` when `~/mcp.json` existed resulted in writing to `~/mcp.json` instead of `~/.fx/mcp.json`.
2. `unregister_fx_mcp_server` used `let _ = std::fs::remove_file(config_path);`, silently swallowing deletion IO errors (violating Invariant #5).
3. `FX_HOME` environment variable override is a `ce-ai` extension convention not in official `FX_*` variable docs.
4. Clean collision handling removes stale `type` fields from `extra` map (`existing_extra.remove("type")`) before inserting `"type": "local"`.

## Refinements & Architecture

### 1. OpenSpec Contract & Documentation
- Documented deterministic path resolution purely by basename matching in `FxAdapter::default_config_path`.
- Documented error propagation policy on empty file deletion in `unregister_fx_mcp_server`.
- Documented `$FX_HOME` environment variable override as a `ce-ai` extension convention in `design.md` and `spec.md`.
- Documented extra map `type` collision cleanup pattern in `design.md` and `spec.md`.

### 2. Implementation Hardening
- Removed `home.join("mcp.json").exists()` in `src/harness/fx.rs`, restoring deterministic path resolution.
- Propagated IO errors on `std::fs::remove_file` in `unregister_fx_mcp_server` while ignoring `ErrorKind::NotFound`.
- Expanded unit tests in `src/harness/fx.rs` verifying path resolution when `$HOME/mcp.json` pre-exists, extra map `type` purging on re-registration, and IO error propagation.
