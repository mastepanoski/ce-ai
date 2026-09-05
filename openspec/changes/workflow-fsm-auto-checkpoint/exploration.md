# Exploration: Workflow FSM Auto-Checkpoint & Harness Lifecycle

## 1. Technical Investigation of Harness Hook Ecosystems

Currently, `ce-ai` only hooks into Turn-0 session initialization (`SessionStart` in Claude/Codex/Cursor, `session.created` in OpenCode, `PreInvocation` in Agy, `before_agent_start` in Pi). Each hook executes `ce-ai workflow resume` to deliver drift diagnostics and advisory text.

A detailed investigation of the supported harnesses reveals hook events suitable for stage inference and auto-checkpointing:

| Harness | Events Beyond Session-Start | Characteristics & Injection Capabilities | Applicable Workflow Trigger |
|---|---|---|---|
| **Claude Code** | `PostToolUse`, `Stop`, `PreCompact` | `Stop` can return exit code 2 to block termination (MUST NOT be used). `PreCompact` is the critical boundary to persist state before memory compaction. | Turn-end: `Stop`. Compaction: `PreCompact`. |
| **Codex CLI** | `PostToolUse`, `PreCompact`, `Stop`, `SessionEnd` | Schema mirrors Claude Code hooks in `.codex/config.toml` (`[hooks]`). | Turn-end: `Stop`. Compaction: `PreCompact`. |
| **GitHub Copilot CLI** | `postToolUse` | Supports returning an object with `additionalContext` back to the active agent prompt. | Turn-end / Post-tool: `postToolUse`. |
| **Cursor** | `afterFileEdit`, `afterShellExecution`, `postToolUse`, `stop`, `subagentStop` | Used in production by GitButler for auto-commits. | Turn-end: `stop`. File events: `afterFileEdit`. |
| **Google Antigravity (`agy`)** | `PreToolUse`, `PostToolUse`, `PreInvocation`, `PostInvocation`, `Stop` | Defined in `.agents/hooks.json` under custom hook groups (e.g. `"compound-engineering"`). Flat array per hook type. `Stop` runs at loop termination. | Turn-0: `PreInvocation`. Turn-end: `Stop`. |
| **Pi** | `tool_result`, `agent_end`, `session_before_compact` | Registered via `pi.on(event, handler)` in `.pi/extensions/compound-engineering.ts`. `session_before_compact` fires before compaction; `agent_end` fires when agent finishes turn. | Turn-end: `agent_end`. Compaction: `session_before_compact`. |
| **OpenCode** | `tool.execute.after`, `file.edited`, `session.idle`, `experimental.session.compacting` | In-process JS plugin (`BUILTIN_LOADER`). `session.idle` fires when agent becomes idle. | Turn-end: `session.idle`. Compaction: `compacting`. |

### Precedent: `rtk` (rtk-ai.app)
`rtk` uses `PreToolUse` in Claude Code / Cursor to transparently rewrite shell commands (`git status` -> `rtk git status`). This proves harness hooks can execute frequently and safely when kept lightweight and fail-open.

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
- **Option B: Embedded Version Header + Stale Detection (Recommended)**:
  - Embed a version identifier (e.g. `// ce-ai:hook v=2` for TS/JS, or hook group inspection for JSON).
  - In `init-prj --force`, `sync`, and `upgrade`: inspect installed hooks. If version is stale or required hooks (`Stop`, `session_before_compact`) are missing, refresh the hook while preserving non-managed blocks.

---

## 4. De-init Asymmetry Gap Analysis

### Root Cause
`remove_*_hook` functions in `src/harness/{claude,codex,copilot,cursor,agy}.rs` were written to strip only the single hook key that existed at creation time:
- `agy.rs::remove_pre_invocation_hook`: only strips `group_obj.get_mut("PreInvocation")`.
- `claude.rs::remove_session_start_hook`: only strips `hooks_obj.get_mut("SessionStart")`.
- `cursor.rs::remove_session_start_hook`: only strips `hooks_obj.get_mut("sessionStart")`.
- `copilot.rs::remove_session_start_hook`: only strips `hooks_obj.get_mut("sessionStart")`.
- `codex.rs::remove_session_start_hook`: only strips `SessionStart` from TOML.

Adding `Stop` or `PostToolUse` without updating `remove_*_hook` would leave orphaned commands in user config upon `ce-ai deinit-prj`.

### Evaluated Options
- **Symmetric Key Removal (Recommended)**: Every harness adapter file must maintain parity between `ensure_*_hook` and `remove_*_hook`. Every hook key added must be explicitly cleared in removal. Verified by CLI integration roundtrip tests.

---

## 5. Concurrency & Race Condition Analysis

### Root Cause
In `src/state/mod.rs:38` (`write_atomic`) and `src/state/state.rs:251` (`State::save`):
The update pattern is:
```
load(path) -> mutate in memory -> save(path) [tempfile + atomic rename]
```
`write_atomic` guarantees single-write file integrity, but does not provide multi-process transaction isolation. If two hooks (or two parallel subagents) read `state.json` simultaneously, both mutate their copy, and the last writer clobbers the first writer's updates.

### Evaluated Options
- **Option A: Heavyweight OS File Locks (`fs2` / `flock`)**:
  Can cause deadlock or timeouts across OS platforms and containers.
- **Option B: Optimistic Compare-and-Swap (CAS) with Monotonic Stage Gating (Recommended)**:
  Immediately before writing to `state.json`, reload the state from disk.
  Verify that the pending transition is still valid against the freshly read state (`current_stage.can_transition_to(target_stage)`).
  For automated inference: only apply monotonic stage advances (`target_stage > disk_stage`). Never overwrite a manual checkpoint or a higher inferred stage.

---

## 6. Security, Blast Radius, Performance & Safety Analysis

1. **Security (Path Traversal via Git Branch)**:
   `git branch --show-current` can be controlled by external actors in PRs or forks (e.g. `../../etc/passwd` or hostile branch names).
   Interpolating branch directly into `openspec/changes/<branch>` creates a directory traversal vector.
   *Mitigation*: Implement strict sanitization: strip path traversal elements (`..`, leading `/`), allow only alphanumeric, `-`, `_`, and map `/` to `-`.
2. **Blast Radius (Shared `state.json`)**:
   `state.json` stores all adopted projects on the workstation.
   *Mitigation*: Validate JSON serialization before atomic rename; ensure failure in one project never mutates or invalidates other project entries.
3. **Performance (Process Spawn Overhead)**:
   Spawning a CLI binary on every tool call adds 20-50 subprocess executions per agent turn, introducing noticeable lag.
   *Mitigation*: Primary automated checkpoints must trigger on **turn-end** (`Stop`, `agent_end`, `session.idle`) and **pre-compaction** (`PreCompact`, `session_before_compact`, `compacting`). In OpenCode (in-process JS), tool execution hooks use in-memory debouncing.
4. **Product Contract (Opt-In / Opt-Out)**:
   The README advertises workflow checkpoints as opt-in.
   *Mitigation*: Support `auto_checkpoint = false` in configuration (and `--no-auto-checkpoint` flag). When disabled, only explicit `ce-ai workflow checkpoint` calls advance the FSM.
5. **Execution Safety (Zero Blocking)**:
   Harness hooks (`Stop`, `PreCompact`) must NEVER return non-zero or blocking exit codes (e.g. Claude Code exit code 2). All hooks must be strictly observational and 100% fail-open.
