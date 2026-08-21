# Technical Execution Plan: Release v0.6.0 Implementation

> **Spec Reference**: [`openspec/changes/v0_6_0_roadmap/`](../../openspec/changes/v0_6_0_roadmap/)

---

## 🎯 Release Goal
Expand `ce-ai` v0.6.0 into a proactive engineering environment by introducing:
1. **Interactive FSM Dashboard in TUI** (`src/tui.rs`).
2. **Extended Companion Diagnostics in `ce-ai doctor`** (`src/commands/doctor.rs`).
3. **Real-Time Sync Watcher (`ce-ai sync --watch`)** (`src/commands/sync.rs`).

---

## 📋 Task Breakdown & Sub-Loops

### Sub-Task 1: TUI Workflow Dashboard (`src/tui.rs`)
- **Action**: Add a new tab `Workflow (FSM)` to the TUI tab bar.
- **Widgets**: Render Ratatui `Gauge` showing 7-stage progress and a `Table` showing active checkpoints from `state.json`.
- **Verification**: Run `cargo check --tests` and verify layout rendering.

### Sub-Task 2: Extended Doctor Diagnostics (`src/commands/doctor.rs`)
- **Action**: Add companion tool health probes to `ce-ai doctor`:
  - `doctor_check_engram_db()`: Check file existence/permissions for `~/.engram/engram.db`.
  - `doctor_check_codegraph_index()`: Check `.codegraph/` index presence.
  - `doctor_check_rtk_path()`: Probe PATH for `rtk` binary.
- **Verification**: Run `cargo test` and verify CLI output format.

### Sub-Task 3: Real-Time Sync Watcher (`src/commands/sync.rs`)
- **Action**: Add `--watch` flag to `ce-ai sync` clap parser and implement loop re-syncing managed paths on modification.
- **Verification**: Run `cargo test --test cli`.

### Sub-Task 4: Comprehensive CLI Integration Tests (`tests/cli.rs`)
- **Action**: Add integration tests verifying TUI state loading and extended doctor diagnostic findings.
- **Verification**: `cargo test` passes 100% green.

---

## 🛡️ Verification Checklist before Hand-off
- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test`
- [ ] All OpenSpec tasks in `openspec/changes/v0_6_0_roadmap/tasks.md` updated to `- [x]` as units complete.
