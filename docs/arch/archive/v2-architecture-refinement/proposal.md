# Proposal: CE-AI v2 Architecture Refinement — Decoupling Semantic Authority & OpenSpec

## 1. Problem Statement

Following detailed review of the CE-AI v2 architecture draft and the upstream Compound Engineering maintainer's feedback, a key architectural refinement has been identified:

1. **Semantic Authority Overreach:** In v1, `ce-ai` imposed its own mandatory workflow semantics on top of Compound Engineering. The primary symptom was the hard write gate enforcing OpenSpec packages (`proposal.md`, `spec.md`, `tasks.md`) before permitting any code modification. As the upstream maintainer observed:
   > *"Requiring proposal/spec/tasks docs before any write grows the pile that then has to be kept trustworthy."*
2. **False Linearity in State Modeling:** The maintainer emphasized:
   > *"The stages also aren't linear in CE. Trivial work skips planning, ce-debug has its own path, and ce-compound only writes a doc when there's something worth capturing."*
   Modeling workflow as a rigid sequence of numbered stages (`Stage 1 -> Stage 2 -> Stage 3 -> ...`) misrepresents Compound Engineering and forces artificial synchronization.
3. **Core Domain Conflation:** In v1, OpenSpec was treated as part of the core domain model of `ce-ai`. OpenSpec should not be eliminated, but it must be stripped of its authority over the workflow and extracted from the core domain model into an **optional integration**.

The guiding architectural rule for CE-AI v2 is:
> **CE-AI must not require artifacts that Compound Engineering itself does not require.**

## 2. In-Scope & Objectives

- **Eliminate Semantic Authority:** Formally establish the principle that `ce-ai` never introduces mandatory stages, artifacts, or gates beyond what Compound Engineering defines.
- **Decouple OpenSpec from Core Domain:** Reposition OpenSpec as an optional, opt-in integration that sits outside the core domain model (`CE Compatibility`, `Host Adapters`, `Workflow Observation`, `Environment Coordination`).
- **Transition from Numbered Stages to Observable Capabilities:** Replace scalar stage cursors (`current_stage: StageN`) with an **Observable Workflow State / Capabilities Matrix** derived purely from repository artifacts (`active_work`, `plan_status`, `verification_status`, `handoff_status`, `knowledge_status`, `openspec_status`).
- **Codify Architectural Invariants in PRD:** Add explicit foundational principles to `docs/architecture/ce-ai-v2-architecture-prd.md`:
  1. *No Semantic Authority*
  2. *Repository Reality Over Mirrored State*
- **Update Migration Roadmap:** Reflect the decoupled domain model and capabilities-based observation engine across the phased v2 roadmap in `docs/plans/2026-10-01-ce-ai-v2-architectural-migration-plan.md`.
- **Monotonic Concept Accretion:** Accrete domain terms in `CONCEPTS.md`.

## 3. Out of Scope

- Removing OpenSpec completely (it remains an optional, opt-in tool for complex architectural initiatives).
- Breaking existing v1.x CLI commands during this architectural specification phase.

## 4. Risk Evaluation & Mitigation

| Risk | Impact | Mitigation Strategy |
| :--- | :--- | :--- |
| Users lose spec-driven planning for complex tasks | Medium | Keep OpenSpec fully functional as an opt-in integration (`ce-ai spec` / `openspec/`) when explicitly chosen by the user. |
| Existing tools expect linear stage numbering | Low | Provide backward-compatible mapping from observable capability states to legacy stage numbers during the v1.x deprecation window. |
| Conceptual ambiguity between CE and CE-AI | High | Enforce strict separation of concerns: CE owns workflow semantics; AI hosts own execution; CE-AI coordinates and observes. |
