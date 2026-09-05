---
title: "Workflow FSM Auto-Checkpoint Lifecycle, Monotonic Provenance Guard, and Multi-Harness Integration"
category: "architecture"
date: "2026-09-05"
tags:
  - workflow-fsm
  - auto-checkpoint
  - provenance-guard
  - lifecycle-hooks
  - multi-harness
components:
  - workflow
  - state
  - harness
  - init-prj
  - sync
applies_when: "Extending workflow stage inference, lifecycle hooks across harnesses, or branch-scoped state progression in ce-ai"
---

# Workflow FSM Auto-Checkpoint Lifecycle, Monotonic Provenance Guard, and Multi-Harness Integration

## Context

In `ce-ai`, the 7-stage Compound Engineering workflow (`Ideation` ➔ `OpenSpec` ➔ `Plan` ➔ `WorkTdd` ➔ `Verification` ➔ `KnowledgeCapture` ➔ `GitShipping`) previously depended entirely on explicit CLI invocations (`ce-ai workflow checkpoint <stage>`). However, canonical compound-engineering skills do not call `ce-ai workflow checkpoint`. As a result, the workflow state remained stalled at Stage 1 even as agents implemented features, passed tests, and completed work (Issue #296).

## Solution Architecture

Release v1.40.0 implements autonomous Workflow FSM progression driven by filesystem evidence and harness lifecycle hooks.

### 1. Stage Inference Engine (`infer_stage_from_repo`)

Rather than relying on agents to execute CLI commands, `ce-ai` inspects unambiguous observable repository artifacts:
- **Stage 7 (GitShipping)**: `gh pr view --json state` confirms open/merged PR, or git commits ahead of remote tracking branch.
- **Stage 6 (KnowledgeCapture)**: Changes or additions under `docs/solutions/` or modifications to `CONCEPTS.md`.
- **Stage 5 (Verification)**: Clean git working tree with test files updated, or target compilation / verification runs.
- **Stage 4 (WorkTdd)**: Branch names starting with `fix/` or `feat/` bypass Stages 1–3 directly to Stage 4. Alternatively, completed tasks in `openspec/changes/<feature>/tasks.md` indicate active implementation.
- **Stage 3 (Plan)**: Presence of `tasks.md` under `openspec/changes/<feature>/`.
- **Stage 2 (OpenSpec)**: Presence of `proposal.md`, `exploration.md`, `design.md`, or `spec.md` under `openspec/changes/<feature>/`.
- **Stage 1 (Ideation)**: Notes under `docs/brainstorms/` or `docs/ideation/`.

### 2. Monotonic Provenance Guard

To ensure automated inference never overrides human intent or regresses state:
- `WorkflowSource::Manual`: Explicit operator checkpoints (`ce-ai workflow checkpoint`).
- `WorkflowSource::Inferred`: Automated filesystem inferences.
- **Rule**: An inferred checkpoint is rejected if a manual checkpoint exists at an equal or higher stage (`inferred_stage <= current_state.stage && current_state.source == WorkflowSource::Manual`).

### 3. Branch-Scoped Workflow Isolation

State is indexed by `<canonical_root>::<branch>` (sanitizing branch names to prevent path traversal):
- Branch-specific workflows evolve independently without cross-branch clobbering.
- Hierarchical fallback: queries check `<canonical_root>::<branch>`, then `<canonical_root>`, then global `workflow`.

### 4. Granularity: Turn-End and Pre-Compaction Hooks

To avoid performance overhead and file churn, hooks are wired exclusively to:
- **Turn-End**: `Stop` (Claude, Codex, Agy), `stop` (Cursor), `session.idle` (OpenCode), `agent_end` (Pi), `postToolUse` (Copilot).
- **Pre-Compaction**: `PreCompact` (Claude, Codex), `session_before_compact` (Pi).
- High-frequency per-tool-call execution is explicitly avoided.

### 5. Opt-in Gating and Transitory Git Safety

- **Adoption Gate**: Auto-checkpointing runs only on adopted projects (`state.is_project_adopted(repo_root)`).
- **Opt-Out**: Users can disable auto-checkpointing in adopted repos via `auto_checkpoint: false` in `state.json`.
- **Transitory Git State**: Checkpointing is suppressed during active rebases, cherry-picks, or merges (`.git/rebase-merge`, `MERGE_HEAD`, etc.).
