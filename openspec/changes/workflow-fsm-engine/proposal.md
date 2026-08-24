# Proposal: Real 7-Stage Workflow FSM Engine & Context Recovery

## Problem Statement
Issue #156 notes that `ce-ai workflow` currently abuses `State.last_update_check` to store an unvalidated checkpoint string (`phase | task | timestamp`). It enforces no stage transitions, validates no stages, and performs no context recovery upon `workflow resume`.

## Proposed Solution (Option B)
Implement a strongly-typed 7-stage Finite State Machine (FSM) engine:
1. `WorkflowStage` enum:
   - Stage 1: `Ideation` (`ce-brainstorm`)
   - Stage 2: `OpenSpec` (`openspec/changes/<feature>/`)
   - Stage 3: `ExecutionPlan` (`ce-plan`)
   - Stage 4: `WorkTdd` (`ce-work` / `ce-debug`)
   - Stage 5: `Verification` (`cargo test` / `make e2e`)
   - Stage 6: `KnowledgeCapture` (`ce-compound`)
   - Stage 7: `GitShipping` (`ce-commit-push-pr`)
2. Transition validation rules enforcing legal stage transitions (sequential forward progress, rewind to previous stage, or reset to Stage 1).
3. Dedicated `WorkflowState` storage in `state.json` (`state.workflow`).
4. Context recovery in `workflow resume`: inspecting active OpenSpec change paths, stage status, tasks, and rendering structured context guidance.
5. `--json` output option for `ce-ai workflow status/checkpoint/resume`.

## Acceptance Criteria
- `WorkflowStage` enum parses string identifiers cleanly (`ideation`, `openspec`, `plan`, `work`, `verify`, `compound`, `ship`).
- Illegal stage transitions (e.g. Stage 1 Ideation directly to Stage 7 GitShipping) return `CeError::Usage` (exit code 2).
- Checkpoints persist cleanly in `state.workflow` in `state.json`.
- `workflow resume` re-hydrates OpenSpec change artifacts and tasks.
- Parity between CLI and TUI dashboard.
