---
title: "Blocking Gate Check & Structured Validation Receipts"
domain: gate
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: Blocking Gate Check & Structured Validation Receipts

## 1. Overview & Architectural Boundaries

The `gate` subsystem enforces boundary gates preventing unauthorized or unvetted write operations during Stage 4 (`ce-work`), providing deterministic blocking and structured audit receipts.

## 2. Capabilities & Requirements

### R1. Exit Code 2 Blocking Enforcement
WHEN an agent invokes a tool write operation (`Write`, `Edit`) targeting `src/**` under Stage 4 (`ce-work`) without formal OpenSpec contracts (`proposal.md`, `spec.md`, `tasks.md`)  
THEN `ce-ai gate check` MUST block execution with Exit Code 2 (`CeError::Usage`) and emit an actionable remediation prompt.

### R2. Zero False-Positive Policy Matrix
WHEN evaluating gate checks  
THEN the gate MUST exempt:
1. `ce-debug` tasks (`--entry-point ce-debug` or task matching `debug`).
2. Repositories adopted with `--tier minimal`.
3. Non-`src/**` target files (`docs/**`, `openspec/**`, `README.md`, configurations).
4. Edge cases flagged as `mtime_fallback`, `worktree_uncommitted`, or `stale_cycle_guard`.

### R3. Structured Validation Receipts
WHEN evaluating a gate check  
THEN the system MUST generate a `GateReceipt` record with execution timestamp, feature name, target path, decision, stage, tier, and missing artifacts, persisting it atomically to `openspec/changes/<feature>/.validation.json` and `state.json`.

## 3. Data Models & CLI Contracts

- `GateReceipt`: `{ timestamp, feature, target_path, decision, stage, tier, missing_artifacts, reason }`.
- `GateDecision`: `Allowed`, `Blocked`, `Exempt`.
- CLI commands: `ce-ai gate check --path <path> [--stage <stage>] [--dry-run]`.

## 4. Invariants & Operational Boundaries

- Emergency kill switch `CE_AI_DISABLE_GATE_CHECK=1` or `--disabled` MUST bypass all blocking checks.
- Gate evaluation MUST complete in under 10ms.
