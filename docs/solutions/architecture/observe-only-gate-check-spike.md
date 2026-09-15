---
title: "Observe-Only Gate Check Spike: Measuring Agent Tool Writes in ce-work Without OpenSpec"
category: "architecture"
date: "2026-09-09"
tags:
  - gate
  - hooks
  - openspec
  - workflow
  - claude
  - doctor
  - status
  - telemetry
components:
  - commands::gate
  - harness::claude
  - commands::init_prj
  - commands::deinit_prj
  - commands::status
  - commands::doctor
applies_when: "Auditing or observing agent tool execution compliance against OpenSpec workflow stages without blocking developer execution"
problem_type: architectural_refactor
---

# Observe-Only Gate Check Spike: Measuring Agent Tool Writes in ce-work Without OpenSpec

## Context

A fundamental tenant of Compound Engineering and Spec-Driven Development is that production code under `src/**` should only be written during Stage 4 (`ce-work`) when formal specifications (`proposal.md`, `spec.md`, `tasks.md`) have been approved. However, abruptly blocking agent tool writes via hooks creates severe user friction and risks false positives from edge cases (e.g. multi-cycle tasks, uncommitted worktree changes, non-git directories).

Issue #333 established an observe-only spike mechanism to measure empirical agent behavior without blocking writes.

## Invariants & Design Principles

1. **Strictly Observe-Only (Zero Write Blocking)**:
   The spike never aborts tool operations or alters exit codes. `ce-ai gate check` always exits with code `0`, ensuring zero disruption to developers or agents.

2. **Single Source of Truth Checkpoint Inspection**:
   `gate check` reads the active workflow checkpoint stage directly from `state.json` via `state.current_workflow_for_branch(&repo_root, branch)`. It never runs repo heuristics to re-infer the stage, preventing stage inference races.

3. **Isolated Edge Case Buckets**:
   Ambiguous or non-standard states are isolated into dedicated edge-case buckets rather than conflated with `would_block` or `pass`:
   - `mtime_fallback`: feature inferred via mtime without a git branch.
   - `worktree_uncommitted`: uncommitted changes detected in `openspec/changes/<feature>/`.
   - `stale_cycle_guard`: multi-cycle same-branch transition guard task flag.

4. **Emergency Kill-Switch**:
   Short-circuits immediately before any state or filesystem access when `CE_AI_DISABLE_GATE_CHECK=1`, `CE_AI_GATE_CHECK_DISABLED=1`, or `--disabled` is passed.

5. **Claude Code `PreToolUse` Hook Lifecycle**:
   Auto-configured in `.claude/settings.json` matching `Write|Edit` executing `ce-ai gate check`. Auto-injected on `ce-ai init-prj` and surgically removed on `ce-ai deinit-prj`.

6. **Privacy-Preserving Telemetry**:
   Logs non-sensitive metadata (timestamp, harness, tool, target path, branch, stage, feature, decision, reason) to `<config_dir>/gate-events.jsonl` without capturing diffs or file contents. `ce-ai status` and `ce-ai doctor` report aggregated observation counts.
