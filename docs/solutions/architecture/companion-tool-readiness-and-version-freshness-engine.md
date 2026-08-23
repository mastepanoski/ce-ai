---
module: src/source/tools_registry.rs
tags: [companion-tools, version-freshness, doctor, tools-status, semver, offline-resilience]
problem_type: architecture
---

# Companion-Tool Readiness & Version Freshness Engine

## Problem
`ce-ai` detected whether companion binaries (Engram, CodeGraph, Context7, RTK) were present on `PATH`, but lacked **version freshness validation**, **skill presence suggestions** (e.g. `sequential-thinking`), and **self-update recommendations**. AI agent sessions could execute against outdated sidecars or missing reasoning skills without notice.

## Solution
Built the `ToolsRegistryCache` engine in `src/source/tools_registry.rs` to validate version freshness, enforce 24-hour TTL caching, and surface readiness probes in `ce-ai doctor` and `ce-ai tools status`.

### Key Design Highlights
1. **Embedded Registry & 24h Local Cache**:
   - Embedded defaults (`src/source/tools_registry.rs`) provide pinned minimum versions.
   - `~/.ce-ai/cache/companion-registry.json` maintains a 24-hour TTL local cache.
   - Non-blocking HTTP updates degrade gracefully to `FreshnessStatus::Offline` if offline or timing out (~500ms).

2. **Resilient Exit Code Rules & `--strict` Flag**:
   - By default, missing or outdated tools print informational hints (`doctor-info:`) without failing `ce-ai doctor` (Exit 0).
   - Passing `--strict` (`ce-ai doctor --strict`) enforces zero outdated/missing companion tools (failing with non-zero Exit 1 for CI/CD pipelines).

3. **Skill Suggestions & Self-Update Hints**:
   - `tools status` and `doctor` probe the 4-tier Skill Registry for key reasoning skills (e.g. `sequential-thinking`).
   - Surfaces `ce-ai upgrade` self-update recommendations when behind GitHub releases.

4. **Atomic Write Invariant**:
   - All mutations to `companion-registry.json` use `crate::state::write_atomic` preserving user permissions (`0644`).
