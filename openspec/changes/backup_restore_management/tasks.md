# OpenSpec Task Checklist: Backup Management & Point-in-Time Recovery

**Feature Name**: `backup_restore_management`  
**Status**: Execution Planning  
**Target Release**: `v0.5.0`  
**Related Issue**: [#13](https://github.com/mastepanoski/ce-ai/issues/13)  

---

## Phase 1: Core Backup Inspection & Restore API (`src/state/backups.rs`)
- [ ] Implement `BackupEntry` struct with JSON serialization.
- [ ] Implement `list_backups(root: &Path, harness_filter: Option<&str>) -> Result<Vec<BackupEntry>, CeError>`.
- [ ] Implement `restore_backup_by_id(root: &Path, backup_id: &str, target_path: &Path) -> Result<(), CeError>`.
- [ ] Add unit tests in `src/state/backups.rs` verifying backup listing, filtering, and targeted restoration.

---

## Phase 2: CLI Subcommand (`src/commands/backups.rs` & `src/main.rs`)
- [ ] Add `Backups` subcommand and `BackupsArgs` Clap parser in `src/main.rs`.
- [ ] Implement `src/commands/backups.rs` dispatching `list` and `restore` actions.
- [ ] Add CLI integration tests in `tests/cli.rs` testing `ce-ai backups list` and `ce-ai backups restore`.

---

## Phase 3: TUI Dashboard Integration (`src/tui.rs`)
- [ ] Add `MenuTab::Backups` to `MenuTab` enum in `src/tui.rs`.
- [ ] Implement `render_content_panel` for `MenuTab::Backups` rendering backup snapshot table.
- [ ] Wire `run_backups_cmd` dispatcher and modal confirmation dialog.

---

## Phase 4: Verification & Release
- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo test`.
- [ ] Run `make e2e`.
- [ ] Update `CHANGELOG.md` and bump version to `0.5.0`.
