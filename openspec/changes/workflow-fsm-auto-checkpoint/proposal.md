# Proposal: Workflow FSM Auto-Checkpoint Inference & Harness Hook Integration

## 1. Problem Statement
The 7-stage Workflow FSM (`ce-ai workflow status` / `checkpoint` / `resume`, implemented in `src/commands/workflow.rs` and `src/state/state.rs`) is designed as a resilience and recovery mechanism for context-window exhaustion and session hand-offs. However, in real-world agent workflows, the FSM **never advances automatically**:
- None of the canonical compound-engineering skills (`ce-brainstorm`, `ce-plan`, `ce-work`, `ce-debug`, `ce-compound`, `ce-commit-push-pr`) call `ce-ai workflow checkpoint`. Grepping the entire canonical skill distribution returns zero invocations.
- Upstream skills are external and immutable from the perspective of `ce-ai`; `ce-ai` must not rely on upstream skills modifying their prompts or command sequences to invoke checkpoints.
- Although `probe_repo_state()` in `src/commands/workflow.rs` already gathers signals (branch, dirty files, manifest drift, OpenSpec task completion), this data is only printed as advisory text during `resume` and is **never written back to update `WorkflowState.stage` in `state.json`**.
- Furthermore, expanding workflow tracking to automatic hooks exposes 7 specific technical risks and architectural gaps (Issue #296):
  1. **Harness Hook Scope**: Today, `ce-ai` only registers Turn-0 session-start hooks (`SessionStart`, `PreInvocation`, `before_agent_start`). Harnesses expose rich post-tool, turn-end, and pre-compaction hooks (`PostToolUse`, `Stop`, `PreCompact`, `tool_result`, `agent_end`, `session_before_compact`, `tool.execute.after`, `session.idle`) that can guarantee stage persistence before context is lost.
  2. **Adapter Concrete Targets**: Agy (`.agents/hooks.json`), Pi (`.pi/extensions/compound-engineering.ts`), and OpenCode (`.opencode/plugins/compound-engineering.js`) have concrete extension mechanisms that require tailored implementation.
  3. **Workspace Scoping Bug**: `State::normalize_workspace_key` (`src/state/state.rs:266`) keys `workflows: BTreeMap<String, WorkflowState>` solely by canonical directory path. Switching branches in the same working tree clobbers the previous branch's `WorkflowState`.
  4. **Hook Versioning & Upgrade Gap**: `ensure_session_start_hook` / `ensure_pre_invocation_hook` / `ensure_session_start_plugin` are only invoked during `init-prj` (never during `sync` or `upgrade`). Their idempotency check is string presence, meaning existing projects never receive updated hooks even with `init-prj --force`.
  5. **De-init Symmetry Gap**: `deinit-prj` surgical cleanup functions (`remove_*_hook` in Claude, Cursor, Copilot, Codex, Agy) only know the single hook key that existed when they were written. Any newly registered hook keys (`PostToolUse`, `Stop`, etc.) would be orphaned upon project de-adoption.
  6. **Concurrency & Race Conditions in `state.json`**: `write_atomic` provides atomic file replacement via temporary file rename, but lacks compare-and-swap (CAS) or optimistic concurrency. High-frequency hook writes from subagents or multiple sessions can silently overwrite each other.
  7. **Security, Blast Radius & Performance**:
     - Branch names are untrusted strings that must be sanitized before being used in filesystem paths (`Path::join`) to prevent directory traversal.
     - `state.json` is a single shared file for all projects; corruption or parse failures break `ce-ai` globally.
     - Spawning CLI processes on every tool call introduces unacceptable latency; turn-end and pre-compact hooks or in-process debouncing are required.
     - Product contract: README documents "recording is opt-in". Automatic checkpoints must provide an explicit opt-out configuration (`auto_checkpoint = false`).
     - Execution safety: Hooks (`Stop`, `PreCompact`) must NEVER utilize blocking exit codes (e.g. exit 2 to force continuation) to avoid infinite agent loops. All hooks must be strictly observational and fail-open.

## 2. Scope & Boundaries

### In Scope
- **Stage Inference Engine (`infer_workflow_stage`)**:
  - Automatically derive the active `WorkflowStage` from observable repo artifacts:
    - Stage 1 (Ideation): `docs/brainstorms/*.md` or `docs/ideation/` present without corresponding OpenSpec.
    - Stage 2 (OpenSpec): `openspec/changes/<feature>/{proposal,spec}.md` present.
    - Stage 3 (Plan): `tasks.md` present with `total_tasks > 0` and `completed_tasks == 0`.
    - Stage 4 (Work/TDD): `completed_tasks > 0 && completed_tasks < total_tasks`, OR non-default branch (`feat/*`, `fix/*`) with dirty/uncommitted changes (direct entry / `ce-debug` path).
    - Stage 5 (Verify): `completed_tasks == total_tasks` on feature branch.
    - Stage 6 (Compound): New/modified files under `docs/solutions/` on the branch.
    - Stage 7 (Ship): Open PR or merged branch.
  - Abort inference during transitory git states (`.git/rebase-merge`, `.git/rebase-apply`, `.git/CHERRY_PICK_HEAD`, `.git/MERGE_HEAD`).
- **Provenance & Monotonicity**:
  - Extend `WorkflowState` with `source: WorkflowSource` (`Manual` vs `Inferred`).
  - Allow inference to advance stage monotonically (`can_transition_to`), but forbid inferred transitions from regressing or overwriting a `Manual` checkpoint of equal or higher stage.
- **Branch-Aware Workspace Keying**:
  - Key `workflows` as `<canonical_root>::<branch>` with fallback to `<canonical_root>` when branch is detached or unavailable.
  - Maintain backward compatibility with existing `state.json` structures.
- **Concurrency & CAS (`update_workflow_state`)**:
  - Implement read-before-write compare-and-swap pattern in `State::save` / `validate_and_set_workflow_for` to prevent silent last-writer-wins clobbering.
- **Hook Lifecycle & Versioning**:
  - Version all harness hooks with an embedded version tag (`// ce-ai:hook v=2` or JSON metadata).
  - Refresh hooks in `ce-ai init-prj --force`, `ce-ai sync`, and `ce-ai upgrade`.
  - Provide symmetric removal for all registered hook keys in `deinit-prj`.
- **Harness Integration**:
  - Agy: configure `PreInvocation` and non-blocking `Stop` in `.agents/hooks.json`.
  - Pi: configure `before_agent_start`, `agent_end`, and `session_before_compact` in `.pi/extensions/compound-engineering.ts`.
  - OpenCode: configure `session.created`, `session.idle`, and `experimental.session.compacting` in `compound-engineering.js`.
  - Claude, Codex, Cursor, Copilot: maintain session start and symmetric deinit.
- **Security & Performance**:
  - Path sanitization for `feature_name` and branch names.
  - Configurable opt-out: `auto_checkpoint: bool` (default true for adopted projects, togglable via `ce-ai config set auto-checkpoint false`).
  - Fail-open execution across all hooks.
- **Comprehensive Testing**:
  - Unit tests for inference, CAS, branch-keying, and sanitization.
  - CLI integration roundtrip tests for every supported harness hook (`init-prj` -> `deinit-prj`).

### Out of Scope
- Modifying upstream compound-engineering skills (`~/.config/opencode/skills/*`).
- Blocking agent termination or forcing loop continuation via hook exit codes.
- Distributing workflow state across network remotes (workflow state remains workstation-local in `state.json`).

## 3. Success Criteria
- [ ] Running `ce-ai workflow resume`, `status`, or harness hooks automatically detects and persists the active FSM stage in `state.json` without explicit `checkpoint` calls.
- [ ] Switching git branches in the same repository preserves independent FSM stages per branch without cross-branch clobbering.
- [ ] Transitory git states (rebase, merge, cherry-pick) suppress auto-checkpointing.
- [ ] Inferred checkpoints never regress an explicit manual checkpoint.
- [ ] `ce-ai init-prj --force`, `ce-ai sync`, and `ce-ai upgrade` detect and update stale hook versions.
- [ ] `ce-ai deinit-prj` cleanly and symmetrically removes all registered hook keys without leaving orphaned hooks.
- [ ] Malformed or hostile branch names cannot trigger directory traversal outside `openspec/changes/`.
- [ ] Setting `auto-checkpoint = false` disables automatic persistence while preserving manual checkpoints.
- [ ] 100% test pass rate across unit tests and CLI integration roundtrip tests.
