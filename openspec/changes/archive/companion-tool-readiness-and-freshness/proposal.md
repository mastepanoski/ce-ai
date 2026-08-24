# OpenSpec Proposal: Companion-Tool Readiness & Version Freshness

- **Change:** `companion-tool-readiness-and-freshness`
- **Issue:** #112
- **Author:** Antigravity AI
- **Date:** 2026-08-22
- **Status:** Proposed

---

## 🎯 1. Problem & Context

Currently, `ce-ai` detects whether companion sidecar binaries (Engram, CodeGraph, Context7, RTK) exist on `PATH` or in default paths, but it does NOT perform **version freshness checks**, **skill recommendations** (e.g. `sequential-thinking`), or **self-update suggestions**.

An AI coding agent session can start with outdated companion tools or missing reasoning skills without any warning. `ce-ai doctor` and `ce-ai tools status` should act as a **proactive readiness engine**, verifying versions, suggesting skill additions, recommending `ce-ai upgrade` when behind GitHub releases, and maintaining 100% offline resilience.

---

## 🚀 2. Proposed Solution

1. **Version Freshness Engine**:
   - Query installed versions of Engram, CodeGraph, Context7, RTK, and `ce-ai`.
   - Compare installed versions against an embedded base registry (`src/source/tools_registry.rs`) backed by a local 24-hour TTL cache (`~/.ce-ai/cache/companion-registry.json`).
   - Categorize status into `ok`, `outdated`, `missing`, or `unknown (offline)`.
2. **Actionable Remediation Hints**:
   - Print exact, copy-pasteable commands for missing or outdated tools (e.g., `ce-ai tools install codegraph`, `ce-ai upgrade`).
3. **Resilient Exit Code Rules & `--strict` Flag**:
   - `missing` tools raise non-zero exit code in `doctor` (default).
   - `outdated` tools emit informational update hints without failing `doctor` (Exit 0), unless `--strict` flag is passed.
4. **Skill Suggestions**:
   - Probe the 4-tier Skill Registry for key skills (e.g., `sequential-thinking`) and surface Context7 MCP wiring status.
5. **Atomic & Non-Destructive Mutations**:
   - Preserve all unmanaged user plugins and skills in `opencode.json` using `crate::state::write_atomic`.

---

## 🔒 3. Scope & Boundaries

### In-Scope
- `ce-ai tools status` and `ce-ai doctor` version freshness & readiness probes.
- Embedded registry fallback + 24-hour local TTL cache + graceful `(offline)` degradation.
- Adding `--strict` flag to `ce-ai doctor`.
- `ce-ai upgrade` self-update notification in `doctor`.
- Skill presence checks in 4-tier Skill Registry.

### Out-of-Scope / Non-Goals
- No auto-installation of background binaries (all installations require explicit user opt-in via `ce-ai tools install`).
- No long blocking network timeouts (network checks timeout at ~500ms and degrade to offline cache).

---

## 🛡️ 4. Risk Matrix & Management

| # | Risk | Likelihood | Impact | Management / Mitigation |
|---|------|-----------|--------|------------------------|
| **R1** | Supply-chain tampering during tool downloads | Medium | High | Verify SHA256 checksums against pinned registry manifests before execution. |
| **R2** | Clobbering user custom plugins/skills in `opencode.json` | Medium | High | Atomic JSON merge via `write_atomic`; preserve all unmanaged keys (Hard Invariant). |
| **R3** | Unintended background tool installation | Low | Medium | `doctor` and `tools status` only report and suggest; installation is explicit opt-in. |
| **R4** | Network failure or rate limits in offline environments | High | Low | 24h TTL cache + embedded fallback; graceful degradation to `(offline)` with zero exit code errors. |
| **R5** | Tool-set desynchronization across 12 agent harnesses | Medium | Medium | Single source of truth registry (`tools_registry.rs`) consumed by `doctor` and `tools`. |

---

## ✅ 5. Success Criteria

- `ce-ai tools status` displays version numbers and status tags (`ok`, `outdated`, `missing`, `unknown (offline)`).
- `ce-ai doctor` reports `outdated` tools as informational hints without failing exit code by default.
- `ce-ai doctor --strict` fails with non-zero exit code when any tool is `missing` or `outdated`.
- 100% unit and CLI integration tests pass with zero warnings (`cargo clippy --all-targets --all-features -- -D warnings`).
