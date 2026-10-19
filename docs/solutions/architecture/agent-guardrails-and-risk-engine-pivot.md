---
module: src/commands/guard.rs, src/commands/decisions.rs
date: '2026-10-18'
problem_type: architecture
category: architecture
component: agent_guardrails
severity: medium
symptoms:
  - "Multi-purpose Decision Engine was overburdened with obsolete model routing logic"
  - "Cloud Jev default introduced ~350ms network latency and confusing unauthenticated doctor warnings"
  - "Modern AI harnesses configure models natively, making external CLI model routing redundant"
root_cause: architectural_shift
resolution_type: architectural_addition
tags:
  - guardrails
  - agent-safety
  - risk-engine
  - local-first
  - apple-silicon-mlx
title: "Agent Guardrails and Risk Engine Pivot: Local-First Safety Precedence"
applies_when: "When configuring execution safety guardrails, command risk checks, or evaluating System 1 decision layers for autonomous AI agents."
---

# Agent Guardrails & Risk Engine Pivot: Local-First Safety Precedence

## Problem

The **Pluggable Decision Engine** (`src/decisions/`, `ce-ai decisions`) was originally conceived as a general-purpose probabilistic System 1 layer that handled model routing, skill resolution, readiness assessment, and tool risk checks, defaulting to Jev (TypeSafe AI cloud API).

In the contemporary agentic ecosystem (2026):
1. **Harness Native Model Selection**: Modern coding-agent harnesses (Antigravity, Claude Code, Cursor, OpenCode, Codex) configure models natively via CLI parameters and configuration files. Following the decommissioning of `ce-ai models` in v3.0.0, `ce-ai decisions route` became an isolated advisory command with a hardcoded model catalog (`claude-3-5-*`) that no harness consumed automatically.
2. **Network Latency & Cloud Friction**: Cloud Jev introduced ~316–420 ms of network latency per decision, tracked spend against a monthly budget, and triggered false-positive `doctor-warn: decision-provider-unauthenticated` warnings in `ce-ai doctor` on systems without TypeSafe AI credentials.
3. **True Agent Needs**: What autonomous agent loops, background subagents, and CI pipelines genuinely need is a fast, local-first safety guardrail that intercepts destructive shell commands, credential leaks, and protected git operations before tool execution.

## Solution

In this architectural pivot:
1. **First-Class `ce-ai guard` Command**:
   - Promoted operational guardrails to `ce-ai guard check <tool> <command>` with deterministic safety precedence (<1 ms) for catastrophic commands (`rm -rf /`, force push to main, `.ssh` exfiltration) and 6-dimension semantic risk scoring.
   - Added `ce-ai guard status`, `setup`, and `test` subcommands.
   - Maintained backward compatibility via alias `ce-ai decisions check-risk -> ce-ai guard check`.
2. **Deprecation of Model Routing**:
   - Deprecated `ce-ai decisions route` and `ce-ai decisions compare` with clear notices advising that modern harnesses manage model assignments natively.
3. **Local-First Presets & Clean Doctor Diagnostics**:
   - Updated the `recommended` setup preset to prioritize local execution: auto-detecting Apple Silicon macOS (`aarch64`) for ultra-low latency (<20 ms) `laya-mlx`, or falling back to deterministic local safety rules without requiring cloud API keys.
   - Silenced unauthenticated Jev warnings in `ce-ai doctor` when TypeSafe AI was not explicitly requested.

## Why This Works

Moving from a diffuse "decision engine" to focused "agent guardrails" aligns `ce-ai` with the reality of modern LLM agents: models reasoning natively about tasks, while an ultra-fast, local-first supervisor guards the execution perimeter against accidental destruction.
