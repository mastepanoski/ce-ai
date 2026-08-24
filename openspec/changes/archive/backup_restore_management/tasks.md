# OpenSpec Task Checklist: Backup Management & Point-in-Time Recovery

**Feature Name**: `backup_restore_management`  
**Status**: Completed  
**Target Release**: `v0.5.0`  
**Related Issue**: [#13](https://github.com/mastepanoski/ce-ai/issues/13)  

---

## Phase 1: Core Backup Inspection & Restore API (`src/state/backups.rs`)
- [x] Implement `BackupEntry` struct with JSON serialization.
- [x] Implement `list_backups(root: &Path, harness_filter: Option<&str>) -> Result<Vec<BackupEntry>, CeError>`.
- [x] Implement `restore_backup_by_id(root: &Path, backup_id: &str, target_path: &Path) -> Result<(), CeError>`.
- [x] Add unit tests in `src/state/backups.rs` verifying backup listing, filtering, and targeted restoration.

---

## Phase 2: CLI Subcommand (`src/commands/backups.rs` & `src/main.rs`)
- [x] Add `Backups` subcommand and `BackupsArgs` Clap parser in `src/main.rs`.
- [x] Implement `src/commands/backups.rs` dispatching `list` and `restore` actions.
- [x] Add CLI integration tests in `tests/cli.rs` testing `ce-ai backups list` and `ce-ai backups restore`.

---

## Phase 3: TUI Dashboard Integration (`src/tui.rs`)
- [x] Add `MenuTab::Backups` to `MenuTab` enum in `src/tui.rs`.
- [x] Implement `render_content_panel` for `MenuTab::Backups` rendering backup snapshot table.
- [x] Wire `run_backups_cmd` dispatcher and modal confirmation dialog.

---

## Phase 4: Verification & Release
- [x] Run `cargo fmt --check`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test`.
- [x] Run `make e2e`.
- [x] Update `CHANGELOG.md` and bump version to `0.5.0`.
