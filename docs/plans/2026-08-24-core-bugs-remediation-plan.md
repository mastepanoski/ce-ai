# Implementation Plan: Core Bug Remediation (Issues #160, #156, #159)

## Objective
Remediate three high-priority (P1) core bugs:
1. **Issue #160 (Dry-Run Purity)**: Guarantee 100% zero disk mutation during `--dry-run` across `workflow checkpoint`, remote `install`, and remote `upgrade`, verified via triple-directory snapshot tests (`config_dir`, `home_dir`, `workspace_dir`).
2. **Issue #156 (Real 7-Stage Workflow FSM Engine)**: Transition `ce-ai workflow` from string storage in `last_update_check` to a strongly-typed `WorkflowStage` FSM engine in `state.workflow` with legal transition enforcement and context recovery (OpenSpec change package: `openspec/changes/workflow-fsm-engine/`).
3. **Issue #159 (Real `sync --watch` Loop)**: Upgrade `ce-ai sync --watch` from a single-pass exit to a real long-running watcher loop with automatic drift repair, safe Ctrl-C signal handling, and graceful termination (OpenSpec change package: `openspec/changes/sync-watch-loop/`).

## Schema Migration & User Impact
- `state.json`: Schema is expanded to include `workflow: Option<WorkflowState>`. Older state files containing `last_update_check` string entries are parsed transparently into `WorkflowState` fallback, preserving full backward compatibility.
- OpenSpec Change Specs: `openspec/changes/dry-run-purity/`, `openspec/changes/workflow-fsm-engine/`, `openspec/changes/sync-watch-loop/`.

## Proposed Code Changes

### 1. `src/state/state.rs`
- Add `WorkflowStage` enum (1..=7) and `WorkflowState` struct.
- Add `workflow: Option<WorkflowState>` to `State` with `last_update_check` fallback parsing.

### 2. `src/commands/workflow.rs`
- Guard state persistence with `if !ctx.dry_run`.
- Enforce transition rules ($N \rightarrow 1$, $N \rightarrow N$, $N \rightarrow N+1$, $N \rightarrow N-1$).
- Support `--json` flag and context recovery re-hydration probing `openspec/changes/`.

### 3. `src/commands/install.rs` & `src/commands/upgrade.rs`
- Under `ctx.dry_run`, resolve remote sources to transient temporary directories without writing to `cache/` or updating `state.json`.

### 4. `src/commands/sync.rs`
- Implement long-running polling loop for `sync --watch` with `--interval-ms` and `--max-passes` support.
- Pre-check drift via in-memory `diff::diff` before running disk repairs.

### 5. `src/tui.rs`
- Update workflow stage transition key handlers (1..7) and modal rendering to enforce `WorkflowStage` transition validation rules.

### 6. `tests/cli.rs`
- Add snapshot helper `assert_dry_run_zero_mutation` verifying `config_dir`, `home_dir`, and `workspace_dir`.
- Add integration tests for dry-run purity, FSM transition enforcement, and `sync --watch` drift restoration.

## Verification Plan
```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
