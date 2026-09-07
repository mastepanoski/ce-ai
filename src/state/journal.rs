//! Operation journal for transactional multi-file commands (Issue #166, #312).
//!
//! Every tracked filesystem mutation records its prior content in an
//! append-only journal **before** being performed. A crashed or failing command
//! leaves the journal behind; the next install/sync rolls applied mutations back
//! in reverse (deterministic recovery) before starting fresh, and `ce-ai doctor`
//! flags the presence of a stale journal.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CeError;
use crate::state::write_atomic;

/// Well-known journal location relative to the ce-ai config dir.
pub fn journal_path(config_dir: &Path) -> PathBuf {
    config_dir.join("install-journal.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JournalHeader {
    command: String,
    started_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordedOp {
    path: PathBuf,
    applied: bool,
    /// Prior file content; `None` = the file did not exist before.
    prior: Option<Vec<u8>>,
}

/// Legacy format used prior to append-only JSONL (v1.44.1 and earlier).
#[derive(Debug, Serialize, Deserialize)]
struct LegacyJournalData {
    command: String,
    started_at: String,
    ops: Vec<RecordedOp>,
}

/// Active operation journal. Create via [`Journal::begin`]; finish with
/// [`Journal::complete`] after the command's final state persistence.
pub struct Journal {
    path: PathBuf,
    file: File,
    fail_after_writes: Option<usize>,
    writes_seen: usize,
}

impl Journal {
    /// Rolls back any stale journal (reverse order, best-effort with stderr
    /// warnings), starts a fresh one for `command`, and honors the
    /// `CE_AI_FAIL_AFTER_WRITES=<N>` fault-injection variable.
    pub fn begin(config_dir: &Path, command: &str) -> Result<Self, CeError> {
        let path = journal_path(config_dir);
        if path.exists() {
            Self::recover_stale_journal(&path);
            let _ = std::fs::remove_file(&path);
        }

        let fail_after_writes = std::env::var("CE_AI_FAIL_AFTER_WRITES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok());

        let header = JournalHeader {
            command: command.to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
        };
        let mut header_bytes = serde_json::to_vec(&header).map_err(CeError::Json)?;
        header_bytes.push(b'\n');

        // Atomically initialize the journal file with the header line.
        write_atomic(&path, &header_bytes)?;

        // Maintain an open handle in append mode for linear-time arming.
        let file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .map_err(CeError::Io)?;

        Ok(Self {
            path,
            file,
            fail_after_writes,
            writes_seen: 0,
        })
    }

    /// Reconstructs and rolls back mutations from a stale journal file.
    fn recover_stale_journal(path: &Path) {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(err) => {
                eprintln!(
                    "warning: ignoring unreadable install journal at {}: {err}",
                    path.display()
                );
                return;
            }
        };

        let mut lines = bytes
            .split(|&b| b == b'\n')
            .map(|l| l.strip_suffix(b"\r").unwrap_or(l))
            .filter(|l| !l.iter().all(|b| b.is_ascii_whitespace()));

        let first_line = match lines.next() {
            Some(line) => line,
            None => {
                eprintln!(
                    "warning: ignoring corrupt install journal at {}: file is empty",
                    path.display()
                );
                return;
            }
        };

        match serde_json::from_slice::<JournalHeader>(first_line) {
            Ok(header) => {
                let mut ops = Vec::new();
                for line in lines {
                    match serde_json::from_slice::<RecordedOp>(line) {
                        Ok(op) => ops.push(op),
                        Err(err) => {
                            // Mid-write partial lines on crash are safely ignored;
                            // earlier valid records are preserved for rollback.
                            eprintln!(
                                "warning: ignoring incomplete journal entry at {}: {err}",
                                path.display()
                            );
                        }
                    }
                }
                Journal::rollback(&header.command, &ops);
            }
            Err(_) => {
                // Check if this is a legacy single-blob journal from earlier ce-ai versions.
                match serde_json::from_slice::<LegacyJournalData>(&bytes) {
                    Ok(legacy) => Journal::rollback(&legacy.command, &legacy.ops),
                    Err(err) => {
                        eprintln!(
                            "warning: ignoring corrupt install journal at {}: {err}",
                            path.display()
                        );
                    }
                }
            }
        }
    }

    /// Best-effort reverse rollback of every applied mutation.
    fn rollback(command: &str, ops: &[RecordedOp]) {
        let mut reverted = 0usize;
        for op in ops.iter().rev().filter(|o| o.applied) {
            let res = match &op.prior {
                Some(bytes) => std::fs::write(&op.path, bytes),
                // File was created by the command: remove it. Missing is fine.
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

    /// Arms a mutation: captures prior content, marks the op applied and
    /// appends the op record directly to the journal file **before** the caller
    /// mutates the path. When fault injection triggers, returns an error
    /// instead so the caller aborts mid-sequence with the journal intact for recovery.
    pub fn arm(&mut self, path: &Path) -> Result<(), CeError> {
        self.writes_seen += 1;
        let prior = std::fs::read(path).ok();
        let op = RecordedOp {
            path: path.to_path_buf(),
            applied: true,
            prior,
        };

        let mut op_bytes = serde_json::to_vec(&op).map_err(CeError::Json)?;
        op_bytes.push(b'\n');
        self.file.write_all(&op_bytes).map_err(CeError::Io)?;
        self.file.sync_all().map_err(CeError::Io)?;

        if self.fail_after_writes.is_some_and(|n| self.writes_seen > n) {
            return Err(CeError::Runtime(format!(
                "injected fault (CE_AI_FAIL_AFTER_WRITES={}): aborted before mutation #{} of {}",
                self.fail_after_writes.unwrap_or_default(),
                self.writes_seen,
                path.display()
            )));
        }
        Ok(())
    }

    /// Removes the journal after the command's final state persistence.
    pub fn complete(self) -> Result<(), CeError> {
        // Drop the file handle first to release OS file locks (required on Windows).
        drop(self.file);
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(CeError::Io(e)),
        }
    }
}

/// Reads the recorded command name from a journal file, best-effort.
pub fn recorded_command(config_dir: &Path) -> Option<String> {
    let bytes = std::fs::read(journal_path(config_dir)).ok()?;
    let first_line = bytes
        .split(|&b| b == b'\n')
        .map(|l| l.strip_suffix(b"\r").unwrap_or(l))
        .find(|l| !l.iter().all(|b| b.is_ascii_whitespace()))?;

    if let Ok(header) = serde_json::from_slice::<JournalHeader>(first_line) {
        return Some(header.command);
    }
    // Fallback for legacy format
    let data: LegacyJournalData = serde_json::from_slice(&bytes).ok()?;
    Some(data.command)
}

#[cfg(test)]
#[path = "tests/journal.rs"]
mod tests;
