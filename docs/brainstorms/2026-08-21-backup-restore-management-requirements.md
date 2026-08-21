# Requirements Document: Backup Management & Point-in-Time Config Recovery

**Date**: 2026-08-21  
**Status**: Draft / Ready for Review  
**Target Milestone**: `v0.5.0`  
**Related Issue**: [#13](https://github.com/mastepanoski/ce-ai/issues/13)  
**Author**: Mauro Stepanoski  
**Scope**: Standard  

---

## 1. Feature Summary & User Value

`ce-ai` currently generates timestamped backup folders under `~/.ce-ai/backups/<utc-timestamp>/` prior to modifying host agent configurations. However, users currently have no CLI command or TUI interface to view stored backups or restore a specific past snapshot when configuration issues occur.

This feature adds:
1. `ce-ai backups list`: Inspect all stored configuration backups with timestamps, target harnesses, and sizes.
2. `ce-ai backups restore <timestamp_or_id>`: Safely restore a specific past configuration snapshot using atomic file writes.
3. **TUI Dashboard Panel**: An interactive `Backups & Restore` tab in the Ratatui dashboard allowing 1-click snapshot recovery.

---

## 2. Core User Outcomes & Acceptance Criteria

### Outcome 1: Inspect Historical Backups via CLI & TUI
- **Requirement 1.1**: `ce-ai backups list` MUST list all backup directories stored in `~/.ce-ai/backups/` sorted newest-first.
- **Requirement 1.2**: Users MUST be able to filter backup lists by target harness using `ce-ai backups list --harness <name>`.
- **Requirement 1.3**: The TUI dashboard MUST render an interactive table of backups under `MenuTab::Backups`.

### Outcome 2: Targeted Point-in-Time Recovery
- **Requirement 2.1**: `ce-ai backups restore <timestamp>` MUST locate the specified backup folder and validate JSON integrity before restoring.
- **Requirement 2.2**: Before overwriting a live configuration, `ce-ai` MUST create a new safety backup of the current state.
- **Requirement 2.3**: File writes MUST use `write_atomic` to prevent partial file corruption.
- **Requirement 2.4**: Restoring a backup MUST update `~/.ce-ai/state.json` to record the restoration event.

---

## 3. Scope Boundaries & Non-Goals

### In-Scope
- CLI subcommands `ce-ai backups list` and `ce-ai backups restore`.
- TUI dashboard `Backups & Restore` tab.
- Pre-restore safety snapshot creation.

### Out-of-Scope (Deferred)
- Cloud backup storage (S3 / GCS / GitHub Gists).
- Backup encryption (enforced via local filesystem permissions).

---

## 4. OpenSpec Mapping & File Impact

| Spec File | Purpose |
| :--- | :--- |
| `openspec/changes/backup_restore_management/proposal.md` | Proposal & problem framing |
| `openspec/changes/backup_restore_management/exploration.md` | Technical design alternatives |
| `openspec/changes/backup_restore_management/design.md` | Data structures & CLI contract |
| `openspec/changes/backup_restore_management/spec.md` | Formal requirements & acceptance criteria |
| `openspec/changes/backup_restore_management/tasks.md` | Task execution checklist |

### Target Code Files Affected
- `src/state/backups.rs`: `BackupEntry`, `list_backups`, `restore_backup_by_id`.
- `src/commands/backups.rs`: CLI subcommand handler.
- `src/main.rs`: Clap CLI argument parser routing.
- `src/tui.rs`: `MenuTab::Backups` interactive Ratatui tab rendering.
- `tests/cli.rs`: Integration tests for backup listing and restoration.
