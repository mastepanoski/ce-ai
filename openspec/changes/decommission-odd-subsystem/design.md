# Technical Design: Decommission ODD Subsystem and Align Compound Agile Fast-Path

## Architecture Overview

```
                      ce-ai workflow status
                               │
            ┌──────────────────┴──────────────────┐
            ▼                                     ▼
    Organic / Agile Mode                 Compound / Formal Mode
 (No active OpenSpec change)          (Active openspec/changes/ present)
            │                                     │
   Compound Agile Fast-Path                7-Stage Compound Cycle
   - Direct /ce-work & /ce-debug          - Full 7-stage FSM
   - Lightweight docs/plans/              - Stage 2: OpenSpec contract
   - Advisory 200 LOC ceiling             - Verifiable task checklist
```

## Detailed Component Changes

### 1. `src/commands/workflow.rs`
- **`status_lines_with_mode`**:
  - Update `ExecutionMode::Organic` banner:
    ```text
    == [Workflow Mode: Compound Agile Fast-Path (Direct / docs/plans/)] ==
    Agile Tactical Execution:
      • Problem & Fix         ➔ Direct implementation via /ce-debug or /ce-work
      • Light Planning        ➔ Optional single-file plans in docs/plans/ via /ce-plan
      • Quality & Testing     ➔ Verify tests, clippy, and formatting before commit
      * Scope Boundary        ➔ For architecture or scope > 200 LOC, formalize in openspec/changes/
    ```
- **`handle_graduate`**:
  - Prepend a deprecation notice when invoked:
    `warning: 'ce-ai workflow graduate' and the 'odd/tasks/' directory structure are deprecated. Use 'docs/plans/' for agile tasks, or initialize 'openspec/changes/<feature>/' directly for architecture features.`
  - Maintain functional execution for legacy `odd/tasks/<feature>.md` files so existing fixtures and regression tests pass smoothly.
- **`guide_lines`**:
  - Update the description of Path 1 to **Compound Agile Fast-Path (Tactical & Bug Fixes)**.
  - Remove `ce-ai workflow graduate` from the core commands list.

### 2. `src/commands/gate.rs`
- Update line 656 advice:
  - Mention `openspec/changes/<feature>/` or `docs/plans/` instead of prompting `ce-ai workflow graduate`.

### 3. Documentation & Governance
- Update masterclass, CLI reference, positioning, `README.md`, and `CONCEPTS.md`.
- Document architectural solution in `docs/solutions/architecture/decommission-odd-and-align-compound-fast-path.md`.
