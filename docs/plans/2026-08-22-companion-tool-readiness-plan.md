# Implementation Plan: Companion-Tool Readiness & Version Freshness

- **Date:** 2026-08-22
- **Issue:** #112
- **Origin:** `docs/brainstorms/2026-08-22-companion-tool-readiness-requirements.md`
- **OpenSpec Change:** `companion-tool-readiness-and-freshness`
- **Status:** Approved (Execution Plan Ready)

---

## 🎯 1. Executive Summary & Goals

Implement companion-tool version freshness checks, skill presence suggestions (`sequential-thinking`), self-update recommendations (`ce-ai upgrade`), and graceful offline degradation across `ce-ai doctor` and `ce-ai tools status`.

---

## 🏛️ 2. Architectural Design & Touch Surface

### Target Files:
- `src/source/tools_registry.rs` (net-new module: registry constants, TTL cache, SemVer comparison).
- `src/source/mod.rs` (module export).
- `src/commands/tools.rs` (status rendering, version extraction, skill suggestions).
- `src/commands/doctor.rs` (readiness findings, `--strict` flag, self-update hints).
- `tests/cli.rs` (integration test scenarios).

---

## 📋 3. Implementation Units

### Unit U1: Embedded Registry & TTL Cache (`src/source/tools_registry.rs`)
- Implement `FreshnessStatus` enum and `ToolsRegistry` struct.
- Atomic JSON write to `~/.ce-ai/cache/companion-registry.json`.
- Non-blocking HTTP fetch with 500ms timeout & fallback to `FreshnessStatus::Offline`.

### Unit U2: Status Command Enhancements (`src/commands/tools.rs`)
- Version extraction from CLI outputs.
- Skill Registry suggestions for `sequential-thinking`.
- `--json` machine-readable output.

### Unit U3: Doctor Probes & Strict Flag (`src/commands/doctor.rs`)
- Add `--strict` CLI flag.
- Exit code rules: missing $\rightarrow$ Exit 1; outdated $\rightarrow$ info / Exit 0 (unless `--strict`).
- Self-update hints for `ce-ai`.

### Unit U4: Empirical Verification
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `make e2e`
