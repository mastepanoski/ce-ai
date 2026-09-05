# Exploration: Workflow FSM Auto-Checkpoint & Harness Lifecycle

## 1. Technical Investigation of Harness Hook Ecosystems

Currently, `ce-ai` only hooks into Turn-0 session initialization (`SessionStart` in Claude/Codex/Cursor, `session.created` in OpenCode, `PreInvocation` in Agy, `before_agent_start` in Pi). Each hook executes `ce-ai workflow resume` to deliver drift diagnostics and advisory text.

A detailed investigation of the supported harnesses reveals hook events suitable for stage inference and auto-checkpointing:

| Harness | Events Beyond Session-Start | Characteristics & Injection Capabilities | Applicable Workflow Trigger |
|---|---|---|---|
| **Claude Code** | `Stop`, `PreCompact` (and `PostToolUse`) | `.claude/settings.json`. `Stop` runs at end of loop/turn. `PreCompact` fires before context compaction. Exit code 2 can block (MUST NOT be used; fail-open only). | Turn-end: `Stop`. Compaction: `PreCompact`. |
| **Codex CLI** | `Stop`, `PreCompact`, `PostToolUse`, `SessionEnd` | `.codex/config.toml` (`[hooks]`). Schema mirrors Claude Code hooks. | Turn-end: `Stop`. Compaction: `PreCompact`. |
| **GitHub Copilot CLI** | `postToolUse` | `.github/hooks/hooks.json`. Supports returning an object with `additionalContext` back to the active agent prompt. | Turn-end / Post-tool: `postToolUse`. |
| **Cursor** | `afterFileEdit`, `afterShellExecution`, `postToolUse`, `stop`, `subagentStop` | `.cursor/hooks.json`. Used in production by GitButler for auto-commits. | Turn-end: `stop`. |
| **Google Antigravity (`agy`)** | `PreToolUse`, `PostToolUse`, `PreInvocation`, `PostInvocation`, `Stop` | Defined in `.agents/hooks.json` under custom hook groups (e.g. `"compound-engineering"`). Flat array per hook type. `Stop` runs at loop termination. | Turn-0: `PreInvocation`. Turn-end: `Stop`. |
| **Pi** | `tool_result`, `agent_end`, `session_before_compact` | Registered via `pi.on(event, handler)` in `.pi/extensions/compound-engineering.ts`. `session_before_compact` fires before compaction; `agent_end` fires when agent finishes turn. | Turn-end: `agent_end`. Compaction: `session_before_compact`. |
| **OpenCode** | `session.idle`, `experimental.session.compacting` | In-process JS plugin (`BUILTIN_LOADER`). `session.idle` fires when agent becomes idle. | Turn-end: `session.idle`. Compaction: `compacting`. |

### Precedent: `rtk` (rtk-ai.app)
`rtk` uses `PreToolUse` in Claude Code / Cursor to transparently rewrite shell commands (`git status` -> `rtk git status`). This proves harness hooks can execute frequently and safely when kept lightweight and fail-open.

### Decision on Hook Trigger Granularity
Rather than spawning CLI processes on every tool call (`PostToolUse`, `afterFileEdit`), which introduces noticeable latency in turns with 20–50 tool calls, we hook into:
1. **Turn-end / Inactivity** (`Stop` in Claude Code, Codex, Cursor, Agy; `agent_end` in Pi; `session.idle` in OpenCode; `postToolUse` in Copilot).
2. **Pre-compaction** (`PreCompact` in Claude Code, Codex; `session_before_compact` in Pi; `experimental.session.compacting` in OpenCode).

This guarantees that:
- Stage inference updates during active multi-hour sessions (e.g. `/ce-debug` fixing a bug and running tests).
- Stage checkpoints are persisted before context exhaustion compaction occurs.
- Spawning overhead is kept to 1 invocation per agent turn rather than dozens per turn.

---

## 2. Workspace Scoping Bug Analysis

### Root Cause
In `src/state/state.rs:266`:
```rust
pub fn normalize_workspace_key(root: &Path) -> String {
    match std::fs::canonicalize(root) {
        Ok(canonical) => canonical.to_string_lossy().to_string(),
        Err(_) => root.to_string_lossy().to_string(),
    }
}
```
`workflows: BTreeMap<String, WorkflowState>` indexes entries solely by canonical filesystem path:
1. Developer works on `feat/billing-v2` in `/repo` -> Stage 4 recorded.
2. Developer switches branch to `fix/typo` in the same directory.
3. Automated or manual checkpoint runs -> overwrites `/repo` entry with Stage 4 or Stage 1 for `fix/typo`.
4. Developer switches back to `feat/billing-v2` -> FSM stage is completely corrupted or stale.

### Evaluated Options
- **Option A: Nested Maps `BTreeMap<CanonicalRoot, BTreeMap<Branch, WorkflowState>>`**:
  Clean hierarchy, but breaking change to `state.json` schema. Requires schema migration for existing user files.
- **Option B: Composite Key `<canonical_root>::<branch>` with Fallback (Recommended)**:
  Retains the flat `BTreeMap<String, WorkflowState>`.
  When branch is known (`probe_git_branch` returns `Some(b)`), key is `<canonical_root>::<branch>`.
  When branch is detached, non-git, or legacy lookup: fallback to `<canonical_root>`, then fallback to top-level `workflow`.
  100% backward-compatible with zero migration needed.

---

## 3. Hook Versioning & Upgrade Gap Analysis

### Root Cause
In `src/commands/init_prj.rs`:
- `ensure_session_start_hook`, `ensure_pre_invocation_hook`, and `ensure_session_start_plugin` are only called during `init-prj`.
- Neither `ce-ai sync` nor `ce-ai upgrade` invokes hook verification.
- In `src/harness/pi.rs`, `has_session_start_hook` checks `content.contains("ce-ai workflow resume")`. If present, `ensure_session_start_hook` returns `Ok(false)` without checking content or version.
- Even `ce-ai init-prj --force` does not pass `force` to hook updaters!

### Evaluated Options
- **Option A: Re-write files unconditionally**: Risks clobbering user customizations.
- **Option B: Embedded Version Header + Hook Completeness Check (Recommended)**:
  - Embed a version identifier (e.g. `// ce-ai:hook v=2` for TS/JS, or check that all expected hook keys exist in JSON/TOML).
  - In `init-prj --force`, `sync`, and `upgrade`: inspect installed hooks. If version is stale or required hooks (`Stop`, `PreCompact`, `session_before_compact`, `session.idle`) are missing, refresh the hook while preserving non-managed blocks.

---

## 4. De-init Asymmetry Gap Analysis

### Root Cause
`remove_*_hook` functions in `src/harness/{claude,codex,copilot,cursor,agy}.rs` were written to strip only the single hook key that existed at creation time:
- `agy.rs::remove_pre_invocation_hook`: only strips `group_obj.get_mut("PreInvocation")`.
- `claude.rs::remove_session_start_hook`: only strips `hooks_obj.get_mut("SessionStart")`.
- `codex.rs::remove_session_start_hook`: only strips `hooks.SessionStart`.
- `cursor.rs::remove_session_start_hook`: only strips `hooks_obj.get_mut("sessionStart")`.
- `copilot.rs::remove_session_start_hook`: only strips `hooks_obj.get_mut("sessionStart")`.

Adding `Stop` or `PreCompact` across these files without updating `remove_*_hook` would leave orphaned commands in user configs upon `ce-ai deinit-prj`.

### Evaluated Options
- **Symmetric Key Removal (Recommended)**: Every harness adapter file must maintain strict parity between `ensure_*_hook` and `remove_*_hook`. Every hook key added must be explicitly cleared in removal. Verified by CLI integration roundtrip tests across all harnesses.

---

## 5. Concurrency, Race Conditions & CAS Tradeoffs

### Root Cause
In `src/state/mod.rs:38` (`write_atomic`) and `src/state/state.rs:251` (`State::save`):
The update pattern is:
```
load(path) -> mutate in memory -> save(path) [tempfile + atomic rename]
```
`write_atomic` guarantees single-write file integrity, but does not provide multi-process transaction isolation. If two hooks (or two parallel subagents) read `state.json` simultaneously, both mutate their copy, and the last writer clobbers the first writer's updates.

### Evaluated Options & Known Tradeoff
- **Option A: Heavyweight OS File Locks (`fs2` / `flock`)**:
  Can cause deadlock or timeouts across OS platforms and container environments.
- **Option B: Reload-Immediately-Before-Save with Monotonic Stage Gating (Recommended)**:
  Immediately before writing to `state.json`, reload the state from disk.
  Verify that the pending transition is still valid against the freshly read state (`current_stage.can_transition_to(target_stage)`).
  For automated inference: only apply monotonic stage advances (`target_stage > disk_stage`). Never overwrite a manual checkpoint or a higher inferred stage.
  *Known Limitation*: This drastically narrows the race window from seconds (spanning an agent turn) down to the sub-millisecond IO duration of reload+save, but does not constitute an ACID distributed lock. Concurrent writes within that exact sub-millisecond window remain last-writer-wins. This is a deliberate, pragmatic tradeoff for workstation CLI execution without external lock daemons.

---

## 6. Security, Blast Radius, Performance & Product Contract

1. **Security (Path Traversal via Git Branch)**:
   `git branch --show-current` can be controlled by external actors in PRs or forks (e.g. `../../etc/passwd` or hostile branch names).
   Interpolating branch directly into `openspec/changes/<branch>` creates a directory traversal vector.
   *Mitigation*: Implement strict sanitization: strip path traversal elements (`..`, leading `/`), allow only alphanumeric, `-`, `_`, and map `/` to `-`.
2. **Blast Radius (Shared `state.json`)**:
   `state.json` stores all adopted projects on the workstation.
   *Mitigation*: Validate JSON serialization before atomic rename; ensure failure in one project never mutates or invalidates other project entries.
3. **Performance (Process Spawn Overhead)**:
   Avoid subprocess execution per tool call. Primary automated checkpoints trigger on **turn-end** (`Stop`, `agent_end`, `session.idle`) and **pre-compaction** (`PreCompact`, `session_before_compact`, `compacting`).
4. **Product Contract (Adoption Opt-In & Configurable Opt-Out)**:
   The README states: *"recording is opt-in"*.
   *Resolution*: Project adoption (`ce-ai init-prj`) is the explicit opt-in boundary. Unadopted repositories receive no hooks and zero automated recording. Within adopted projects, auto-checkpointing is active by default, and can be opted out at any time via `ce-ai config set auto-checkpoint false` (or flag `--no-auto-checkpoint`). The README and user guide will be updated to reflect this adoption-level opt-in model.
5. **Execution Safety (Zero Blocking)**:
   Harness hooks (`Stop`, `PreCompact`) must NEVER return non-zero or blocking exit codes (e.g. Claude Code exit code 2). All hooks must be strictly observational and 100% fail-open.
