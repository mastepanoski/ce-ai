---
module: src/commands/decisions.rs
date: '2026-10-09'
problem_type: architecture
category: architecture
component: cli_subcommands
severity: medium
symptoms:
  - ce-ai models subcommand obsolete in multi-harness architectures
  - asymmetric harness support where models set only worked for opencode
root_cause: architectural_shift
resolution_type: code_fix
tags:
  - cli-decommissioning
  - decisions-routing
  - breaking-change
  - multi-harness
title: "Decommission Models Command and Promote Decisions Route"
applies_when: "When dealing with multi-harness model configuration or task model routing in ce-ai."
---

# Decommission `ce-ai models` Command and Promote `decisions route`

## Problem

In early versions of `ce-ai`, OpenCode was the only supported AI harness. As a result, `ce-ai models` was introduced to configure agent slot model bindings directly in `~/.config/opencode/opencode.json` and persist snapshots in `~/.ce-ai/profiles/`.

As `ce-ai` evolved into a multi-harness platform supporting Claude Code, Cursor, Codex, Pi, and Antigravity, this design suffered from severe architectural issues:
1. **Asymmetry**: `ce-ai models set/list` only modified OpenCode. On other harnesses, model assignments had zero effect or failed silently.
2. **Confusing UX**: Fresh installations defaulted to `(none)`, leaving users confused about active models when each harness manages its own model selection via native CLI flags or configuration files (`CLAUDE_MODEL`, `~/.cursor/`, etc.).
3. **Dead Synchronization Logic**: Bidirectional synchronization between `opencode.json` and `state.json` created unnecessary doctor warnings and sync loops for a single harness.
4. **Subcommand Confusion**: `ce-ai models route <task>` was actually an inference decision recommendation engine, which was conceptually part of the Decisions module rather than model slot management.

## Solution

In `ce-ai` v3.0.0, the `models` subcommand was completely decommissioned:
1. **CLI Decommissioning**: Removed `Commands::Models` from Clap command dispatch in `src/commands/registry.rs`, returning standard exit code 2 (`Usage`) if invoked.
2. **First-Class Routing**: Promoted task-based model recommendations directly to `ce-ai decisions route <task>` in `src/commands/decisions.rs`.
3. **Clean Code Removal**: Deleted obsolete models and profiles modules (`models.rs` and `profiles.rs`) and their respective test suites (-1,400+ LOC).
4. **Doctor & Sync Decoupling**: Removed obsolete `model-assignment-drift` and code-review mid-tier doctor probes and sync reconciliation logic from `src/commands/doctor.rs` and `src/commands/sync.rs`.

## Why This Works

Each modern AI harness handles model selection natively through its own CLI parameters, configuration files, and profile mechanisms. Removing harness-specific model mutating commands eliminates leaky abstractions and lets `ce-ai` focus cleanly on harness orchestration, workflow state machines, and decision frameworks.
