# Tasks: Pedagogical Guardrail Mode (`ce-ai guard`)

## Implementation Work Units

### Unit G1: State Schema & Backward Compatibility
- [ ] **G1.1 (Data Types):** Define `GuardLevel` (`Junior`, `Strict`) and `GuardrailConfig` in `src/state/state.rs`. (Estimate: ~40 LOC)
- [ ] **G1.2 (State Integration):** Extend `State` with `#[serde(default, skip_serializing_if = "Option::is_none")] pub guardrail: Option<GuardrailConfig>`. (Estimate: ~15 LOC)
- [ ] **G1.3 (Unit Tests):** Add tests verifying serialization round-trip, default behavior, and legacy JSON compatibility without `guardrail` field. (Estimate: ~65 LOC)
*Subtotal Estimate: ~120 LOC*

---

### Unit G2: Guard CLI Command & Registry Dispatch
- [ ] **G2.1 (Clap Subcommand):** Add `GuardCommands` (`Enable`, `Disable`, `Status`) and `GuardArgs` to `src/commands/mod.rs`. (Estimate: ~35 LOC)
- [ ] **G2.2 (Command Implementation):** Create `src/commands/guard.rs` implementing `run_guard_enable`, `run_guard_disable`, `run_guard_status` with `write_atomic` and `--dry-run` guards. (Estimate: ~110 LOC)
- [ ] **G2.3 (Registry Dispatch):** Implement `CeCommand` for `GuardCommand` and wire into `src/commands/registry.rs`. (Estimate: ~25 LOC)
- [ ] **G2.4 (CLI Integration Tests):** Add tests in `tests/cli.rs` covering `guard enable`, `guard disable`, `guard status`, `--level strict`, invalid level exit code 2, and `--dry-run`. (Estimate: ~80 LOC)
*Subtotal Estimate: ~250 LOC*

---

### Unit G3: Health Checks, Doctor, and TUI Visibility
- [ ] **G3.1 (Doctor Integration):** Update `src/commands/doctor.rs` to report guardrail activation and verify asset consistency. (Estimate: ~30 LOC)
- [ ] **G3.2 (Status Integration):** Update `src/commands/status.rs` to render guardrail status row. (Estimate: ~25 LOC)
- [ ] **G3.3 (TUI Dispatch):** Wire guardrail commands into TUI action runner (`src/tui/runner.rs` / `src/tui/handlers.rs`). (Estimate: ~45 LOC)
- [ ] **G3.4 (End-to-End Verification):** Run `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `make e2e`. (Estimate: ~30 LOC)
*Subtotal Estimate: ~130 LOC*

---

## Forecast & Delivery Slices

- **PR 1 (G1 + G2):** State schema + `ce-ai guard` CLI commands + CLI tests (~370 LOC, within <400 line budget).
- **PR 2 (G3):** Doctor + Status + TUI integration + docs (~160 LOC).
