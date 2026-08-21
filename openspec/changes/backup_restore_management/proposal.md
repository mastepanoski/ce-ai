# OpenSpec Proposal: Explicit Backup Management & Point-in-Time Recovery

**Feature Name**: `backup_restore_management`  
**Status**: Proposed / In Review  
**Target Milestone**: `v0.5.0`  
**Related GitHub Issue**: [#13](https://github.com/mastepanoski/ce-ai/issues/13)  
**Author**: Mauro Stepanoski  

---

## 1. Executive Summary

`ce-ai` automatically creates timestamped configuration backups under `~/.ce-ai/backups/<utc-timestamp>/` prior to any mutation. However, users currently lack a dedicated CLI subcommand (`ce-ai backups list` / `ce-ai backups restore`) or TUI dashboard interface to inspect historical backups and perform point-in-time recovery for specific harness configurations.

This specification defines the explicit backup listing, targeted snapshot recovery engine, CLI subcommand interface, and Ratatui TUI dashboard tab for configuration recovery.

---

## 2. Problem Statement & Motivation

- **Current Limitation**: When configuration issues or unwanted skill changes occur, users must either run `ce-ai uninstall` (which only restores the latest backup) or manually navigate the file system inside `~/.ce-ai/backups/`.
- **User Pain Point**: There is no command to view historical backups across different harnesses or choose a specific past snapshot to restore.
- **Goal**: Provide a safe, transparent, and atomic point-in-time recovery mechanism in both CLI and TUI modes.

---

## 3. In-Scope vs. Out-of-Scope Boundaries

### In-Scope
- `ce-ai backups list`: Command to list all stored timestamped backup snapshots with creation date, target harness, file name, and file size.
- `ce-ai backups restore <timestamp_or_id>`: Command to restore a specific historical backup snapshot to its target harness location using atomic file writes (`write_atomic`).
- **TUI Dashboard Integration**: Interactive `Backups & Restore` tab in Ratatui TUI allowing users to browse backups per harness and trigger 1-click snapshot recovery.
- **Safety Pre-Checks**: Validates backup file integrity (SHA256 checksums, JSON schema syntax) before replacing live harness configs.

### Out-of-Scope
- Off-site cloud backup syncing (S3, GCS, GitHub Gists).
- Backup encryption (host directory permissions enforce user security).

---

## 4. Success Criteria

1. `ce-ai backups list` correctly lists all backup snapshots under `~/.ce-ai/backups/` sorted newest-first.
2. `ce-ai backups restore <timestamp>` restores the specified backup file atomically and updates `state.json`.
3. TUI `Backups` tab allows filtering by harness and triggering 1-click snapshot restoration.
4. Unit and CLI integration test coverage passes 100% cleanly.
