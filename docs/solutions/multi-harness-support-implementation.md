---
module: harness
tags: [multi-harness, adapters, registration-spec, sync]
problem_type: architecture
---

# Solution: Multi-Harness Support Implementation

> Updated for v1.19.x–v1.20.x (registration strategy table, real custom
> mode, sync error transparency). Original v0.3.0 notes corrected — they
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
   `src/commands/sync.rs` maps each kind to `{register_mcp, skills_subpath}`;
   adding a variant is a compile error until classified (v1.19.2). The same
   consolidation for `install.rs` is tracked follow-up debt.
4. **Custom mode contract** — flags ▸ state snapshot ▸
   `~/.ce-ai/custom_harness.json`; assets copied into user directories with
   a SHA256 manifest; surgical uninstall (v1.19.0).
5. **Sync transparency** — per-harness arms propagate IO errors; the
   verification matrix hash-checks all eight directory-copying skill
   surfaces (v1.19.1); best-effort cleanups report via
   `state::report_best_effort_*` helpers (v1.20.1).

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
