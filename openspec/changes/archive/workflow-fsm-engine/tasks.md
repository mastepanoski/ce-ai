# Task Breakdown: Real 7-Stage Workflow FSM Engine & Context Recovery

- [x] Add `WorkflowStage` enum and `WorkflowState` struct in `src/state/state.rs`
- [x] Add `workflow: Option<WorkflowState>` field in `State` struct with backward-compatible `last_update_check` fallback
- [x] Implement transition validation logic ($N \rightarrow 1$, $N \rightarrow N$, $N \rightarrow N+1$, $N \rightarrow N-1$) in `src/commands/workflow.rs`
- [x] Implement context recovery probing with OpenSpec directory & git branch fallbacks in `workflow resume`
- [x] Update `src/tui.rs` stage transition key handlers (1..7) and modal rendering to enforce `WorkflowStage` transition validation rules
- [x] Add `--json` output support for `ce-ai workflow status/checkpoint/resume`
- [x] Add unit and CLI integration tests in `tests/cli.rs`
