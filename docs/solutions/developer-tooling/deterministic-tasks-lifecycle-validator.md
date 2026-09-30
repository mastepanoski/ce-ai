---
title: "Deterministic Tasks Lifecycle Tail Validation and Auto-Fixing"
category: "developer-tooling"
date: "2026-09-30"
tags:
  - tasks
  - openspec
  - lifecycle-tail
  - validation
  - determinism
  - workflow-governance
components:
  - scripts/validate-tasks-tail.py
  - tests/test_validate_tasks_tail.py
applies_when: "Validating OpenSpec tasks.md files, preparing Stage 4 work, or preventing omitted lifecycle steps in AI sessions"
problem_type: "developer-tooling"
---

# Deterministic Tasks Lifecycle Tail Validation and Auto-Fixing

## Problem
In AI-assisted Compound Engineering workflows, autonomous coding agents frequently jump directly from passing tests to git shipping (commit, PR, merge), completely bypassing post-implementation lifecycle steps:
1. Code simplification (`ce-simplify-code` passes for reuse, quality, and efficiency).
2. Formal multi-agent code review and receipt stamping (`ce-ai workflow review-receipt`).
3. Knowledge capture (`ce-compound` generating `docs/solutions/` artifacts and accretive `CONCEPTS.md` updates).
4. Lifecycle status checkpoints.

Because `ShipReadinessGap` in `ce-ai` is intentionally non-blocking telemetry (observe-only, issue #354), relying exclusively on natural language instructions in `AGENTS.md` suffers from stochastic model decay across long context windows. When `tasks.md` omits these tail items, agents consider their work complete once code-level checkboxes are satisfied.

## Solution Architecture: Script-Driven Deterministic Enforcement

Following the successful precedent of `scripts/validate-concepts.py`, this solution introduces `scripts/validate-tasks-tail.py`: a standalone, zero-dependency Python 3 utility that provides fail-closed validation and automated repair for OpenSpec checklists.

### 1. Heuristic Code Change Detection
The validator inspects whether the target change actually touches executable code:
- Scans `git status --porcelain=v1` for changes in `src/`, `tests/`, `benches/`, or script binaries (`.rs`, `.py`, `.sh`, `.ts`, `.js`, `.go`).
- Inspects sibling `proposal.md`, `design.md`, or `spec.md` files for code path references.
- Exempts pure documentation or configuration modifications (`README.md`, `docs/`, `LICENSE`) where code simplification is inapplicable.

### 2. Lifecycle Tail Verification
For code-touching changes, the script scans `tasks.md` for the three mandatory post-implementation milestones:
- **Simplification**: matches `simplify` or `ce-simplify-code`.
- **Review**: matches `review`, `ce-code-review`, or `review-receipt`.
- **Compound**: matches `compound`, `ce-compound`, `docs/solutions`, or `CONCEPTS.md`.

If any milestone is absent, it prints actionable remediation guidance and exits with code 1.

### 3. Idempotent Auto-Fixing (`--fix`)
When invoked with `--fix`, the script appends a standardized `Lifecycle Tail` unit to `tasks.md`:
```markdown
- [ ] **Unit Final: Lifecycle Completion & Governance Gate**
  - [ ] **Simplification (`ce-simplify-code`)**: Audit changed code for reuse, quality, and efficiency while preserving behavior.
  - [ ] **Revisión formal (`ce-code-review`)**: Run review and record `ce-ai workflow review-receipt`.
  - [ ] **Captura de conocimiento (`ce-compound`)**: Capture learnings in `docs/solutions/<category>/` and monotonically accrete `CONCEPTS.md`.
  - [ ] **Verificación de cierre**: Comply with DoD (`cargo fmt`, `cargo clippy`, `cargo test`, `make e2e`, `doc lint --strict`).
```
Re-running `--fix` detects the existing elements and exits 0 without duplicate appending.

## Key Learnings
1. **Determinism over Prose**: AI agents follow explicit checklist items (`- [ ]`) far more reliably than ambient prompt directives. Embedding the lifecycle tail into the task checklist itself closes the gap between intention and execution.
2. **Fast-Failing Local Scripts**: Standalone scripts in `scripts/` provide immediate sub-50ms feedback in local pre-commit hooks and CI pipelines without requiring binary re-compilation.
