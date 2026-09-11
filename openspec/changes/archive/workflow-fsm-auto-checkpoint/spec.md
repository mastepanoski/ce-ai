# Specification: Workflow FSM Auto-Checkpoint & Harness Lifecycle

## 1. Automated Stage Inference

### 1.1 Ideation (Stage 1)
- **WHEN** `docs/ideation/` or `docs/brainstorms/*.md` exists and no `openspec/changes/<feature>/` directory exists,
- **THEN** `infer_workflow_stage` SHALL deduce `WorkflowStage::Ideation`.

### 1.2 OpenSpec Definition (Stage 2)
- **WHEN** `openspec/changes/<feature>/` contains `proposal.md` and `spec.md`, but either `tasks.md` is absent or `total_tasks == 0`,
- **THEN** `infer_workflow_stage` SHALL deduce `WorkflowStage::OpenSpec`.

### 1.3 Execution Plan (Stage 3)
- **WHEN** `openspec/changes/<feature>/tasks.md` exists with `total_tasks > 0` and `completed_tasks == 0`,
- **THEN** `infer_workflow_stage` SHALL deduce `WorkflowStage::ExecutionPlan`.

### 1.4 Work / TDD (Stage 4)
- **WHEN** `openspec/changes/<feature>/tasks.md` exists with `completed_tasks > 0` and `completed_tasks < total_tasks`,
- **OR WHEN** an active git branch starting with `fix/` or `feat/` has uncommitted/dirty changes without OpenSpec artifacts (direct entry / `ce-debug`),
- **THEN** `infer_workflow_stage` SHALL deduce `WorkflowStage::WorkTdd`.

### 1.5 Verification (Stage 5)
- **WHEN** `openspec/changes/<feature>/tasks.md` has `completed_tasks == total_tasks` and the branch is not yet merged or pushed,
- **THEN** `infer_workflow_stage` SHALL deduce `WorkflowStage::Verification`.

### 1.6 Knowledge Capture (Stage 6)
- **WHEN** new or modified markdown files exist under `docs/solutions/` on the current branch,
- **THEN** `infer_workflow_stage` SHALL deduce `WorkflowStage::KnowledgeCapture`.

### 1.7 Git Shipping (Stage 7)
- **WHEN** an active PR exists on GitHub for the current branch, or the current commit has been merged into the default branch,
- **THEN** `infer_workflow_stage` SHALL deduce `WorkflowStage::GitShipping`.

---

## 2. Transitory Git State Guard

- **WHEN** `.git/rebase-merge`, `.git/rebase-apply`, `.git/CHERRY_PICK_HEAD`, or `.git/MERGE_HEAD` is present in the repository,
- **THEN** stage auto-inference SHALL return `None` and MUST NOT persist any checkpoint to `state.json`.

---

## 3. Monotonicity & Provenance Protection

- **WHEN** an explicit `ce-ai workflow checkpoint` command executes,
- **THEN** the persisted `WorkflowState` SHALL have `source = WorkflowSource::Manual`.
- **WHEN** an automated stage inference produces a stage advancement,
- **THEN** the persisted `WorkflowState` SHALL have `source = WorkflowSource::Inferred`.
- **WHEN** an automated stage inference produces a stage that is less than or equal to an existing checkpoint with `source = WorkflowSource::Manual`,
- **THEN** the inference engine SHALL NOT overwrite or regress the manual checkpoint.

---

## 4. Workspace & Branch Scoping

- **WHEN** `ce-ai workflow checkpoint`, `status`, or `resume` runs on a git repository with an active branch name `B`,
- **THEN** `State` SHALL store and retrieve `WorkflowState` under the composite key `<canonical_root>::<B>`.
- **WHEN** a repository has detached HEAD or is not a git repository,
- **THEN** `State` SHALL fall back to storing and querying under `<canonical_root>`.
- **WHEN** reading legacy `state.json` without branch composite keys,
- **THEN** `State` SHALL seamlessly fall back to `<canonical_root>` and top-level `workflow`.

---

## 5. Concurrency Control & Limitations

- **WHEN** persisting an inferred or manual checkpoint via `atomic_update_workflow`,
- **THEN** the process SHALL reload `state.json` immediately prior to writing, verify that the transition from the freshly reloaded state is legal (`can_transition_to`), and write via atomic file replacement (`write_atomic`).
- **NOTE (Known Limitation)**: This reload-before-save pattern reduces the race window to sub-millisecond IO duration, but does not provide distributed multi-process locking. Concurrent writes within that sub-millisecond window remain last-writer-wins.

---

## 6. Comprehensive Hook Lifecycle & Upgrades across 7 Harnesses

- **WHEN** `ce-ai init-prj` runs on a project:
  - Claude Code: SHALL configure `SessionStart`, `Stop`, and `PreCompact` in `.claude/settings.json`.
  - Codex CLI: SHALL configure `SessionStart`, `Stop`, and `PreCompact` in `.codex/config.toml`.
  - Cursor: SHALL configure `sessionStart` and `stop` in `.cursor/hooks.json`.
  - GitHub Copilot: SHALL configure `sessionStart` and `postToolUse` in `.github/hooks/hooks.json`.
  - Antigravity: SHALL configure `PreInvocation` and `Stop` in `.agents/hooks.json`.
  - Pi: SHALL configure `before_agent_start`, `agent_end`, and `session_before_compact` in `.pi/extensions/compound-engineering.ts` (`v=2`).
  - OpenCode: SHALL configure `session.created`, `session.idle`, and `compacting` in `compound-engineering.js`.
- **WHEN** `ce-ai init-prj --force`, `ce-ai sync`, or `ce-ai upgrade` runs on an adopted project,
- **THEN** it SHALL verify that harness hooks across all 7 harnesses contain the latest hooks and version tag (`v=2`), rewriting them if missing or stale.

---

## 7. De-init Hook Symmetry

- **WHEN** `ce-ai deinit-prj` executes,
- **THEN** it SHALL surgically remove all registered hook keys (`SessionStart`, `Stop`, `PreCompact`, `sessionStart`, `stop`, `postToolUse`, `PreInvocation`, etc.) across all harness configurations, leaving no orphaned commands.

---

## 8. Security & Sanitization

- **WHEN** resolving a feature directory from a git branch name,
- **THEN** `ce-ai` SHALL sanitize the branch string to strip leading slashes, path traversal sequences (`..`), and special characters before performing path joining.

---

## 9. Product Contract & Opt-Out

- **WHEN** a repository is not adopted via `ce-ai init-prj`,
- **THEN** no hooks SHALL be installed and no automated recording SHALL occur (adoption-level opt-in).
- **WHEN** an adopted repository has `auto_checkpoint` configured to `false` (or `--no-auto-checkpoint` is passed),
- **THEN** automated stage inference SHALL NOT persist state changes to `state.json`.

---

## 10. Fail-Open Safety

- **WHEN** any harness hook (`Stop`, `PreCompact`, etc.) executes,
- **THEN** it SHALL execute in observational mode, exit with code 0, and NEVER return blocking exit codes.
