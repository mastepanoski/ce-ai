# Exploration: Spike Observe-Only — Medir (Sin Bloquear) Escritura en ce-work Sin OpenSpec Aprobado

## 1. Technical Investigation

### 1.1 Reading Declared Workflow Stage from Checkpoint
In `ce-ai`, the active workflow state is explicitly tracked per workspace and branch in `state.json` (`src/state/state.rs`):
```rust
pub struct WorkflowState {
    pub stage: WorkflowStage,
    pub task: String,
    pub feature_name: Option<String>,
    pub updated_at: String,
    pub source: WorkflowSource,
    pub resolution: Option<FeatureResolution>,
}
```
`state.current_workflow_for_branch(root, branch)` retrieves the active checkpoint.
- **Why re-inferring was rejected**: Re-inferring stage via `infer_stage_from_repo` inside the gate check would duplicate FSM heuristic logic, introduce race conditions against unstaged files, and recreate the blind spot documented in DDW post-mortems (guards failing on unmodeled states like `IDLE`). Reading the declared stage directly respects the Single Source of Truth established by the FSM.

### 1.2 Edge Case Detection Signals (Issue #337 / PR #344)
Issue #337 established clear signals for three degraded inference scenarios:
1. **`mtime_fallback`**: When `wf.resolution == Some(FeatureResolution::MtimeFallback)`. Occurs in non-git environments or when the active branch does not match any OpenSpec change directory.
2. **`worktree_uncommitted`**: Detected hermetically via `crate::commands::workflow::probe_openspec_has_uncommitted(repo_root, &feature)`. Occurs when spec files are authored locally in a worktree but not yet committed to git.
3. **`stale_cycle_guard`**: When `wf.task.contains("(nuevo ciclo detectado)")` (persisted by `maybe_auto_checkpoint` upon same-branch cycle restarts).
These three buckets must be strictly quarantined from the happy path so that environmental or FSM transition quirks do not corrupt the false-positive measurement.

### 1.3 Claude Code Hook Substrate
In Claude Code, hook configurations reside in `.claude/settings.json` under the `hooks` dictionary.
`src/harness/claude.rs` already manages lifecycle hooks (`SessionStart`, `Stop`, `PreCompact`) using atomic JSON writes.
For write observation, Claude Code provides the `PreToolUse` event:
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "ce-ai gate check"
          }
        ]
      }
    ]
  }
}
```
When Claude Code invokes a `PreToolUse` hook command:
- It streams the tool call details as JSON over `stdin`:
  `{ "tool_name": "Write", "tool_input": { "path": "src/commands/foo.rs", "content": "..." } }`
  or `{ "tool": "Edit", "input": { "file_path": "src/main.rs" } }`.
- Dual-input design: `ce-ai gate check` will parse explicit CLI flags (`--tool`, `--path`) first, falling back to streaming `stdin` JSON if flags are absent. This allows both seamless Claude Code integration and effortless unit/integration testing without subprocess piping.

### 1.4 Kill-Switch Protocol
The kill-switch must ensure total inertness with zero disk access:
- Evaluated as the first instruction in `ce-ai gate check`.
- Triggers on environment variables `CE_AI_DISABLE_GATE_CHECK=1` or `CE_AI_GATE_CHECK_DISABLED=1`, or the `--kill-switch` / `--disabled` CLI flag.
- When active, returns `Ok(())` (exit code 0) immediately.

### 1.5 Structured Telemetry Logging
- **Log Path**: `ctx.config_dir.join("gate-events.jsonl")`.
- **Concurrency & Safety**: Opened with append-only mode (`OpenOptions::new().create(true).append(true)`). Each record is a single newline-terminated JSON object (`GateEventRecord`).
- **Privacy Assurance**: Telemetry logs only metadata (timestamp, tool, target path, declared stage, active feature, decision, reason). It never captures file contents, buffers, or diffs.

## 2. Evaluated Options

| Dimension | Option A | Option B (Selected) | Rationale |
|---|---|---|---|
| **Decision Logic Placement** | Shell script in Claude hook | Pure Rust function in `src/commands/gate.rs` | Option B is fully testable with standard unit tests, platform-independent, and avoids shell escaping/parsing bugs. |
| **Stage Determination** | Re-infer stage from filesystem | Read declared stage from `WorkflowState` checkpoint | Option B avoids duplicating FSM logic and testing against transient filesystem states. |
| **Spike Execution Mode** | Blocking gate with override receipt | Observe-only (always exit 0, log only) | Option B satisfies the core objective: gather empirical false-positive data before introducing blocking bottlenecks. |
| **Input Interface** | `stdin` only | Dual mode (flags + `stdin` fallback) | Option B enables transparent hook usage by Claude Code while maintaining straightforward CLI testing and manual execution. |

## 3. Tradeoffs & Architectural Alignment

- **Append-only JSONL vs SQLite/Database**: JSONL requires zero new dependencies, handles concurrent appends cleanly on Unix/Windows, and can be read line-by-line in constant memory for metrics aggregation in `doctor` and `status`.
- **Fail-Safe Exception Handling**: Any I/O error encountered while writing to `gate-events.jsonl` emits an informative debug warning to `stderr` but never returns an error exit code to Claude Code. The primary invariant (*"the write always proceeds"*) is inviolable.
