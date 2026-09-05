# Tasks: Workflow FSM Auto-Checkpoint & Harness Lifecycle

## Work Unit 1: State & Scoping Core (`src/state/state.rs`, `src/state/mod.rs`)
- [ ] Add `WorkflowSource` enum (`Manual`, `Inferred`) with `Default` returning `Manual`.
- [ ] Extend `WorkflowState` struct with `pub source: WorkflowSource` (`#[serde(default)]` for backward compatibility).
- [ ] Implement `State::workspace_branch_key(root: &Path, branch: Option<&str>) -> String`.
- [ ] Implement `State::current_workflow_for_branch(&self, root: &Path, branch: Option<&str>) -> Option<WorkflowState>` with hierarchical fallback (`<canonical_root>::<branch>` -> `<canonical_root>` -> `self.workflow`).
- [ ] Implement `State::atomic_update_workflow` with read-before-write compare-and-swap (CAS) reload check to eliminate multi-turn read-modify-write races on `state.json`.
- [ ] TDD Unit Tests:
  - Verify branch composite key creation and fallback.
  - Verify CAS reload check prevents clobbering when concurrent updates occur.
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

## Work Unit 3: Comprehensive Harness Hook Implementations (All 7 Harnesses)
- [ ] Update `src/harness/claude.rs`:
  - Register `Stop` and `PreCompact` hooks in `.claude/settings.json`.
  - Symmetrically update `remove_session_start_hook` to clean up `SessionStart`, `Stop`, and `PreCompact`.
- [ ] Update `src/harness/codex.rs`:
  - Register `Stop` and `PreCompact` hooks in `.codex/config.toml`.
  - Symmetrically update `remove_session_start_hook` to clean up `SessionStart`, `Stop`, and `PreCompact`.
- [ ] Update `src/harness/cursor.rs`:
  - Register `stop` hook in `.cursor/hooks.json`.
  - Symmetrically update `remove_session_start_hook` to clean up `sessionStart` and `stop`.
- [ ] Update `src/harness/copilot.rs`:
  - Register `postToolUse` hook in `.github/hooks/hooks.json`.
  - Symmetrically update `remove_session_start_hook` to clean up `sessionStart` and `postToolUse`.
- [ ] Update `src/harness/agy.rs`:
  - Register `Stop` hook in `.agents/hooks.json` under `"compound-engineering"`.
  - Symmetrically update `remove_pre_invocation_hook` to clean up both `PreInvocation` and `Stop`.
- [ ] Update `src/harness/pi.rs`:
  - Extend `PI_EXTENSION_CONTENT` in `.pi/extensions/compound-engineering.ts` with `agent_end` and `session_before_compact` hooks.
  - Embed version marker `// ce-ai:hook v=2`.
  - Update `has_session_start_hook` and `ensure_session_start_hook` to detect version `v=2`.
- [ ] Update `src/opencode/plugins.rs` and `.opencode/plugins/compound-engineering.js`:
  - Add `event.type === "session.idle"` handler in `BUILTIN_LOADER` to trigger turn-end auto-checkpoint.
- [ ] TDD Unit Tests:
  - Unit tests verifying serialization, hook presence, and complete removal across each of the 7 harness modules.
*Estimated LOC: ~280 lines*

## Work Unit 4: Rollout, Versioning & Symmetry Integration (`src/commands/init_prj.rs`, `sync.rs`, `upgrade.rs`, `deinit_prj.rs`)
- [ ] Update `src/commands/init_prj.rs`:
  - Support updating hooks across all harnesses when `force == true` or when existing hook has stale version / missing keys.
- [ ] Wire hook verification and refresh into `src/commands/sync.rs` and `src/commands/upgrade.rs`.
- [ ] Update `src/commands/deinit_prj.rs`:
  - Symmetrically clean up all registered hook keys across Claude, Codex, Copilot, Cursor, and Agy.
- [ ] TDD Unit Tests:
  - Verify `init-prj --force` upgrades stale hooks across harnesses.
  - Verify `sync` reconciles missing or stale hooks.
*Estimated LOC: ~180 lines*

## Work Unit 5: CLI Integration & Roundtrip Test Suite (`tests/cli.rs`)
- [ ] Add CLI integration test: `workflow_auto_checkpoint_infers_stage_on_resume_and_status`.
- [ ] Add CLI integration test: `workflow_branch_scoping_prevents_cross_branch_clobbering`.
- [ ] Add CLI integration test: `workflow_transitory_git_state_suppresses_auto_checkpoint`.
- [ ] Add CLI integration test: `workflow_inferred_stage_never_regresses_manual_checkpoint`.
- [ ] Add CLI integration tests: roundtrip `init-prj` -> `deinit-prj` verifying 100% clean removal without orphans across:
  - Claude Code (`.claude/settings.json`)
  - Codex (`.codex/config.toml`)
  - Cursor (`.cursor/hooks.json`)
  - Copilot (`.github/hooks/hooks.json`)
  - Antigravity (`.agents/hooks.json`)
  - Pi (`.pi/extensions/compound-engineering.ts`)
*Estimated LOC: ~260 lines*

## Work Unit 6: Documentation & Contract Clarification (`docs/user-guide/fsm-and-checkpoints-explained.md`, `README.md`)
- [ ] Update `docs/user-guide/fsm-and-checkpoints-explained.md`:
  - Describe automated stage inference via harness turn-end and pre-compact hooks.
  - Accurately explain `ce-debug` direct entry behavior (Stage 4 auto-inference on dirty fix branch).
  - Clarify the adoption-level opt-in model and document the `auto_checkpoint` configuration option.
- [ ] Verify `README.md` length constraint (<= 100 lines) and clarify wording regarding adoption-level opt-in.
*Estimated LOC: ~80 lines*

---
**Total PR / Feature Forecast: ~1,190 changed lines across 6 focused work units.**
