# OpenSpec Requirements Specification: Backup Management & Point-in-Time Recovery

**Feature Name**: `backup_restore_management`  
**Status**: Specification Approved  

---

## Requirements

### Requirement BK-1: Listing Stored Backups
**WHEN** `ce-ai backups list` is executed  
**THEN** `ce-ai` MUST inspect `~/.ce-ai/backups/`, parse all valid timestamped backup directories, and print a table/list of backup snapshots sorted by timestamp (newest-first).

**WHEN** `ce-ai backups list --harness <name>` is executed  
**THEN** `ce-ai` MUST filter the returned list to display only backup snapshots associated with the specified harness.

---

### Requirement BK-2: Targeted Snapshot Restoration
**WHEN** `ce-ai backups restore <timestamp_or_id>` is executed  
**THEN** `ce-ai` MUST:
1. Locate the backup folder matching `<timestamp_or_id>` in `~/.ce-ai/backups/`.
2. Validate the integrity and JSON syntax of the target backup file.
3. Automatically create a safety backup of the current live configuration.
4. Atomically overwrite the target harness config (`write_atomic`).
5. Update `~/.ce-ai/state.json` to record the restoration event.

**WHEN** `<timestamp_or_id>` is missing or invalid  
**THEN** `ce-ai` MUST output an error with exit code `2` (Usage) listing valid backup IDs.

---

### Requirement BK-3: TUI Backups Dashboard Integration
**WHEN** the user navigates to the `Backups & Restore` tab in the Ratatui TUI dashboard  
**THEN** the TUI MUST:
1. Render an interactive list of stored backup snapshots with timestamp, harness target, and size.
2. Allow selecting a backup snapshot using `Up`/`Down` arrow keys.
3. Trigger atomic snapshot recovery upon pressing `[Enter]`.
4. Display success/error modal confirmation upon completion.
