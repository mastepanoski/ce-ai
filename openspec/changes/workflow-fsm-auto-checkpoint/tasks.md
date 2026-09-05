# Tasks: Workflow FSM Auto-Checkpoint & Harness Lifecycle

## Work Unit 1: State & Scoping Core (`src/state/state.rs`, `src/state/mod.rs`)
- [ ] Add `WorkflowSource` enum (`Manual`, `Inferred`) with `Default` returning `Manual`.
- [ ] Extend `WorkflowState` struct with `pub source: WorkflowSource` (`#[serde(default)]` for backward compatibility).
- [ ] Implement `State::workspace_branch_key(root: &Path, branch: Option<&str>) -> String`.
- [ ] Implement `State::current_workflow_for_branch(&self, root: &Path, branch: Option<&str>) -> Option<WorkflowState>` with hierarchical fallback (`<canonical_root>::<branch>` -> `<canonical_root>` -> `self.workflow`).
- [ ] Implement `State::atomic_update_workflow` with read-before-write compare-and-swap (CAS) to eliminate read-modify-write races on `state.json`.
- [ ] TDD Unit Tests:
  - Verify branch composite key creation and fallback.
  - Verify CAS prevents clobbering when concurrent updates occur.
  - Verify deserialization of legacy `state.json` without `source` or branch keys defaults to `WorkflowSource::Manual`.
*Estimated LOC: ~180 lines*

## Work Unit 2: Stage Inference Engine & Security Sanitization (`src/commands/workflow.rs`)
- [ ] Implement `is_transitory_git_state(repo_root: &Path) -> bool` checking for `.git/rebase-merge`, `.git/rebase-apply`, `.git/CHERRY_PICK_HEAD`, and `.git/MERGE_HEAD`.
- [ ] Implement `sanitize_feature_name(branch: &str) -> String` to prevent path traversal outside `openspec/changes/`.
- [ ] Implement `infer_stage_from_repo(repo_root: &Path, branch: Option<&str>) -> Option<(WorkflowStage, String, Option<String>)>` mapping observable repository state to Stages 1 through 7.
- [ ] Enforce monotonic progression: inferred stages advance state only when `can_transition_to` is valid and never regress a `WorkflowSource::Manual` checkpoint.
- [ ] Wire inference into `ce-ai workflow resume` and `ce-ai workflow status`.
- [ ] Support `auto_checkpoint: bool` configuration toggle (skip persistence when disabled).
- [ ] TDD Unit Tests:
  - Verify stage inference across Stages 1 to 7 with mock repo fixtures.
  - Verify transitory git states abort inference cleanly.
  - Verify branch sanitization neutralizes directory traversal attempts (e.g. `../../bad`).
  - Verify manual checkpoints take precedence over equal or lower inferred stages.
*Estimated LOC: ~210 lines*

## Work Unit 3: Harness Hook Implementations (`src/harness/agy.rs`, `src/harness/pi.rs`, `src/opencode/plugins.rs`)
- [ ] Update `src/harness/agy.rs`:
  - Register non-blocking `"Stop"` hook pointing to `ce-ai workflow resume` under `"compound-engineering"` in `.agents/hooks.json`.
  - Update `remove_pre_invocation_hook` to symmetrically remove both `"PreInvocation"` and `"Stop"`.
- [ ] Update `src/harness/pi.rs`:
  - Extend `PI_EXTENSION_CONTENT` in `.pi/extensions/compound-engineering.ts` with `agent_end` and `session_before_compact` hooks.
  - Embed version marker `// ce-ai:hook v=2`.
  - Update `has_session_start_hook` and `ensure_session_start_hook` to detect version `v=2`.
- [ ] Update `src/opencode/plugins.rs` and `.opencode/plugins/compound-engineering.js`:
  - Add `event.type === "session.idle"` handler in `BUILTIN_LOADER` to trigger turn-end auto-checkpoint.
- [ ] TDD Unit Tests:
  - Verify Antigravity hooks write and parse `PreInvocation` and `Stop`.
  - Verify Pi extension contains version header and new event handlers.
  - Verify OpenCode plugin exports idle handler.
*Estimated LOC: ~190 lines*

## Work Unit 4: Rollout, Versioning & Symmetry Integration (`src/commands/init_prj.rs`, `sync.rs`, `upgrade.rs`, `deinit_prj.rs`)
- [ ] Update `src/commands/init_prj.rs`:
  - Support updating hooks when `force == true` or when existing hook has stale version.
- [ ] Wire hook verification and refresh into `src/commands/sync.rs` and `src/commands/upgrade.rs`.
- [ ] Update `src/commands/deinit_prj.rs`:
  - Symmetrically clean up all registered hook keys across Claude, Codex, Copilot, Cursor, and Agy.
- [ ] TDD Unit Tests:
  - Verify `init-prj --force` upgrades stale hooks.
  - Verify `sync` reconciles missing or stale hooks.
*Estimated LOC: ~160 lines*

## Work Unit 5: CLI Integration & Roundtrip Test Suite (`tests/cli.rs`)
- [ ] Add CLI integration test: `workflow_auto_checkpoint_infers_stage_on_resume_and_status`.
- [ ] Add CLI integration test: `workflow_branch_scoping_prevents_cross_branch_clobbering`.
- [ ] Add CLI integration test: `workflow_transitory_git_state_suppresses_auto_checkpoint`.
- [ ] Add CLI integration test: `workflow_inferred_stage_never_regresses_manual_checkpoint`.
- [ ] Add CLI integration test: `init_prj_and_deinit_prj_roundtrip_all_harness_hooks` verifying that `deinit-prj` leaves zero orphaned hook entries in `.agents/hooks.json`, `.cursor/hooks.json`, `.claude/settings.json`, `.github/hooks/hooks.json`, and `.codex/config.toml`.
*Estimated LOC: ~220 lines*

## Work Unit 6: Documentation Updates (`docs/user-guide/fsm-and-checkpoints-explained.md`, `README.md`)
- [ ] Update `docs/user-guide/fsm-and-checkpoints-explained.md`:
  - Describe automated stage inference via harness hooks and touchpoints.
  - Accurately explain `ce-debug` direct entry behavior (Stage 4 auto-inference on dirty fix branch).
  - Document `auto_checkpoint` configuration option.
- [ ] Verify `README.md` length constraint (<= 100 lines) and ensure wording accurately reflects optional auto-checkpointing.
*Estimated LOC: ~80 lines*

---
**Total PR / Feature Forecast: ~1,040 changed lines across 6 focused work units.**
