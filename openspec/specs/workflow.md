---
title: "Workflow Lifecycle, FSM & Turn-0 Delivery Engine"
domain: workflow
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: Workflow Lifecycle, FSM & Turn-0 Delivery Engine

## 1. Overview & Architectural Boundaries

The `workflow` subsystem manages the 7-stage Compound Engineering Flywheel (`Ideation` ➔ `OpenSpec` ➔ `Plan` ➔ `Work/TDD` ➔ `Verify` ➔ `Compound` ➔ `Ship`), enforces state-machine transition invariants, and delivers sub-15ms Turn-0 state awareness to AI coding agents.

## 2. Capabilities & Requirements

### R1. 7-Stage Monotonic Progression & Rewind
WHEN advancing workflow stages via `ce-ai workflow checkpoint`  
THEN the FSM MUST reject non-monotonic jumps (e.g. Stage 1 directly to Stage 3).  
WHEN rewinding stages  
THEN the operator MAY transition backwards to earlier stages or reset to Stage 1 at any time.

### R2. Zero-Step Turn-0 Environment Drift Recovery
WHEN a new agent session initializes (Turn-0)  
THEN `ce-ai workflow resume` MUST execute in under 15ms and output `RepoState`, identifying active Git branch, dirty uncommitted files, manifest SHA256 integrity, adoption marker validity, and open OpenSpec tasks.

### R3. OpenSpec Tasks Desync Detection
WHEN files are modified in git or branch history  
THEN the system MUST verify whether tasks in `tasks.md` are synchronized, emitting diagnostic warnings when code is committed but subtasks remain unchecked.

### R4. Branch-Scoped Workflow Isolation
WHEN working in repositories with multiple concurrent feature branches or worktrees  
THEN workflow state MUST be indexed by canonical repository path and branch name (`<root>::<branch>`), preventing cross-branch checkpoint clobbering.

## 3. Data Models & CLI Contracts

- `WorkflowStage` enum: `Ideation`, `OpenSpec`, `Plan`, `Work`, `Verify`, `Compound`, `Ship`.
- `RepoState` struct: serializable representation of live disk reality.
- CLI commands: `ce-ai workflow resume`, `ce-ai workflow status`, `ce-ai workflow checkpoint`.

## 4. Invariants & Operational Boundaries

- Automated stage inference MUST never downgrade or overwrite a higher manual checkpoint (Monotonic Provenance Guard).
- Turn-0 delivery MUST NOT perform network requests or heavy disk traversals.
