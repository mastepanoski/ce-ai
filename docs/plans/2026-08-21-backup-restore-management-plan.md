# Implementation Plan: Backup Management & Point-in-Time Recovery

**Date**: 2026-08-21  
**Status**: Approved / Ready for Execution  
**Origin**: [Requirements Doc](docs/brainstorms/2026-08-21-backup-restore-management-requirements.md) | [OpenSpec](openspec/changes/backup_restore_management/spec.md)  
**Target Milestone**: `v0.5.0`  
**Related Issue**: [#13](https://github.com/mastepanoski/ce-ai/issues/13)  

---

## 1. Executive Summary & Problem Frame

`ce-ai` automatically creates timestamped backups in `~/.ce-ai/backups/` before any harness mutation, but currently lacks CLI commands and a TUI interface for listing historical backups and performing targeted point-in-time config restoration.

This implementation plan defines the step-by-step development units for:
1. `src/state/backups.rs`: Core `BackupEntry` model, `list_backups`, and `restore_backup_by_id`.
2. `src/commands/backups.rs`: CLI subcommand dispatcher for `ce-ai backups list` and `ce-ai backups restore <timestamp>`.
3. `src/main.rs`: Clap CLI parser routing.
4. `src/tui.rs`: Ratatui dashboard tab `MenuTab::Backups` with interactive selection and modal recovery triggers.
5. `tests/cli.rs`: End-to-end integration tests.

---

## 2. Implementation Units & Verification Scenarios

### Unit 1: Core Backup Model & Inspection API
- **Target File**: `src/state/backups.rs`
- **Description**: Add `BackupEntry` struct representing a timestamped backup directory snapshot. Implement `list_backups(root: &Path, harness_filter: Option<&str>)` returning backups sorted newest-first. Implement `restore_backup_by_id(root: &Path, id: &str, target: &Path)` with pre-restore safety snapshots and `write_atomic`.
- **Test Scenarios**:
  - `test_list_backups_returns_sorted_entries`: Verify listing timestamped dirs in `~/.ce-ai/backups/`.
  - `test_list_backups_with_harness_filter`: Verify filtering by harness name (e.g. `opencode`, `claude`).
  - `test_restore_backup_by_id_restores_target`: Verify targeted snapshot restoration onto target path.
  - `test_restore_backup_invalid_id_fails`: Verify error exit when non-existent backup ID is passed.

### Unit 2: CLI Subcommand Handler & Parser Routing
- **Target Files**: `src/commands/backups.rs`, `src/commands/mod.rs`, `src/main.rs`
- **Description**: Define `BackupsArgs` and `BackupsSubcommand` (`List` and `Restore`) in `src/main.rs`. Implement `src/commands/backups.rs` to output formatted tables in CLI mode. Register `pub mod backups;` in `src/commands/mod.rs`.
- **Test Scenarios**:
  - `cli_backups_list_outputs_formatted_table`: Verify running `ce-ai backups list` via `assert_cmd`.
  - `cli_backups_restore_restores_selected_snapshot`: Verify running `ce-ai backups restore <timestamp>`.

### Unit 3: Interactive TUI Dashboard Integration
- **Target File**: `src/tui.rs`
- **Description**: Add `MenuTab::Backups` to sidebar menu. Render backup list table showing ID, timestamp, target harness, and file size. Support `Up`/`Down` key navigation and `[Enter]` confirmation modal to execute `restore_backup_by_id`.
- **Test Scenarios**:
  - `test_tui_render_backups_tab`: Verify TUI state initialization and tab navigation rendering.

---

## 3. Execution Dependency Sequence

```
  [Unit 1: Core Backup API (src/state/backups.rs)]
                         │
                         ▼
  [Unit 2: CLI Subcommand (src/commands/backups.rs & src/main.rs)]
                         │
                         ▼
  [Unit 3: TUI Integration (src/tui.rs)] ➔ [Verification & Integration Tests]
```

---

## 4. Verification Checklist & Definition of Done

- [ ] All unit tests in `src/state/backups.rs` pass (`cargo test`).
- [ ] Integration tests in `tests/cli.rs` pass cleanly.
- [ ] Formatting (`cargo fmt --check`) and Clippy (`cargo clippy --all-targets --all-features -- -D warnings`) report 0 warnings.
- [ ] Containerized Docker E2E gate (`make e2e`) passes cleanly.
