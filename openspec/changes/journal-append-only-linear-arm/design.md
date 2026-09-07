# Design: Append-Only JSONL Operation Journal

## Data Structures & Schema

```rust
use std::fs::File;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::error::CeError;

/// First line of the append-only journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JournalHeader {
    command: String,
    started_at: String,
}

/// Recorded mutation appended as a single JSON line.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordedOp {
    path: PathBuf,
    applied: bool,
    /// Prior file content; `None` = the file did not exist before.
    prior: Option<Vec<u8>>,
}

/// Active operation journal holding an open append file handle.
pub struct Journal {
    path: PathBuf,
    file: File,
    fail_after_writes: Option<usize>,
    writes_seen: usize,
}
```

## Lifecycle Flow

### 1. `Journal::begin(config_dir: &Path, command: &str) -> Result<Self, CeError>`
1. Compute `path = journal_path(config_dir)`.
2. If `path.exists()`:
   - Read entire file bytes.
   - Attempt to parse as JSONL:
     - Extract non-empty lines.
     - Parse line 1 as `JournalHeader`.
     - For subsequent lines: parse as `RecordedOp`. Incomplete trailing lines are ignored.
     - Execute `Journal::rollback(&header.command, &ops)`.
   - If line 1 is not a valid `JournalHeader`, attempt fallback parsing as legacy `JournalData`. If that succeeds, rollback; otherwise emit warning:
     `warning: ignoring corrupt install journal at <path>: <err>`.
   - Remove stale journal file: `std::fs::remove_file(&path)`.
3. Check `CE_AI_FAIL_AFTER_WRITES` environment variable.
4. Prepare `JournalHeader { command, started_at }`.
5. Serialize header to compact JSON followed by `b'\n'`.
6. Write header atomically using `write_atomic(&path, &header_bytes)` to guarantee atomic file initialization.
7. Open `path` in append mode:
   ```rust
   let file = std::fs::OpenOptions::new()
       .append(true)
       .open(&path)
       .map_err(CeError::Io)?;
   ```
8. Return `Journal { path, file, fail_after_writes, writes_seen: 0 }`.

### 2. `Journal::arm(&mut self, path: &Path) -> Result<(), CeError>`
1. `self.writes_seen += 1;`
2. Capture prior content: `let prior = std::fs::read(path).ok();`
3. Construct `RecordedOp { path: path.to_path_buf(), applied: true, prior }`.
4. Serialize to single-line JSON bytes and append `b'\n'`.
5. Write directly to the open handle: `self.file.write_all(&op_bytes).map_err(CeError::Io)?;`
6. Synchronize to storage: `self.file.sync_all().map_err(CeError::Io)?;`
7. Check fault injection:
   ```rust
   if self.fail_after_writes.is_some_and(|n| self.writes_seen > n) {
       return Err(CeError::Runtime(format!(
           "injected fault (CE_AI_FAIL_AFTER_WRITES={}): aborted before mutation #{} of {}",
           self.fail_after_writes.unwrap_or_default(),
           self.writes_seen,
           path.display()
       )));
   }
   ```
8. Return `Ok(())`.

### 3. `Journal::complete(self) -> Result<(), CeError>`
1. Explicitly drop `self.file` to release file descriptors on Unix and file locks on Windows.
2. Remove `self.path`:
   ```rust
   drop(self.file);
   match std::fs::remove_file(&self.path) {
       Ok(()) => Ok(()),
       Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
       Err(e) => Err(CeError::Io(e)),
   }
   ```

### 4. `recorded_command(config_dir: &Path) -> Option<String>`
1. Read `journal_path(config_dir)` bytes (or read first line).
2. Extract first line; parse `JournalHeader`.
3. If header parses, return `Some(header.command)` ($O(1)$ lookup).
4. Fall back to parsing legacy `JournalData` for backward compatibility.
5. If both fail or file absent, return `None`.

## Recovery Algorithm (`Journal::rollback`)

```rust
fn rollback(command: &str, ops: &[RecordedOp]) {
    let mut reverted = 0usize;
    for op in ops.iter().rev().filter(|o| o.applied) {
        let res = match &op.prior {
            Some(bytes) => std::fs::write(&op.path, bytes),
            None => match std::fs::remove_file(&op.path) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(e),
            },
        };
        match res {
            Ok(()) => reverted += 1,
            Err(err) => eprintln!(
                "warning: journal rollback could not restore {}: {err}",
                op.path.display()
            ),
        }
    }
    if reverted > 0 {
        eprintln!(
            "warning: recovered incomplete '{command}' — rolled back {reverted} mutation(s)"
        );
    }
}
```
