# OpenSpec Proposal: Release v0.5.0 Roadmap Features

## Problem Statement
Completes the v0.5.0 release milestone by addressing Issues #7, #9, and #10:
1. Support `--scope workspace|global` in `ce-ai install`.
2. Manage companion tools (`Engram`, `CodeGraph`, `Context7`, `RTK`) via `ce-ai tools status|install`.
3. Provide Workflow FSM & Progress Recovery via `ce-ai workflow status|checkpoint|resume`.

## Proposed Changes
- **Commands**:
  - `src/commands/install.rs`: Add `--scope <workspace|global>` flag.
  - `src/commands/tools.rs`: Implement `ce-ai tools status` and `ce-ai tools install`.
  - `src/commands/workflow.rs`: Implement `ce-ai workflow status|checkpoint|resume`.
- **State**:
  - Add `tools` and `workflow_checkpoint` state to `state.json`.
