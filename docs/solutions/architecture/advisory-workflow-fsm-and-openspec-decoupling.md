---
schema_version: "2"
module: architecture
date: 2026-10-05
problem_type: architecture
component: workflow-observation
related_components:
  - gate-check
  - openspec-decoupling
severity: medium
title: Advisory Workflow Observation Engine and OpenSpec Decoupling in v2 Phase 3
tags:
  - architecture
  - fsm
  - gate
  - openspec
  - v2-migration
applies_when: When transitioning from authoritative workflow stage gatekeepers to advisory repository observation and optional specifications.
---

# Advisory Workflow Observation Engine and OpenSpec Decoupling in v2 Phase 3

## Problem Statement

In previous versions of `ce-ai`, the system functioned as an authoritative workflow gatekeeper. It stored a rigid 7-stage cursor in `state.json` and blocked tool write operations under `src/**` via `ce-ai gate check` unless formal OpenSpec contracts (`proposal.md`, `spec.md`, `tasks.md`) existed in `openspec/changes/<feature>/`.

As established during boundary reviews with upstream Compound Engineering:
1. **Repository Reality as SSOT:** In Compound Engineering, the git branch, commits, markdown plans, and pull requests *are* the workflow state. A separate state cursor stored in `state.json` invariably drifts from reality and requires redundant reconciliation heuristics.
2. **Artificial Documentation Debt:** Imposing formal specification packages for trivial changes, bug fixes, or exploratory code generates documentation debt and slows execution.
3. **No Semantic Authority:** `ce-ai` must never require artifacts or impose mandatory stages that Compound Engineering itself does not mandate.

## Architectural Solution

In Phase 3 of the CE-AI v2 migration, the workflow architecture transitioned from authoritative enforcement to read-only advisory observation:

1. **Advisory Observation Subsystem (`src/observation/`):**
   - Implemented `ObservableWorkflowState` deriving real-time state directly from repository artifacts:
     - `active_work`: detected via uncommitted git changes and branch context.
     - `plan`: detected via `.md` plan files in `CeDocsConfig::plans_dir`, parsing checkboxes (`- [x]` vs `- [ ]`).
     - `verification`: evaluated via review receipts and test execution markers.
     - `handoff`: detected via handoff receipts or documentation artifacts.
     - `knowledge_capture`: evaluated against recent changes and solution documentation in `CeDocsConfig::solutions_dir`.
     - `openspec`: detected optionally if `openspec/changes/<feature>` exists; returns `None` if inactive.

2. **Decoupled OpenSpec & Non-Blocking Gate (`src/commands/gate.rs`):**
   - Changed default gate mode from `Enforce` to `Observe` (`GateMode::Observe`).
   - Agent tool writes (`Write`, `Edit`) are logged as advisory telemetry to `gate-events.jsonl` without returning blocking exit codes or halting tool execution.
   - Formal OpenSpec contracts are retained as an optional capability for complex or regulated features, rather than a mandatory write prerequisite.

3. **Status Diagnostic Display (`src/commands/workflow.rs`):**
   - `ce-ai workflow status` renders the `Advisory Workflow Capabilities Matrix` directly from on-disk artifacts.
   - `ce-ai workflow status --json` embeds the structured `observable_state` object with 100% backward compatibility for existing consumers.

## Prevention & Validation

- All unit and CLI integration tests pass cleanly (`cargo test`).
- Concept definitions accreted monotonically in `CONCEPTS.md`.
- Validated with `cargo run -- doc lint --strict`.
