# OpenSpec Exploration: Backup Management & Point-in-Time Recovery

**Feature Name**: `backup_restore_management`  
**Status**: Completed Exploration  

---

## 1. Technical Investigation

### Current Backup Directory Structure
`ce-ai` currently stores backups under `~/.ce-ai/backups/`:
```
~/.ce-ai/backups/
├── 20260821T005512.123456Z/
│   └── opencode.json
├── 20260821T011530.654321Z/
│   └── claude.json
└── 20260821T023000.999999Z/
    └── .cursorrules
```

### Key Technical Challenges & Options

#### Option A: Directory-based Timestamp Indexing (Selected)
- Read `~/.ce-ai/backups/` directory entries.
- Parse directory names formatted as `%Y%m%dT%H%M%S%.6fZ`.
- Inspect contained configuration files to determine target harness and file size.
- *Pros*: Zero external index file dependency; resilient against direct filesystem edits.
- *Cons*: Requires reading metadata of directory children.

#### Option B: Manifest-indexed Backup Registry
- Maintain a central `backups.json` manifest recording backup IDs, timestamps, and target paths.
- *Pros*: Fast metadata lookup.
- *Cons*: Subject to manifest drift if users manually delete backup folders.

**Decision**: Option A (Directory-based Timestamp Indexing) combined with atomic write safety guarantees.

---

## 2. Safety & Validation Controls

- **Atomic Writes**: Restoring a backup must write to a temporary file before renaming (`crate::state::write_atomic`).
- **Validation Before Restore**:
  - Check file non-emptiness.
  - Parse JSON structure (for JSON-based harnesses like OpenCode, Claude, Pi, Kimi).
  - Verify path safety before replacing live configuration.
- **Pre-Restore Backup**: Automatically create a new backup snapshot of the current live configuration before restoring an older backup, ensuring recovery is always undoable.
