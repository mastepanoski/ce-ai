---
module: architecture::v2
tags: [architecture, v2, openspec, semantic-authority, workflow, observable-state]
problem_type: architecture
title: "Decoupling Semantic Authority and OpenSpec in CE-AI v2"
applies_when: "When designing workflow observation, evaluating spec requirements, or preventing architectural authority overreach."
date: 2026-10-02
component: architecture
severity: standard
---

# Decoupling Semantic Authority and OpenSpec in CE-AI v2

## Context & Problem Statement

In `ce-ai` v1, OpenSpec was integrated as an authoritative, mandatory "Stage 2" gate within a rigid 7-stage sequential pipeline (`Stage 1 -> Stage 2 -> Stage 3 -> ...`). Any attempt to write code without prior existence of `proposal.md`, `spec.md`, and `tasks.md` was blocked by `ce-ai gate check`.

During an architectural boundary review, the upstream maintainer of Compound Engineering pinpointed a critical flaw:
> *"Requiring proposal/spec/tasks docs before any write grows the pile that then has to be kept trustworthy... The stages also aren't linear in CE. Trivial work skips planning, `ce-debug` has its own path, and `ce-compound` only writes a doc when there's something worth capturing."*

Imposing OpenSpec as an authoritative write gate caused severe friction:
1. **Semantic Authority Overreach:** `ce-ai` invented workflow requirements and gating barriers that upstream Compound Engineering never specified.
2. **False Linearity:** Modeling workflows as numbered sequential stages failed to accommodate trivial code edits, dedicated bug reproduction loops (`ce-debug`), and contingent learning capture.
3. **Documentation Debt Inflation:** Forcing formal specifications on small edits increased cognitive drag and left untrusted, low-value artifacts behind.

## Key Principles & Architectural Shifts

### 1. Foundational Rule of Boundary Respect
> **CE-AI must not require artifacts that Compound Engineering itself does not require.**

`ce-ai` derives its workflow authority strictly from the upstream contracts of the workflows it supports. It cannot invent mandatory gating artifacts without creating an impedance mismatch between human engineers, AI agents, and upstream tools.

### 2. Inviolable Principle: No Semantic Authority
- **What**: `ce-ai` MUST NOT introduce mandatory workflow stages, required artifacts, or transition gates beyond those defined by Compound Engineering contracts.
- **Why**: Protects developers and agents from artificial documentation debt and unnecessary procedural roadblocks.
- **Where**: `docs/architecture/ce-ai-v2-architecture-prd.md`, `CONCEPTS.md`.
- **Learned**: Spec-driven engineering is an empowering technique, not a dogmatic mandate. Optional integrations (like OpenSpec) can be offered, but only activated on-demand.

### 3. Inviolable Principle: Repository Reality Over Mirrored State
- **What**: Repository artifacts (git branch, commits, PR status, plan files under `docs_root/plans/`, run receipts) are the sole authoritative source of truth.
- **Why**: Maintaining an external stage cursor in `state.json` guarantees state drift and necessitates hundreds of lines of fragile heuristic reconciliation.
- **Where**: `src/commands/workflow.rs`, `src/state/state.rs`.
- **Learned**: Upstream CE tools (`ce-handoff`, `ce-work`) inspect the repository directly. `ce-ai` must do the same.

### 4. Decoupling Core Domain vs. Optional Integrations
- **Core Domain**: CE compatibility layer (`src/compat/`), host adapters (`src/harness/`), workflow observation engine (`src/workflow/`), and fleet coordinator (`src/fleet/`).
- **Optional Integrations**: OpenSpec operates as an external, opt-in integration via `ce-ai spec` commands. When users or agents tackle high-complexity or regulated features, they explicitly invoke OpenSpec; when making trivial edits or debugging, OpenSpec is completely bypassed without friction.

### 5. Observable Workflow Capabilities Matrix over Linear Stage Cursors
- **What**: Abandon the scalar `current_stage: u8` cursor in favor of an artifact-derived capabilities matrix:
  ```rust
  pub struct ObservableWorkflowState {
      pub active_work: bool,
      pub plan: Option<PlanSummary>,
      pub verification: VerificationStatus,
      pub handoff: Option<HandoffArtifact>,
      pub knowledge_capture: KnowledgeCaptureStatus,
      pub openspec: Option<OpenSpecState>,
  }
  ```
- **Why**: Replaces the query *"What stage are we in?"* with *"What do we objectively know about the workflow state?"*. This naturally handles skips, non-linear jumps, and exploratory work without invalidating the FSM model.

## Prevention & Verification

- **Concept Integrity**: New concepts (*No Semantic Authority*, *Repository Reality Over Mirrored State*, *Observable Workflow State*, *Optional Integration Autonomy*) are permanently accreted in `CONCEPTS.md` and guarded by `scripts/validate-concepts.py`.
- **Linting & Hygiene**: Validated with `cargo run -- doc lint --strict`.
