# Specification: OpenSpec Tasks Checkbox Desync Reconciliation & Warning

## Requirements

### R1: Task Extraction & Path Tokenization
- WHEN `reconcile_tasks_with_git` or `extract_paths_from_task_text` parses a markdown task item (e.g. `- [ ] 1. Implement foo in `src/commands/workflow.rs``), THEN:
  - It MUST extract all backticked code spans.
  - It MUST extract path-like tokens containing directory separators (`/`) or known file extensions (`.rs`, `.ts`, `.js`, `.json`, `.toml`, `.md`, `.sh`, `.yml`, `.yaml`).
  - Leading/trailing punctuation (`:`, `,`, `(`, `)`, `.`) MUST be cleanly trimmed.
  - Ignored directories (`openspec/`, `.git/`) and lockfiles MUST NOT be treated as actionable task implementation paths.

### R2: Multi-Level Desync Detection Heuristic
- WHEN unchecked tasks (`- [ ]`) in `openspec/changes/<feature>/tasks.md` reference files that exist in the repository diff (either uncommitted in working tree or committed on feature branch vs base):
  - THEN `reconcile_tasks_with_git` MUST flag each matching task in `TaskDesyncReport.desynced_tasks`.
  - The match MUST support exact path matching, directory prefix matching (e.g. `src/commands/`), and path suffix matching (e.g. `workflow.rs`).
- WHEN tasks contain no explicit file paths, BUT `total_tasks > 0`, `completed_tasks == 0`, and non-spec code files (`src/`, `tests/`, `skills/`) are modified:
  - THEN `reconcile_tasks_with_git` MUST set `TaskDesyncReport.is_aggregate_desync = true`.
- WHEN all tasks in `tasks.md` are marked `- [x]`, OR no unchecked tasks correlate with modified files and `completed_tasks > 0`:
  - THEN `has_desync()` MUST return `false`.

### R3: Visible Non-Blocking Warning in `resume_lines`
- WHEN `ce-ai workflow resume` or context re-hydration executes for a feature with detected tasks desync:
  - THEN the output MUST include a visible `! Warning: Tasks desync detected` banner under `tasks progress`.
  - The warning MUST state the number of desynced tasks and preview referenced files.
  - The command MUST proceed to return `Ok(lines)` and exit with code 0 (non-blocking).

### R4: Visible Warning in `status_lines` and `checkpoint_lines`
- WHEN `ce-ai workflow status` runs and tasks desync is detected:
  - THEN `status_lines` MUST output the tasks desync warning.
- WHEN `ce-ai workflow checkpoint` runs and tasks desync is detected:
  - THEN `checkpoint_lines` MUST save the checkpoint to `state.json` successfully (preserving developer sovereignty) AND emit the warning in the returned output lines.

### R5: Non-Fatal Diagnostic Probe in `ce-ai doctor`
- WHEN `ce-ai doctor` executes in a workspace where an active adopted feature change exhibits tasks desync:
  - THEN `doctor` MUST output a notice formatted as:
    `doctor-warn: openspec tasks desync in '<feature>': <summary>`
  - The finding MUST NOT be added to fatal `findings`, and `doctor` MUST exit with code 0 if no other fatal errors exist.

### R6: FSM Monotonic Guard in `maybe_auto_checkpoint`
- WHEN `maybe_auto_checkpoint` evaluates automatic stage transitions for a feature:
  - IF `task_desync.has_desync()` is true AND `completed_tasks < total_tasks`:
  - THEN `maybe_auto_checkpoint` MUST NOT automatically transition the workflow into `Verification` (Stage 5), `KnowledgeCapture` (Stage 6), or `GitShipping` (Stage 7).
  - The stage MUST remain at `WorkTdd` (Stage 4) or `ExecutionPlan` (Stage 3) until tasks are reconciled.

### R7: Performance & Git Isolation
- Git diff probing for tasks reconciliation MUST NOT mutate git state, alter index/working tree, or fail when running offline / without remote connections.
- If `git` is unavailable or errors out, reconciliation MUST degrade gracefully to `None` without failing the parent command.
