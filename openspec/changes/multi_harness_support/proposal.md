# OpenSpec Proposal: Multi-Harness Support for `ce-ai`

**Change Identifier:** `multi_harness_support`  
**Tracking Issue:** GitHub Issue #1 & Issue #4  
**Author:** AI Agent & Compound Engineering Framework  
**Date:** 2026-08-21  

---

## 1. Problem Statement

`ce-ai` v0.2.0 manages Compound Engineering plugin installation and model role assignments exclusively for the **OpenCode** harness. However, developers work across a variety of AI coding environments including Claude Code, Pi, Cursor, Copilot, Codex, Grok, Kimi, AGY, DeepSeek, and custom shell harnesses like `fx.sh`.

Without multi-harness support, developers must manually copy skills, rules, and configuration files into non-OpenCode harnesses, creating configuration drift, security risks, and breaking model role synchronization.

## 2. Proposed Solution

Extend `ce-ai` with a modular **Harness Adapter System**:
1. Statically typed `HarnessKind` enum supporting 11 native harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`) plus `--harness custom`.
2. Structured JSON Merger for JSON-based harnesses and Markdown Rule Ingestion Adapter (`<!-- CE-AI MANAGED BLOCK -->`) for instruction-based harnesses.
3. Interactive and flag-based fallback mode (`--harness custom --plugins-dir <path>`).
4. Centralized state synchronization (`ce-ai models set`, `ce-ai sync --all`, `ce-ai status`).

## 3. Success Criteria

- Clean installation, status reporting, syncing, model setting, and uninstallation across all 12 harness targets.
- 100% preservation of user-defined custom configuration keys and unmanaged instruction text.
- Atomic writes (`write_atomic`), SHA256 file manifest tracking, and timestamped backups for all harnesses.
- 100% green unit, integration, and containerized Docker E2E test suites across Linux, macOS, and Windows.
