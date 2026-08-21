# OpenSpec Tasks: Release v0.5.0 Implementation

- [ ] **Unit 1: Workspace Scope Installation (`src/commands/install.rs` & `src/state/state.rs`)**
  - [ ] Add `--scope` flag and git repository root resolution.
- [ ] **Unit 2: Companion Tools Manager (`src/commands/tools.rs` & `src/main.rs`)**
  - [ ] Implement `ce-ai tools status` and `ce-ai tools install`.
  - [ ] Integrate companion tools checks into `ce-ai doctor`.
- [ ] **Unit 3: Workflow FSM & Recovery Engine (`src/commands/workflow.rs` & `src/main.rs`)**
  - [ ] Implement `ce-ai workflow status`, `checkpoint`, and `resume`.
- [ ] **Unit 4: Integration Tests (`tests/cli.rs`)**
  - [ ] Write CLI integration tests for workspace scope, tools manager, and workflow FSM.
