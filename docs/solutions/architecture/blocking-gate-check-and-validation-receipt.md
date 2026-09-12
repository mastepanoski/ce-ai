---
title: "Blocking Gate Check & Structured Validation Receipt for Unvetted Writes"
category: "architecture"
date: "2026-09-12"
tags:
  - gate
  - hooks
  - openspec
  - workflow
  - claude
  - doctor
  - status
  - telemetry
  - validation-receipt
components:
  - commands::gate
  - state::state
  - commands::workflow
  - commands::status
  - commands::doctor
applies_when: "Enforcing OpenSpec contract presence during Stage 4 (ce-work) agent writes or auditing validation receipts"
---

# Blocking Gate Check & Structured Validation Receipt for Unvetted Writes

## Context

Following the observe-only telemetry spike deployed in Issue #333 (v1.49.0), Issue #334 elevates `ce-ai gate check` into an active, blocking enforcement gate. In Spec-Driven Development and Compound Engineering, agent tool writes (`Write` and `Edit`) modifying files under `src/**` during Stage 4 (`ce-work`) are forbidden unless the formal OpenSpec contract artifacts (`proposal.md`, `spec.md`, and `tasks.md`) have been authored and verified.

## Architectural Invariants & Core Design

### 1. Claude Code PreToolUse Exit Code 2 Contract
Claude Code `PreToolUse` hooks strictly adhere to the following exit code contract:
- **Exit Code 0**: Execution proceeds normally; stdout and stderr are suppressed from model context.
- **Exit Code 2 (`CeError::Usage`)**: Hook tool execution aborts; stderr is emitted directly into the agent model context, allowing the agent to parse the failure reason and self-remediate.
- **Other Non-Zero Exit Codes**: Stderr is displayed to the user console only, but the tool execution continues unblocked.

Therefore, `ce-ai gate check` maps `GateDecision::Blocked` to `CeError::Usage`, ensuring mechanical enforcement.

### 2. Pure Policy Evaluation Matrix (Zero False Positives)
To prevent blocking legitimate workflows, `evaluate_gate_policy` evaluates a strict priority matrix:
1. **Emergency Kill-Switch**: `CE_AI_DISABLE_GATE_CHECK=1` or `--disabled` bypasses immediately with Exit 0.
2. **Path Filtering**: Writes outside `src/**` (`docs/**`, `openspec/**`, `README.md`, configs) pass with Exit 0.
3. **Adoption Tier Exemption**: Repositories adopted with `--tier minimal` pass with Exit 0.
4. **Direct Entry Point Exemption**: Tasks matching `ce-debug` or `--entry-point ce-debug` pass with Exit 0.
5. **Non-Stage 4 Writes**: Stages 1-3, 5-7 pass with Exit 0.
6. **Edge Case Isolation**: States flagged as `mtime_fallback`, `worktree_uncommitted`, or `stale_cycle_guard` pass with Exit 0 and are recorded under edge-case telemetry.
7. **Stage 4 Contract Verification**: If `proposal.md`, `spec.md`, or `tasks.md` is missing:
   - In `enforce` mode (default): yields `GateDecision::Blocked`, prints remediation feedback to stderr, and exits with Exit 2.
   - In `observe` mode: yields `GateDecision::WouldBlock`, logs telemetry, and exits with Exit 0.

### 3. Structured Validation Receipt
Every gate evaluation generates a `GateReceipt` record containing:
- `timestamp`: ISO-8601 evaluation timestamp.
- `feature`: Active OpenSpec change name.
- `target_path`: File path attempted.
- `decision`: `GateDecision` (`Pass`, `Blocked`, `WouldBlock`, `Undetermined`, `EdgeCase`).
- `stage`: Active workflow stage (e.g. `4`).
- `tier`: Adoption tier (`full` or `minimal`).
- `missing_artifacts`: Array of missing filenames (`proposal.md`, `spec.md`, `tasks.md`).
- `reason`: Descriptive explanation.

Receipts are persisted:
1. Atomically to `openspec/changes/<feature>/.validation.json` if the feature change directory exists.
2. Atomically to `state.json` under `state.gate_receipts[feature]`.
3. Appended to `<config_dir>/gate-events.jsonl` for persistent metrics.

### 4. Visibility in Status and Doctor
- `ce-ai status`: Reports aggregated telemetry metrics (`{total} observed ({blocked} blocked, {would_block} would-block, {pass} pass, ...)`), and surfaces active blocked write warnings (`gate-warn:`).
- `ce-ai doctor`: Evaluates state receipts and emits non-fatal advisory warnings (`doctor-warn: gate-check: write blocked on '<path>' for feature '<feature>'`) without failing general health checks.

### 5. Archive Directory Mtime Fallback Exclusion
In `probe_openspec_context_in` (`src/commands/workflow.rs`), directories named `archive` and any directories starting with `.` are explicitly filtered out during candidate scan. This permanently prevents completed and archived changes in `openspec/changes/archive/` from being mistakenly resolved as the active feature.
