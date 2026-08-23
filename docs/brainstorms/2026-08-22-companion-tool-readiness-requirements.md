# Companion-Tool Readiness & Version Freshness Requirements

- **Date:** 2026-08-22
- **Issue:** #112 (Companion-tool readiness — version freshness, skill suggestions, and ce-setup integration)
- **Status:** Approved (Brainstorm Completed)
- **Scope Tier:** Standard

---

## 🎯 1. Overview & Problem Statement

`ce-ai` currently detects whether companion tool binaries (Engram, CodeGraph, Context7, RTK) are present on `PATH` or in their standard config directories, but it lacks **version freshness validation**, **skill suggestions** (e.g. `sequential-thinking`), and **self-update recommendations**.

Without version checking, an AI agent session can run against severely outdated sidecars or missing reasoning skills without warning.

This feature enhances `ce-ai doctor` and `ce-ai tools status` to act as a **proactive environment readiness engine**, comparing installed versions against expected minimums, suggesting skill additions, offering self-upgrade hints for `ce-ai`, and maintaining 100% offline resilience.

---

## 🚀 2. Goals & Success Criteria

1. **Version Freshness Probes**: `ce-ai doctor` and `ce-ai tools status` compare installed vs expected versions for Engram, CodeGraph, Context7, RTK, and `ce-ai` itself.
2. **Distinct Status Categories**:
   - `ok`: Tool installed and up to date.
   - `outdated`: Tool installed but older than recommended minimum.
   - `missing`: Tool not installed.
   - `unknown (offline)`: Network unavailable, fallback to local cache/embedded registry.
3. **Actionable Remediation Hints**: Every `outdated` or `missing` tool prints the exact copy-paste command to install or upgrade (e.g., `ce-ai tools install engram`, `ce-ai upgrade`).
4. **Resilient Exit Code Rules**:
   - `missing` tools raise non-zero exit code in `doctor` (default behavior).
   - `outdated` tools emit informational hints without failing `doctor` (Exit 0), unless `--strict` is passed.
5. **Skill Suggestions**: `ce-ai tools status` and `doctor` recommend installing key reasoning skills (e.g. `sequential-thinking` in Skill Registry) and verify Context7 MCP configuration.
6. **Zero User-Config Loss**: Wiring MCP sidecars into `opencode.json` or other harness configs uses atomic JSON merging (`write_atomic`), preserving all unmanaged user plugins and skills.

---

## 🔒 3. Scope Boundaries & Non-Goals

### In Scope
- Companion tool version detection via CLI execution (`engram --version`, `codegraph --version`, `rtk --version`, `ce-ai --version`).
- Embedded registry fallback (`src/source/tools_registry.rs`) paired with a 24-hour local TTL cache (`~/.ce-ai/cache/companion-registry.json`).
- Adding `--strict` flag to `ce-ai doctor` to enforce zero outdated tools in CI/CD pipelines.
- Self-update detection for `ce-ai` against latest GitHub releases.
- Skill presence probes in the 4-tier Skill Registry.

### Out of Scope / Non-Goals
- **No Background Auto-Installs**: `ce-ai` will NEVER install or upgrade binaries automatically without explicit user commands (`ce-ai tools install <tool>`).
- **No Blocking Network Calls**: Network requests for version manifests must use strict timeouts (~500ms) and degrade gracefully to offline cache.

---

## 🛡️ 4. Risk Analysis & Management

| # | Risk | Severity | Mitigation |
|---|------|----------|------------|
| **R1** | Supply-chain tampering during tool fetching | High | Verify SHA256 checksums against pinned manifests before execution. |
| **R2** | Clobbering user custom plugins in harness configs | High | Atomic JSON merge via `write_atomic` preserving unmanaged keys. |
| **R3** | Unintended background mutations | Medium | `doctor` and `tools status` only report and suggest; installation remains explicit. |
| **R4** | Network latency or offline failure | Medium | Local TTL cache (24h) + embedded fallback; graceful degradation to `(offline)`. |

---

## 📋 5. User-Facing CLI Ergonomics

### A. `ce-ai tools status` Output Format
```text
== [Companion Tools & Memory Sidecars Readiness Status] ==
  ✓ engram (Engram Memory Server)        v1.2.0 (ok)
  ⚠️ codegraph (CodeGraph Indexer)       v0.4.1 (outdated -> v0.5.0 available; run 'ce-ai tools install codegraph')
  ❌ context7 (Tech Specs Provider)      not found (suggested: 'ce-ai tools install context7')
  ✓ rtk (Token Reduction Engine)         v0.2.1 (ok)

== [Skill Registry Suggestions] ==
  ⚠️ sequential-thinking                 missing in Skill Registry (suggested: 'ce-ai skills resolve sequential-thinking')

== [Orchestrator Readiness] ==
  ✓ ce-ai CLI                            v1.6.3 (ok)
```

### B. `ce-ai doctor` Extensions
```text
doctor-info: engram v1.2.0 (ok)
doctor-info: codegraph v0.4.1 (outdated, v0.5.0 available; run 'ce-ai tools install codegraph')
doctor-info: sequential-thinking skill missing (run 'ce-ai skills resolve sequential-thinking')
```

---

## 🔄 6. OpenSpec Handoff & Next Steps

This requirements document is frozen in `docs/brainstorms/2026-08-22-companion-tool-readiness-requirements.md`.

Next phase: **Stage 2 (OpenSpec Definition)** in `openspec/changes/companion-tool-readiness-and-freshness/`:
- `proposal.md`
- `exploration.md`
- `design.md`
- `spec.md`
