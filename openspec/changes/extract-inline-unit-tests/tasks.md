# Tasks: Extract Inline Unit Tests into Dedicated Test Files (#265)

## Work Units & Forecasted Slices (<400 LOC per PR)

### Unit T1: OpenSpec Specification Contract (This PR)
- [x] T1.1: Author `proposal.md` with problem statement and risk register.
- [x] T1.2: Author `exploration.md` with file inventory and layout tradeoffs.
- [x] T1.3: Author `design.md` with domain inventory and target file mapping.
- [x] T1.4: Author `spec.md` with formal requirements and acceptance matrix.
- [x] T1.5: Author `tasks.md` with executable task checklist.
*Forecast: ~320 LOC (docs only).*

---

### Unit T2: Extract State & OpenCode Domains (PR 2)
- [x] T2.1: Extract `src/state/state.rs` tests to `src/state/tests/state.rs`.
- [x] T2.2: Extract `src/state/diff.rs` tests to `src/state/tests/diff.rs`.
- [x] T2.3: Extract `src/state/ports.rs` tests to `src/state/tests/ports.rs`.
- [x] T2.4: Extract `src/state/backups.rs` tests to `src/state/tests/backups.rs`.
- [x] T2.5: Extract `src/state/journal.rs` tests to `src/state/tests/journal.rs`.
- [x] T2.6: Extract `src/state/profiles.rs` tests to `src/state/tests/profiles.rs`.
- [x] T2.7: Extract `src/state/mod.rs` tests to `src/state/tests/mod_tests.rs`.
- [x] T2.8: Extract `src/opencode/config.rs` tests to `src/opencode/tests/config.rs`.
- [x] T2.9: Extract `src/opencode/manifest.rs` tests to `src/opencode/tests/manifest.rs`.
- [x] T2.10: Extract `src/opencode/plugins.rs` tests to `src/opencode/tests/plugins.rs`.
- [x] T2.11: Verify quality gates (`cargo fmt --check`, `cargo clippy`, `cargo test`).
*Forecast: ~380 LOC net diff.*

---

### Unit T3: Extract Source & Error Domains (PR 3)
- [ ] T3.1: Extract `src/error.rs` tests to `src/tests/error.rs`.
- [ ] T3.2: Extract `src/source/cache.rs` tests to `src/source/tests/cache.rs`.
- [ ] T3.3: Extract `src/source/tools_registry.rs` tests to `src/source/tests/tools_registry.rs`.
- [ ] T3.4: Extract `src/source/registry.rs` tests to `src/source/tests/registry.rs`.
- [ ] T3.5: Extract `src/source/release.rs` tests to `src/source/tests/release.rs`.
- [ ] T3.6: Extract `src/source/archive.rs` tests to `src/source/tests/archive.rs`.
- [ ] T3.7: Verify quality gates (`cargo fmt --check`, `cargo clippy`, `cargo test`).
*Forecast: ~360 LOC net diff.*

---

### Unit T4: Extract Harness Adapters — Part 1 (PR 4)
- [ ] T4.1: Extract `src/harness/agents.rs` tests to `src/harness/tests/agents.rs`.
- [ ] T4.2: Extract `src/harness/pi.rs` tests to `src/harness/tests/pi.rs`.
- [ ] T4.3: Extract `src/harness/claude.rs` tests to `src/harness/tests/claude.rs`.
- [ ] T4.4: Extract `src/harness/copilot.rs` tests to `src/harness/tests/copilot.rs`.
- [ ] T4.5: Extract `src/harness/grok.rs` tests to `src/harness/tests/grok.rs`.
- [ ] T4.6: Extract `src/harness/codex.rs` tests to `src/harness/tests/codex.rs`.
- [ ] T4.7: Verify quality gates (`cargo fmt --check`, `cargo clippy`, `cargo test`).
*Forecast: ~370 LOC net diff.*

---

### Unit T5: Extract Harness Adapters Part 2 & TUI (PR 5)
- [ ] T5.1: Extract `src/harness/custom.rs` tests to `src/harness/tests/custom.rs`.
- [ ] T5.2: Extract `src/harness/agy.rs` tests to `src/harness/tests/agy.rs`.
- [ ] T5.3: Extract `src/harness/cursor.rs` tests to `src/harness/tests/cursor.rs`.
- [ ] T5.4: Extract `src/harness/fx.rs` tests to `src/harness/tests/fx.rs`.
- [ ] T5.5: Extract `src/harness/kimi.rs` tests to `src/harness/tests/kimi.rs`.
- [ ] T5.6: Extract `src/harness/mod.rs` tests to `src/harness/tests/mod_tests.rs`.
- [ ] T5.7: Extract `src/tui/mod.rs` tests to `src/tui/tests/mod_tests.rs`.
- [ ] T5.8: Verify quality gates (`cargo fmt --check`, `cargo clippy`, `cargo test`).
*Forecast: ~390 LOC net diff.*

---

### Unit T6: Extract Commands — Part 1 (PR 6)
- [ ] T6.1: Extract `src/commands/upgrade.rs` tests to `src/commands/tests/upgrade.rs`.
- [ ] T6.2: Extract `src/commands/tools.rs` tests to `src/commands/tests/tools.rs`.
- [ ] T6.3: Extract `src/commands/audit.rs` tests to `src/commands/tests/audit.rs`.
- [ ] T6.4: Extract `src/commands/models.rs` tests to `src/commands/tests/models.rs`.
- [ ] T6.5: Extract `src/commands/guard.rs` tests to `src/commands/tests/guard.rs`.
- [ ] T6.6: Verify quality gates (`cargo fmt --check`, `cargo clippy`, `cargo test`).
*Forecast: ~380 LOC net diff.*

---

### Unit T7: Extract Commands Part 2 & Knowledge Capture (PR 7)
- [ ] T7.1: Extract `src/commands/sync.rs` tests to `src/commands/tests/sync.rs`.
- [ ] T7.2: Extract `src/commands/doctor.rs` tests to `src/commands/tests/doctor.rs`.
- [ ] T7.3: Extract `src/commands/init_prj.rs` tests to `src/commands/tests/init_prj.rs`.
- [ ] T7.4: Extract `src/commands/workflow.rs` tests to `src/commands/tests/workflow.rs`.
- [ ] T7.5: Extract `src/commands/install.rs` tests to `src/commands/tests/install.rs`.
- [ ] T7.6: Knowledge capture in `docs/solutions/architecture/extract-inline-unit-tests-2026-08-28.md`.
- [ ] T7.7: Verify full quality gates (`cargo fmt --check`, `cargo clippy`, `cargo test`, `make e2e`).
*Forecast: ~380 LOC net diff.*
