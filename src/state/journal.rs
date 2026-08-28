//! Operation journal for transactional multi-file commands (Issue #166).
//!
//! Every tracked filesystem mutation records its prior content in an
//! atomically-persisted journal **before** being performed. A crashed or
//! failing command leaves the journal behind; the next install/sync rolls
//! applied mutations back in reverse (deterministic recovery) before
//! starting fresh, and `ce-ai doctor` flags the presence of a stale journal.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CeError;
use crate::state::write_atomic;

/// Well-known journal location relative to the ce-ai config dir.
pub fn journal_path(config_dir: &Path) -> PathBuf {
    config_dir.join("install-journal.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecordedOp {
    path: PathBuf,
    applied: bool,
    /// Prior file content; `None` = the file did not exist before.
    prior: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JournalData {
    command: String,
    started_at: String,
    ops: Vec<RecordedOp>,
}

/// Active operation journal. Create via [`Journal::begin`]; finish with
/// [`Journal::complete`] after the command's final state persistence.
pub struct Journal {
    path: PathBuf,
    data: JournalData,
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
            match std::fs::read(&path)
                .map_err(|e| CeError::State(e.to_string()))
                .and_then(|b| {
                    serde_json::from_slice::<JournalData>(&b)
                        .map_err(|e| CeError::State(e.to_string()))
                }) {
                Ok(data) => Journal::rollback(&data),
                Err(err) => eprintln!(
                    "warning: ignoring corrupt install journal at {}: {err}",
                    path.display()
                ),
            }
            let _ = std::fs::remove_file(&path);
        }

        let fail_after_writes = std::env::var("CE_AI_FAIL_AFTER_WRITES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok());

        let data = JournalData {
            command: command.to_string(),
            started_at: chrono::Utc::now().to_rfc3339(),
            ops: Vec::new(),
        };
        write_atomic(
            &path,
            &serde_json::to_vec_pretty(&data).map_err(CeError::Json)?,
        )?;
        Ok(Self {
            path,
            data,
            fail_after_writes,
            writes_seen: 0,
        })
    }

    /// Best-effort reverse rollback of every applied mutation.
    fn rollback(data: &JournalData) {
        let mut reverted = 0usize;
        for op in data.ops.iter().rev().filter(|o| o.applied) {
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
                "warning: recovered incomplete '{}' — rolled back {reverted} mutation(s)",
                data.command
            );
        }
    }

    /// Arms a mutation: captures prior content, marks the op applied and
    /// persists the journal **before** the caller mutates the path. When
    /// fault injection triggers, returns an error instead so the caller
    /// aborts mid-sequence with the journal intact for recovery.
    pub fn arm(&mut self, path: &Path) -> Result<(), CeError> {
        self.writes_seen += 1;
        let prior = std::fs::read(path).ok();
        self.data.ops.push(RecordedOp {
            path: path.to_path_buf(),
            applied: true,
            prior,
        });
        self.persist()?;
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
        match std::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(CeError::Io(e)),
        }
    }

    fn persist(&self) -> Result<(), CeError> {
        write_atomic(
            &self.path,
            &serde_json::to_vec_pretty(&self.data).map_err(CeError::Json)?,
        )
    }
}

/// Reads the recorded command name from a journal file, best-effort.
pub fn recorded_command(config_dir: &Path) -> Option<String> {
    let bytes = std::fs::read(journal_path(config_dir)).ok()?;
    let data: JournalData = serde_json::from_slice(&bytes).ok()?;
    Some(data.command)
}

#[cfg(test)]
#[path = "tests/journal.rs"]
mod tests;
