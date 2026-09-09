# Tasks: Spike Observe-Only — Medir (Sin Bloquear) Escritura en ce-work Sin OpenSpec Aprobado

Work-Unit Budget: ~450 LOC total forecast across 5 atomic units (~90 LOC avg per unit).
Follows TDD methodology: Test-First (RED) ➔ Implementation (GREEN) ➔ Refactor.

---

### Work Unit 1: Kill-Switch & Pure Decision Engine (`src/commands/gate.rs`)
**Forecast**: ~120 changed lines.
**Focus**: Pure decision function and emergency kill-switch logic.

- [x] **1.1 Unit Tests (RED)**:
  - Create `src/commands/tests/gate.rs`.
  - Add test `test_kill_switch_active_returns_true_for_env_vars_and_flag`.
  - Add test `test_pure_decision_engine_stage_4_missing_artifacts_would_block`.
  - Add test `test_pure_decision_engine_stage_4_complete_artifacts_pass`.
  - Add test `test_pure_decision_engine_non_stage_4_pass`.
  - Add test `test_pure_decision_engine_undetermined_when_no_checkpoint`.
  - Add test `test_pure_decision_engine_edge_cases_isolated` (MtimeFallback, WorktreeUncommitted, StaleCycleGuard).
- [x] **1.2 Implementation (GREEN)**:
  - Create `src/commands/gate.rs`.
  - Define `GateDecision` (`WouldBlock`, `Pass`, `Undetermined`, `EdgeCase`) and `GateEdgeCase` (`MtimeFallback`, `WorktreeUncommitted`, `StaleCycleGuard`).
  - Implement `is_gate_kill_switched(disabled_flag: bool) -> bool` checking env vars `CE_AI_DISABLE_GATE_CHECK`, `CE_AI_GATE_CHECK_DISABLED` and flag.
  - Implement `evaluate_gate_decision` as a pure, deterministic function.
- [x] **1.3 Refactor**:
  - Run `cargo test --lib commands::tests::gate`. Verify all pass green.

---

### Work Unit 2: CLI Subcommand `ce-ai gate check` & Telemetry Logging (`src/commands/gate.rs`, `src/main.rs`)
**Forecast**: ~110 changed lines.
**Focus**: Ingestion of tool/path from flags/stdin, checkpoint inspection, and structured append-only JSONL logging.

- [x] **2.1 Unit Tests (RED)**:
  - Add test `test_stdin_json_payload_parsing_for_claude_pre_tool_use`.
  - Add test `test_target_filter_rejects_non_write_and_non_src_paths`.
  - Add test `test_gate_event_record_append_and_stats_aggregation`.
- [x] **2.2 Implementation (GREEN)**:
  - Add `GateCheckArgs` struct in `src/commands/gate.rs`.
  - Wire `Gate(GateCommands)` into Clap `Commands` in `src/main.rs`.
  - Implement `extract_tool_and_path(args: &GateCheckArgs) -> Option<(String, String)>` supporting dual-input (flags vs stdin JSON).
  - Implement `log_gate_event(config_dir: &Path, record: &GateEventRecord) -> Result<(), CeError>`.
  - Implement `run_gate_check(ctx: &Context, args: &GateCheckArgs) -> Result<(), CeError>`:
    - Short-circuit on kill-switch (exit 0).
    - Filter on tool in `[Write, Edit]` and path under `src/` (exit 0).
    - Read active checkpoint via `state.current_workflow_for_branch`.
    - Check edge cases (`resolution`, `probe_openspec_has_uncommitted`, task suffix).
    - Evaluate decision via `evaluate_gate_decision`.
    - Append record to `gate-events.jsonl` (ignoring logging errors safely).
    - Always return `Ok(())`.
- [x] **2.3 Refactor**:
  - Run `cargo test --lib commands::tests::gate`. Verify all pass green.

---

### Work Unit 3: Claude Code `PreToolUse` Hook Wiring (`src/harness/claude.rs`)
**Forecast**: ~70 changed lines.
**Focus**: Hook lifecycle management in `.claude/settings.json`.

- [x] **3.1 Unit Tests (RED)**:
  - Add test `test_claude_gate_hook_lifecycle` in `src/harness/tests/claude.rs` verifying idempotent injection and surgical removal of `PreToolUse` hook for `ce-ai gate check`.
- [x] **3.2 Implementation (GREEN)**:
  - Implement `ensure_claude_gate_hook(settings_path: &Path) -> Result<bool, CeError>`.
  - Implement `remove_claude_gate_hook(settings_path: &Path) -> Result<bool, CeError>`.
  - Implement `has_claude_gate_hook(settings_path: &Path) -> bool`.
  - Hook into `src/commands/init_prj.rs` (configure on init) and `src/commands/deinit_prj.rs` (remove on deinit).
- [x] **3.3 Refactor**:
  - Run `cargo test --lib harness::tests::claude`. Verify all pass green.

---

### Work Unit 4: Observability Aggregation in `status` and `doctor` (`src/commands/status.rs`, `src/commands/doctor.rs`)
**Forecast**: ~60 changed lines.
**Focus**: Loading and displaying gate check telemetry counts without exposing sensitive write contents.

- [x] **4.1 Unit Tests (RED)**:
  - Add test `test_load_gate_stats_from_jsonl` in `src/commands/tests/gate.rs`.
  - Add assertions for gate check line in `src/commands/tests/doctor.rs`.
- [x] **4.2 Implementation (GREEN)**:
  - Implement `load_gate_stats(config_dir: &Path) -> Result<GateStats, CeError>` in `src/commands/gate.rs`.
  - Update `src/commands/status.rs` to print `gate-check: <total> observed (<would_block> would-block, <pass> pass, <undetermined> undetermined, <edge_case> edge-case: ...)`.
  - Update `src/commands/doctor.rs` to print non-blocking `gate-check: ...` telemetry status.
- [x] **4.3 Refactor**:
  - Run `cargo test --lib commands::tests::doctor`. Verify all pass green.

---

### Work Unit 5: End-to-End CLI Integration Tests & Quality Gates (`tests/cli.rs`)
**Forecast**: ~90 changed lines.
**Focus**: End-to-end integration test validating the entire pipeline.

- [x] **5.1 Integration Test (RED ➔ GREEN)**:
  - Add integration test `test_gate_check_spike_observe_only_lifecycle` in `tests/cli.rs`:
    1. Verify kill-switch short-circuit (`CE_AI_DISABLE_GATE_CHECK=1` and `--disabled`).
    2. Verify non-src writes exit 0 without logging.
    3. Verify Stage 4 write without OpenSpec contract logs `would-block` and exits 0.
    4. Verify Stage 4 write with complete OpenSpec contract logs `pass` and exits 0.
    5. Verify uncommitted spec logs `edge_case: worktree_uncommitted`.
    6. Verify `ce-ai status` and `ce-ai doctor` display aggregated telemetry counts.
- [x] **5.2 Verification Gate**:
  - Run `cargo fmt --check`.
  - Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - Run `cargo test`.
