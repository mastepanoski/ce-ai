# OpenSpec Tasks: Release v0.5.0 Implementation

- [x] **Unit 1: Workspace Scope Installation (`src/commands/install.rs` & `src/state/state.rs`)**
  - [x] Add `--scope` flag and git repository root resolution.
- [x] **Unit 2: Companion Tools Manager (`src/commands/tools.rs` & `src/main.rs`)**
  - [x] Implement `ce-ai tools status` and `ce-ai tools install`.
  - [x] Integrate companion tools checks into `ce-ai doctor`.
- [x] **Unit 3: Workflow FSM & Recovery Engine (`src/commands/workflow.rs` & `src/main.rs`)**
  - [x] Implement `ce-ai workflow status`, `checkpoint`, and `resume`.
- [x] **Unit 4: Integration Tests (`tests/cli.rs`)**
  - [x] Write CLI integration tests for workspace scope, tools manager, and workflow FSM.
