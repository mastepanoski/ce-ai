---
title: Backup Listing and Point-in-Time Config Restore Architecture
date: 2026-08-21
category: docs/solutions
module: state/backups
problem_type: architecture_pattern
component: tooling
severity: medium
tags:
  - backups
  - restore
  - path-traversal
  - atomic-writes
  - ratatui-tui
  - clap-cli
---

# Backup Listing and Point-in-Time Config Restore Architecture

## Context
As `ce-ai` manages configurations across multiple AI harnesses (OpenCode, Claude, Cursor, Copilot, Pi), automated sync or installation operations modify harness JSON configurations (e.g. `opencode.json`). Prior to v0.5.0, snapshots were automatically taken during installation, but users lacked a direct CLI mechanism or TUI interface to inspect historical backups and perform targeted point-in-time config restoration.

## Guidance

### 1. Hardened Point-in-Time Recovery Engine (`src/state/backups.rs`)
The recovery engine implements snapshot listing and targeted restoration with three strict security and safety guarantees:
- **Path Traversal Hardening**: Backup snapshot IDs provided via CLI or TUI are strictly validated. Any ID containing `..`, `/`, `\`, or `\0` is rejected with `CeError::Usage` before any filesystem access occurs.
- **Pre-Restore Safety Snapshot & Error Propagation**: Before overwriting live configuration files, `restore_backup_by_id` creates a fresh safety backup (`backup_file(root, target)?;`), propagating any I/O errors (such as disk full) to prevent silent overwrite failures.
- **JSON Validation & Atomic Write**: For JSON configuration files, the content is parsed via `serde_json::from_slice` prior to restoration. File mutation uses `write_atomic` (temporary file + atomic rename) to prevent partial or corrupted file writes.

### 2. Standardized CLI Flag Design (`src/commands/backups.rs`)
- Subcommands `ce-ai backups list` and `ce-ai backups restore <target_id>` accept a target harness filter flag `-t, --harness <name>`.
- Using `-t` (short for target harness) avoids flag collision with Clap's standard `-h, --help` option, ensuring clean user experience across shells.

### 3. Interactive TUI Backup Dashboard (`src/tui.rs`)
- A dedicated `MenuTab::Backups` panel renders an ASCII table of historical backups sorted newest-first.
- Users can switch harness targets using `◄`/`►` or `h`/`l` keys, navigate snapshots with `Up`/`Down` arrows, and trigger point-in-time restoration instantly via `[Enter]` or `r`.

## Why This Matters
- Prevents configuration loss or corruption when experimenting with custom plugins or sync rules across multiple harnesses.
- Eliminates manual JSON editing and backup searching in hidden system directories.
- Provides immediate 1-click recovery from within the interactive TUI environment.

## When to Apply
- When adding state management features that handle file snapshots or configuration mutations.
- When designing CLI subcommands with optional filter flags to prevent short-flag collision with `-h` (`--help`).
- When implementing file recovery operations that require atomic write safety and path traversal protection.

## Examples

### CLI Backup Listing & Restore
```bash
# List historical backups for opencode harness
ce-ai backups list -t opencode

# Restore latest snapshot
ce-ai backups restore latest -t opencode

# Restore specific snapshot ID
ce-ai backups restore 20260821T102512Z -t opencode
```

### Pre-Restore Safety & Atomic Write Logic (`src/state/backups.rs`)
```rust
// Validate JSON syntax before restoring
if file_name.ends_with(".json") {
    serde_json::from_slice::<serde_json::Value>(&content).map_err(|err| {
        CeError::Runtime(format!("backup file '{}' contains invalid JSON: {}", backup_file_path.display(), err))
    })?;
}

// Create safety backup of live target config before overwrite
if target.exists() {
    backup_file(root, target)?;
}

// Write atomically
write_atomic(target, &content)?;
```
