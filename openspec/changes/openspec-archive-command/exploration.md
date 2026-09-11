# Exploration: OpenSpec Change Archival Mechanics and Tooling

## 1. Technical Context & Investigation
In `ce-ai v1.44.2` (PR #322, Issue #323), the probe `probe_unarchived_completed_changes(&repo_root)` was established:
- Iterates over all direct directories in `openspec/changes/` (excluding `archive`).
- Parses `tasks.md` with `count_task_checkboxes(&tasks_path)` counting `- [x]`, `- [X]`, and `- [ ]`.
- Identifies any change where `total > 0 && completed == total`.
- Surfaced via Turn-0 `resume_lines`, `status_lines`, `checkpoint_lines`, and `ce-ai doctor`.

While diagnosis was successfully implemented, execution was omitted. In `exploration.md` of that feature, automated `git mv` was evaluated and rejected because automated file moves without explicit invocation could race with developer edits. However, this left a tool vacancy: there was no command for the user or agent to say: "Yes, archive it now."

## 2. Evaluated Options

### Option 1: Command Surface Placement
- **Approach A: Top-level command `ce-ai archive [feature]` only.**
  - *Pros:* Shortest keystrokes.
  - *Cons:* Bypasses `ce-ai workflow` where other stage lifecycle commands reside.
- **Approach B: Workflow subcommand `ce-ai workflow archive [feature]` only.**
  - *Pros:* Cohesive with `ce-ai workflow {status, checkpoint, resume, review-receipt}`.
  - *Cons:* Slightly longer command line.
- **Approach C (Selected): Dual Registration.**
  - Define `Archive` as `workflow::Action::Archive`, and expose a top-level alias `ce-ai archive` in `src/commands/registry.rs` that delegates directly to it.
  - *Pros:* Delivers maximum discoverability and developer ergonomics while preserving FSM architectural hierarchy.

### Option 2: Directory Move Mechanism
- **Approach A: Pure `std::fs::rename`.**
  - *Pros:* Zero external dependencies, pure Rust.
  - *Cons:* In a git repository, git sees this as a deletion of $N$ files and creation of $N$ untracked files until `git add` is executed. Fails across different filesystem mount points.
- **Approach B: Direct `git mv` execution via `std::process::Command`.**
  - *Pros:* Git tracks renames natively with 100% similarity index, staging the operation immediately.
  - *Cons:* Fails if run in an uncommitted dirty subfolder, non-git directory, or outside worktrees.
- **Approach C (Selected): Git-First with Atomic Filesystem Fallback.**
  - Execute `git mv openspec/changes/<feature> openspec/changes/archive/<feature>`.
  - If git fails or directory is not a git worktree, perform `std::fs::rename` followed by `git add -A` if git is present.
  - Assert that destination does not exist prior to calling either.

### Option 3: Completion Criteria Enforcement
- **Approach A: Checkbox 100% Only (Criterion 1).**
  - *Pros:* Trivially simple.
  - *Cons:* Blocks archival of real-world features that shipped with intentionally cut or rescoped tasks (e.g. 7 folders archived in v1.44.2 required manual editing because tasks were open).
- **Approach B (Selected): Dual Criteria (Criterion 1 + Criterion 2).**
  - Allow archival if all tasks are complete (`completed == total > 0`).
  - Or, if open tasks remain, allow archival only if `--status "<evidence>"` is provided.
  - Prepend `> STATUS: <evidence>\n\n` to `tasks.md` prior to moving, declaring open tasks formally unaudited.
  - Return `CeError::Verification` (exit code 6) if open tasks exist without `--status`.

### Option 4: `archive/README.md` Ledger Synchronization
- **Approach A: Static documentation (no automation).**
  - *Pros:* No markdown parsing code.
  - *Cons:* Table quickly drifts from disk reality; 33 backlog changes would not be recorded.
- **Approach B (Selected): Automated Historical Notes / Triage Table Update.**
  - Parse `openspec/changes/archive/README.md`.
  - Maintain the clean triage table (`| Folder | Open boxes | Next action |`) and append a date-stamped sweep entry under `Historical notes` documenting the archived feature and task counts.

## 3. Evaluated Tradeoffs

| Decision | Chosen Alternative | Key Tradeoff / Rationale |
| :--- | :--- | :--- |
| **Command Scope** | Dual registration (`workflow archive` + top-level `archive`) | Ergonomics for humans/agents without fragmenting the FSM domain. |
| **Move Strategy** | `git mv` with atomic fs fallback | Preserves git history continuity and rename tracking while maintaining offline robustness. |
| **Safety Gate** | Dirty tree check + destination collision check | Prevents clobbering existing archives or committing unreviewed code modifications. |
| **Status Attestation** | Require explicit non-empty `--status` | Prevents bypassing task verification with arbitrary skips while supporting valid cut scopes. |
