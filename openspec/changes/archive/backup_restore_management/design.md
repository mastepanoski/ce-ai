# OpenSpec Technical Design: Backup Management & Point-in-Time Recovery

**Feature Name**: `backup_restore_management`  
**Status**: Technical Design  

---

## 1. Data Schemas & Structs

### `BackupEntry` Struct (`src/state/backups.rs`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub id: String,            // Timestamp directory name (e.g. 20260821T005512.123456Z)
    pub timestamp_rfc3339: String, // Formatted ISO 8601 string
    pub harness: String,       // Target harness identifier (opencode, claude, pi, etc.)
    pub file_name: String,     // File name (opencode.json, config.json, .cursorrules)
    pub size_bytes: u64,       // File size on disk
    pub path: PathBuf,         // Full path to backup file
}
```

---

## 2. Interface Contracts & CLI Architecture

### Subcommand Specification (`src/main.rs` & `src/commands/backups.rs`)
```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Backup management and point-in-time config recovery
    Backups(BackupsArgs),
}

#[derive(Args, Debug)]
pub struct BackupsArgs {
    #[command(subcommand)]
    pub command: BackupsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum BackupsSubcommand {
    /// List historical backup snapshots
    List {
        /// Filter backups by harness target (e.g., opencode, claude, cursor)
        #[arg(short, long)]
        harness: Option<String>,
    },
    /// Restore a specific historical backup snapshot
    Restore {
        /// Timestamp or backup ID to restore (or 'latest')
        target_id: String,
        /// Target harness override
        #[arg(short, long)]
        harness: Option<String>,
    },
}
```

---

## 3. TUI Dashboard Integration (`src/tui.rs`)

### `MenuTab::Backups` Panel
- Rendered in Ratatui sidebar menu as item #8: `Backups & Restore`.
- Displays table of stored backup snapshots (ID, Timestamp, Harness Target, Size).
- Interactive navigation (`Up`/`Down`) allowing user to select a snapshot.
- Pressing `[Enter]` prompts confirmation and restores selected snapshot to host config.
