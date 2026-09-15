---
module: harness
tags: [multi-harness, adapters, registration-spec, sync]
problem_type: architecture
title: "Multi-Harness Support Implementation"
applies_when: "When ce-ai originally managed only OpenCode (opencode.json)."
---

# Solution: Multi-Harness Support Implementation

> Updated for v1.19.x–v1.20.x (registration strategy table, real custom
> mode, sync error transparency); refreshed v1.48.0 (registration table
> relocated to `harness::registration`, manifest re-harvest, verification
> matrix scope corrected). Original v0.3.0 notes corrected — they
> described a `generic_json.rs` adapter that only ever implemented Custom,
> and listed DeepSeek as supported before its de-scope.

## Problem Statement

`ce-ai` originally managed only OpenCode (`opencode.json`). Teams use
multiple AI coding tools, so `ce-ai` needed a unified domain model and
adapter interface covering installation, sync reconciliation, model
assignment translation, and host harness auto-probing across vendors.

## Solution Architecture (current)

1. **`HarnessKind` enum (`src/harness/mod.rs`)** — 12 variants:
   10 native (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`,
   `grok`, `kimi`, `agy`, `fx`) + `custom` (real fallback mode since
   v1.19.0) + `deepseek` (de-scoped in v1.18.0: parse → `CeError::Usage`).
2. **Native adapters (`src/harness/<vendor>.rs`)** — one module per vendor
   owning its config writer (`register_<vendor>_mcp_server`) with identical
   signatures; no shared generic-JSON adapter ever served them.
3. **Exhaustive registration table** — `registration_spec(kind)` in
   `src/harness/registration.rs` maps each kind to its vendor MCP
   registrar (dedicated arms remain for Custom, OpenCode, and de-scoped
   Deepseek); adding a variant is a compile error until classified
   (v1.19.2). Both consumers share the table: `install.rs` and `sync.rs`
   import it (consolidation completed for install in v1.48.0).
4. **Custom mode contract** — flags ▸ state snapshot ▸
   `~/.ce-ai/custom_harness.json`; assets copied into user directories with
   a SHA256 manifest; surgical uninstall (v1.19.0).
5. **Sync transparency** — per-harness arms propagate IO errors; the
   verification matrix hash-checks only surfaces with managed assets
   (OpenCode, Custom, and adopted ledger surfaces) — skills are never
   copied into native harness directories, so registration-only harnesses
   are reported `registered (nothing to verify)` (v1.19.1; scope
   corrected v1.48.0). Per-harness SHA256 integrity instead lives in each
   harness's `install-manifest.json`, re-harvested from the on-disk
   managed tree on every sync (`InstallManifest::harvest`,
   `src/opencode/manifest.rs`), and is certified independently by
   `ce-ai doctor`, `ce-ai status`, and `ce-ai workflow resume`.
   Best-effort cleanups report via `state::report_best_effort_*` helpers
   (v1.20.1).

## Key Invariants

- OpenCode-format keys (`plugin`, `skills.paths`) are written **only** to
  OpenCode's own config — never as a fallback for other harnesses.
- Unsupported kinds fail with a named `CeError::Runtime` instead of
  receiving fabricated mutations.
- Custom roots are user-owned: uninstall removes exactly the
  manifest-recorded files.

## DoD Verification (current gates)

- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`.
- Unit + CLI integration suites green (hermetic per-harness env fixtures).
- `make e2e` Docker gate; 100% green cross-platform CI matrix.
