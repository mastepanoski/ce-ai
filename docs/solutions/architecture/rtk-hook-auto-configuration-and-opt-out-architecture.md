---
title: "RTK Hook Auto-Configuration, Opt-Out Lifecycle, and Silent Output Mitigation"
category: "architecture"
date: "2026-09-06"
tags:
  - harness
  - rtk
  - hooks
  - claude
  - cursor
  - copilot
  - codex
  - audit
  - doctor
components:
  - harness::rtk
  - commands::install
  - commands::init_prj
  - commands::uninstall
  - commands::audit
  - commands::doctor
applies_when: "Configuring CLI compression pre-processors, auditing agent environment hooks, or addressing stdout filtering issues"
---

# RTK Hook Auto-Configuration, Opt-Out Lifecycle, and Silent Output Mitigation

## Context

`rtk` (CLI output token reduction engine) intercepts agent tool execution via CLI execution hooks (`PreToolUse`, command rewrites) rather than registering as an MCP server. Prior to Issue #308, `ce-ai` had no auto-configuration mechanism for `rtk`:
- It surfaced only as an `Info`-level suggestion in `ce-ai audit` (`CliCompressionDetector`) and `ce-ai tools status`.
- Users had to discover and wire it manually (`rtk init --global`).
- Only a subset of `ce-ai` harnesses are officially supported by RTK upstream (`Claude Code`, `Cursor`, `GitHub Copilot`, `Codex CLI`).
- An observed risk: aggressive or failing RTK command filters could silently swallow stdout with exit code 0 (e.g. `rtk gh issue view <n> --comments` producing 0 bytes), which in automated or CI/CD pipelines is more dangerous than a hard error.

## Solution Architecture

Release v1.42.0 introduces end-to-end lifecycle management for RTK hooks with resilient fallbacks and explicit opt-outs:

### 1. Dedicated RTK Adapter & Support Matrix (`src/harness/rtk.rs`)
- `is_rtk_supported(kind)`: Exclusively returns `true` for `Claude`, `Cursor`, `Copilot`, and `Codex`. Returns `false` for unsupported harnesses (`Opencode`, `Pi`, `Custom`, `Deepseek`, `Grok`, `Kimi`, `Agy`, `Fx`).
- `configure_rtk_hook`: Ensures target configuration directories (and upstream prerequisite `$HOME/.claude`) exist, passes isolated `HOME`, executes `rtk init -g`, and emits diagnostic feedback.
- `unconfigure_rtk_hook`: Symmetrically removes injected hooks on `ce-ai uninstall`.
- `is_rtk_hook_configured`: Validates on-disk presence of injected hooks in `settings.json`, `hooks.json`, `rtk-rewrite.json`, or `RTK.md`.

### 2. Granular and Blanket Opt-Out
- CLI flags: `--skip-rtk` (bypasses RTK hook injection) and `--skip-companions` (bypasses both RTK and companion MCP servers) on `ce-ai install` and `ce-ai init-prj`.
- Environment variables: `CE_AI_SKIP_RTK=1` and `CE_AI_SKIP_COMPANIONS=1` allow zero-touch opt-outs in CI/CD scripts and automated workflows.

### 3. Non-Fatal Executable Resilience & Unsupported Harness Safety
- If the `rtk` binary is absent from `PATH`, `install` and `init-prj` emit a guidance notice and proceed successfully (`Ok(())`).
- For unsupported harnesses, RTK configuration is an explicit, logged no-op with zero failure potential.

### 4. Audit Escalation & Doctor Diagnostics
- `ce-ai audit`: Refactored `CliCompressionDetector` to report `AuditStatus::Warn` for supported harnesses lacking RTK hooks, while maintaining `AuditStatus::Info` for unsupported harnesses.
- `ce-ai doctor`: Inspects RTK hook status across installed supported harnesses, warns on missing hooks, and explicitly documents the potential stdout alteration risk on wrapped commands.
