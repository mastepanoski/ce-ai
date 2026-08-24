# OpenSpec Proposal: Real Sidecar Registration for `ce-ai tools install`

- **Change:** `tools-install-real-sidecar-registration`
- **Issue:** #158 (P0)
- **Author:** Antigravity AI
- **Date:** 2026-08-23
- **Status:** Proposed

---

## 🎯 1. Problem Statement

`ce-ai tools install <tool>` currently outputs fake success lines without modifying config files, initializing tools, or probing post-install health. This creates false readiness reports in `doctor`, `status`, and `audit`.

---

## 🚀 2. Proposed Fix

Turn `ce-ai tools install <tool>` into an atomic, capability-verified registration engine:
1. **Atomic Config Injection**: Merges target MCP server definitions into `opencode.json` using `write_atomic`.
2. **User Config Preservation**: Preserves unmanaged user MCP entries and plugins.
3. **Mandatory Post-Probe Gate**: Probes binary/capability readiness post-install. Fails with non-zero exit code (`CeError::Verification`) if probe fails, outputting NO success message.
