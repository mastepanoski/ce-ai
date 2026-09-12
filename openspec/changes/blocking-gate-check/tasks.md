# Tasks: Blocking Gate Check & Validation Receipt (Issue #334)

## Overview & Scope Forecast
- **Estimated Total Changed Lines:** ~940 LOC (production code: ~450 LOC, unit & integration tests: ~490 LOC).
- **Target Work Unit Size:** ~100–200 LOC per unit.
- **Verification Strategy:** Strict Test-Driven Development (TDD) for every unit before progressing to the next.

---

### Work Unit 1: Fix Archive Directory Exclusion in Mtime Fallback
- [x] **WU1.1 (TDD Test):** Add unit test in `src/commands/tests/workflow.rs` verifying that `probe_openspec_context_in` ignores `archive/` directory even when it has the newest mtime timestamp. (Est: ~60 LOC)
- [x] **WU1.2 (Implementation):** Update `probe_openspec_context_in` in `src/commands/workflow.rs` to filter out directory named `archive` and any directory starting with `.`. (Est: ~25 LOC)
- [x] **WU1.3 (Verification):** Run `cargo test workflow::tests` and confirm `ce-ai status` no longer warns about active feature 'archive'.

### Work Unit 2: Data Structures & State Extension
- [x] **WU2.1 (TDD Test):** Add serialization and round-trip tests in `src/state/tests/state.rs` for `GateMode`, expanded `GateDecision::Blocked`, and `GateReceipt`. (Est: ~80 LOC)
- [x] **WU2.2 (Implementation):** Implement `GateMode` enum (`Enforce`, `Observe`), expand `GateDecision` with `Blocked`, and implement `GateReceipt` struct in `src/commands/gate.rs` and `src/state/state.rs`. (Est: ~80 LOC)
- [x] **WU2.3 (Implementation):** Add `gate_mode` and `gate_receipts` fields to `State` in `src/state/state.rs` with backward-compatible serde attributes. (Est: ~30 LOC)
- [x] **WU2.4 (Verification):** Run `cargo test state::tests` and verify clean compilation.

### Work Unit 3: Pure Policy Evaluation Engine
- [x] **WU3.1 (TDD Tests):** Add comprehensive unit tests in `src/commands/tests/gate.rs` testing:
  - `ce-debug` task text / entry point exempt from OpenSpec requirement. (Est: ~40 LOC)
  - `AdoptionTier::Minimal` exempt from OpenSpec requirement. (Est: ~40 LOC)
  - Non-`src/**` target path exempt from OpenSpec requirement. (Est: ~30 LOC)
  - Edge cases (`mtime_fallback`, `worktree_uncommitted`, `stale_cycle_guard`) pass with edge case category. (Est: ~40 LOC)
  - Enforce mode returns `GateDecision::Blocked` with missing files list when contract missing in Stage 4. (Est: ~40 LOC)
  - Observe mode returns `GateDecision::WouldBlock` with missing files list when contract missing in Stage 4. (Est: ~30 LOC)
- [x] **WU3.2 (Implementation):** Implement `evaluate_gate_policy` in `src/commands/gate.rs` with all policy rules, returning `(GateDecision, Option<GateEdgeCase>, Vec<String>, String)`. (Est: ~120 LOC)
- [x] **WU3.3 (Verification):** Run `cargo test gate::tests` and ensure all policy decision tests pass.

### Work Unit 4: Structured Receipt Generation & Actionable Remediation Feedback
- [x] **WU4.1 (TDD Tests):** Add unit tests for `write_gate_receipt` and stderr message formatting in `src/commands/tests/gate.rs`. (Est: ~70 LOC)
- [x] **WU4.2 (Implementation):** Implement `format_blocked_remediation_message` generating the structured error output detailing target file, active stage/feature, missing artifacts, and remediation commands. (Est: ~50 LOC)
- [x] **WU4.3 (Implementation):** Implement `write_gate_receipt` creating/updating `.validation.json` in `openspec/changes/<feature>/` (if directory exists) and updating `state.gate_receipts` via atomic state write. (Est: ~60 LOC)
- [x] **WU4.4 (Verification):** Verify that `.validation.json` is formatted cleanly and atomic writes succeed.

### Work Unit 5: Hook Integration, CLI Wiring & Exit Code 2 Contract
- [x] **WU5.1 (TDD Tests):** Add CLI integration tests simulating `ce-ai gate check` with stdin payload and CLI arguments, asserting Exit Code 2 on blocked writes and Exit Code 0 on passed/exempt writes. (Est: ~120 LOC)
- [x] **WU5.2 (Implementation):** Update `GateCheckArgs` to accept `--mode` and `--entry-point`, wire mode resolution (flag > env > state > default enforce), and update `run_gate_check` to emit stderr message and return `CeError::Usage` (Exit Code 2) on `GateDecision::Blocked`. (Est: ~110 LOC)
- [x] **WU5.3 (Verification):** Run `cargo test gate` and verify Exit Code 2 contract on blocked tool calls.

### Work Unit 6: Doctor & Status Observability Integration
- [x] **WU6.1 (TDD Tests):** Add unit tests in `src/commands/tests/status.rs` and `doctor.rs` verifying gate check receipts and blocked write counts are surfaced properly. (Est: ~70 LOC)
- [x] **WU6.2 (Implementation):** Update `status.rs` and `doctor.rs` to display blocked write counts and surface any recent blocked writes as actionable warnings. (Est: ~60 LOC)
- [x] **WU6.3 (Verification):** Run `cargo test status` and `cargo test doctor`.

### Work Unit 7: Quality Gate & DoD Verification
- [x] **WU7.1 (Formatting & Linter):** Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] **WU7.2 (Test Suite):** Run `cargo test`.
- [x] **WU7.3 (E2E Verification):** Run `make e2e` in Docker to verify containerized installation and execution.
- [x] **WU7.4 (Documentation & Knowledge Capture):** Document solution in `docs/solutions/architecture/` and update `CHANGELOG.md`.
