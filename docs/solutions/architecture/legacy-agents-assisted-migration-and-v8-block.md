---
module: src/commands/init_prj.rs, src/commands/doctor.rs
date: '2026-10-21'
problem_type: architecture
category: architecture
component: project_adoption
severity: medium
symptoms:
  - "Repositories adopted with pre-decoupling CE-AI had contradictory, rigid instructions in AGENTS.md"
  - "Agents froze or over-prescribed OpenSpec change packages for trivial bug fixes and 10-line edits"
  - "Legacy AGENTS.md files still referenced deprecated commands (ce-ai models, ce-ai decisions)"
root_cause: architectural_evolution
resolution_type: assisted_migration_engine
tags:
  - project-adoption
  - agents-md
  - migration
  - openspec-decoupling
  - v8-block
title: "Assisted AGENTS.md Legacy Migration Engine and v8 Adoption Block"
applies_when: "When adopting, modernizing, or repairing host projects with historical pre-decoupling CE-AI instructions in AGENTS.md."
---

# Assisted AGENTS.md Legacy Migration Engine and v8 Adoption Block

## Problem
In early versions of `ce-ai`, project adoption (`ce-ai init-prj`) injected managed workflow blocks into `AGENTS.md` while leaving pre-existing, manually authored directives outside the managed block intact. In older templates, these directives mandated rigid write-blocking rules (such as historical Invariant 6: "NO code changes without formal spec", mandatory 7-stage ASCII FSM diagrams, and deprecated CLI commands like `ce-ai models` and `ce-ai decisions`).

Following the architectural decoupling of OpenSpec (v2.0) and the adoption of the Compound Agile Fast-Path (v3.5), projects that had these clauses hardcoded outside the managed block presented conflicting instructions to AI coding agents (Claude, Gemini, Codex). Agents would frequently freeze or demand full multi-document specification packages (`openspec/changes/<feature>/`) even for trivial bug fixes.

## Solution
Implemented an assisted, non-destructive migration engine in `ce-ai init-prj` (`--migrate [path]` and `--all`) and integrated diagnostic linting and automated repair into `ce-ai doctor --fix`:

1. **Surgical Pattern Scanning & Modernization**:
   - Isolates unmanaged sections of `AGENTS.md` outside the managed block markers (`<!-- ce-ai:block begin ... -->`).
   - Identifies and replaces 7 historical clauses (`PAT-INV-6`, `PAT-CONSTRAINT-OPENSPEC`, `PAT-FSM-7`, `PAT-STAGE-2`, `PAT-CLI-MODELS`, `PAT-CLI-DECISIONS`, `PAT-FSM-READY`) with decoupled Compound Engineering instructions and proactive 200 LOC triage rules.
   - Strictly preserves 100% of host project domain rules, custom linters, and security policies.

2. **Safety Guarantees**:
   - Generates an automatic timestamp-free backup `AGENTS.md.bak` prior to writing disk.
   - Supports `--dry-run` to preview planned modifications and unified diffs with zero disk writes.

3. **Adoption Block Upgrade (v=8)**:
   - Increments `BLOCK_VERSION` from 7 to 8 across project templates.
   - Aligns `AdoptionTier::Full` with decoupled OpenSpec guidance.

4. **Doctor Diagnostic & Auto-Repair**:
   - `ce-ai doctor` inspects adopted project `AGENTS.md` files for unmanaged legacy directives and reports advisory warnings.
   - `ce-ai doctor --fix` automatically modernizes unmanaged directives with backup creation.
