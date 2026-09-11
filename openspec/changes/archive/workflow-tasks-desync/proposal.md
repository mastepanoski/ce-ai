# Proposal: OpenSpec Tasks Checkbox Desync Reconciliation & Warning

## Problem Statement

In `ce-ai`, the workflow Finite State Machine (`infer_stage_from_repo` in [`src/commands/workflow.rs`](file:///Users/mastepanoski/projects/web/ai/ce-ai/src/commands/workflow.rs#L725-L807)) derives progress across Stages 3–7 by inspecting checkbox completion in `openspec/changes/<feature>/tasks.md` (`completed_tasks` vs `total_tasks`).

However:
1. `ce-ai` itself never modifies or writes to `tasks.md`; marking `- [x]` is left entirely to the AI agent or developer running `ce-work`.
2. If an agent executes substantial implementation work (modifying source files, adding tests, committing changes) but neglects to check off tasks in `tasks.md`, the FSM remains pinned at `ExecutionPlan` (Stage 3) if `completed_tasks == 0` or early `WorkTdd` (Stage 4), repeatedly outputting:
   ```text
   == [Context Re-hydration: <feature>] ==
     spec location: ...
     has proposal: true
     has spec: true
     has tasks: true
     tasks progress: 0/N completed ([x])
   ```
3. This desync is completely silent. Neither `resume_lines`, `status_lines`, nor `ce-ai doctor` emit any warning indicating that significant code modifications exist in files implicated by the unchecked tasks. Consequently, context re-hydration continuously re-triggers Stage 3 planning as if no work had ever started.

Issue #313 (Part 1) requests establishing a reconciliation mechanism that detects when work has occurred while `tasks.md` remains desynced, surfacing visible diagnostics without breaking non-interactive operations.

## Evaluation of Approaches & Architectural Decisions

### 1. Reconciliation Detection Heuristic
We evaluate three tiers of detection accuracy:
- **Tier 1: Explicit Path Token Extraction (Deterministic)**: Parse each unchecked task item (`- [ ]`) in `tasks.md` for backticked paths (e.g. `` `src/commands/workflow.rs` ``) or path-like tokens containing directory separators (`/`) and common source extensions (`.rs`, `.ts`, `.js`, `.toml`, `.json`, `.md`, `.sh`). Compare these extracted paths against modified files in the repository diff (both uncommitted working tree changes and feature branch commits since diverging from the base branch).
- **Tier 2: Module / Domain Concept Matching (Structural)**: If a task mentions module identifiers or architectural boundaries (e.g. `workflow`, `install`, `sync`, `doctor`), match them against touched paths under `src/commands/<module>.rs`, `src/<module>/`, or `tests/<module>.rs`.
- **Tier 3: Aggregate Work-vs-Zero-Progress Heuristic (Fallback)**: When `total_tasks > 0` and `completed_tasks == 0`, but the repository exhibits substantial modifications to project code (`src/`, `tests/`, `skills/`) outside of `openspec/`, flag a high-confidence desync even if tasks were written without explicit file citations.

**Adopted Heuristic**: A composite multi-level reconciliation model:
1. Primary: Extract file and directory targets from unchecked task descriptions and test against git diff files (exact match or path prefix match).
2. Secondary Fallback: If no explicit paths are detected in task descriptions, evaluate whether `completed_tasks == 0` while non-spec source files have been changed on the branch/working tree.
3. Quantify the desync: produce a structured `TaskDesyncReport` indicating the number of desynced tasks, matching touched files, and severity.

### 2. Warning Visibility vs Hard Blocking
The issue evaluates whether `ce-ai` should merely warn or strictly block/prompt stage-advancing checkpoints:
- **Option A: Non-blocking Visible Warnings Everywhere (Adopted as Primary)**:
  - Add explicit, actionable warning banners in `resume_lines` (in the context re-hydration block), `status_lines`, and `checkpoint_lines`.
  - Add a non-fatal `doctor-warn: openspec tasks desync: ...` probe in `ce-ai doctor` (exits 0, does not add to fatal `findings`).
  - **Rationale**: Preserves non-interactive CLI compatibility, CI automation, and developer sovereignty. Never breaks user workflows unexpectedly.
- **Option B: Hard Blocking / Interactive Confirmation (Evaluated & Scoped)**:
  - For **manual checkpoints** (`ce-ai workflow checkpoint`): Do NOT block; always allow the human/agent to set their desired stage, but print a prominent warning if desync is detected.
  - For **automated FSM inference** (`maybe_auto_checkpoint`): Prevent automatic monotonic progression past `WorkTdd` into `Verification`, `KnowledgeCapture`, or `GitShipping` if `tasks.md` has 0 tasks completed while code is modified. This stops the FSM from prematurely jumping to completion without validated task signoff, while avoiding runtime prompts or crashes.

## In-Scope Boundaries

- **Reconciliation Engine (`src/commands/workflow.rs`)**:
  - Implement task parsing and path extraction from markdown checkboxes in `tasks.md`.
  - Implement git diff collection combining uncommitted dirty files (`probe_git_dirty_files`) and committed branch changes relative to the base branch (`main` / `master` / merge-base).
  - Implement `reconcile_tasks_with_git_diff(&tasks_path, &touched_files)` returning `Option<TaskDesyncReport>`.
- **Surface Diagnostics**:
  - `resume_lines`: Inject a prominent `! Warning: Tasks desync detected (...)` notice in the context re-hydration block.
  - `status_lines`: Surface the desync warning in the status overview.
  - `checkpoint_lines`: Display a warning notice when saving a checkpoint if tasks remain desynced.
  - `ce-ai doctor`: Add a diagnostic probe reporting `doctor-warn: openspec tasks desync: ...` without failing the command (exit 0).
- **FSM Guard**:
  - In `infer_stage_from_repo` / `maybe_auto_checkpoint`, avoid false transitions and surface desync context cleanly.
- **Verification**:
  - Comprehensive unit tests in `src/commands/tests/workflow.rs` covering path extraction, diff correlation, and desync report generation.
  - Integration tests in `tests/cli.rs` testing `ce-ai workflow resume`, `status`, and `doctor` output under desynced conditions.

## Out-of-Scope Boundaries

- **Issue #313 Part 2**: Investigating or altering Engram memory recency ranking or session-start prompt echoes (explicitly deferred by user directive).
- **Automatic File Editing of `tasks.md`**: `ce-ai` will NOT automatically check off checkboxes (`- [x]`) in `tasks.md`. Checking off tasks remains the explicit cognitive responsibility of the implementing agent or human developer.
- **Hard Blocking Manual Checkpoints**: `ce-ai workflow checkpoint` will never fail or abort due to tasks desync; it will record the checkpoint and emit an informational warning.

## Risk Evaluation & Mitigation

- **Risk: False Positives on General Commits**: A developer might touch documentation or unrelated tooling while working on a feature, triggering an unwarranted warning.
  - *Mitigation*: Filter out ignored paths (`.git/`, lockfiles, transient folders) and prioritize explicit path correlations between task text and touched files. If using aggregate fallback, only trigger when `src/` or `tests/` files are modified and `tasks.md` progress is `0/N`.
- **Risk: Performance Impact of Git Diff Scanning**: Running git diff on large repositories could add latency to `workflow resume` or `doctor`.
  - *Mitigation*: Restrict diff queries to lightweight commands (`git status --porcelain=v1` and `git diff --name-only $(git merge-base HEAD origin/main 2>/dev/null || echo HEAD~1)...HEAD`). Cache or reuse results within the invocation.
- **Risk: Breaking Existing Scripted Workflows**: If doctor failed with a non-zero exit code on desync, existing CI or pre-commit hooks could break.
  - *Mitigation*: Adhere strictly to the requirement: emit `doctor-warn:` without pushing to `findings` (exit code remains 0).

## Success Criteria

1. A standalone reconciliation function identifies when unchecked tasks correspond to touched git files.
2. `ce-ai workflow resume` outputs a clear warning when tasks desync is detected.
3. `ce-ai workflow status` displays the tasks desync warning.
4. `ce-ai doctor` emits `doctor-warn:` identifying the desynced feature change while exiting 0 when no other fatal findings exist.
5. Manual checkpoints succeed without error while echoing the desync warning.
6. 100% test pass rate across unit and CLI integration tests, 0 clippy warnings, strict formatting.
