//! Error type and process exit-code mapping for the ce-ai CLI.
//!
//! Exit-code contract (AGENTS.md invariant #7): `0` = ok, `1` = runtime,
//! `2` = usage, `3` = state, `4` = I/O, `5` = network, `6` = verification.

use std::fmt;

/// Errors that can occur while running ce-ai.
#[derive(Debug)]
pub enum CeError {
    /// Invalid CLI usage; mapped to process exit code 2.
    Usage(String),
    /// Runtime failure; mapped to exit 1.
    Runtime(String),
    /// state.json lifecycle failure (corrupt, unreadable, unpersistable);
    /// mapped to exit 3.
    State(String),
    /// Filesystem I/O failure; mapped to exit 4.
    Io(std::io::Error),
    /// Remote-fetch failure (GitHub API/tarball transport); mapped to exit 5.
    Network(String),
    /// JSON (de)serialization failure; mapped to exit 1.
    Json(serde_json::Error),
    /// Post-operation integrity check failed; mapped to exit 6.
    Verification(String),
}

impl fmt::Display for CeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CeError::Usage(msg) => write!(f, "usage error: {msg}"),
            CeError::Runtime(msg) => write!(f, "runtime error: {msg}"),
            CeError::State(msg) => write!(f, "state error: {msg}"),
            CeError::Network(msg) => write!(f, "network error: {msg}"),
            CeError::Io(err) => write!(f, "I/O error: {err}"),
            CeError::Json(err) => write!(f, "JSON error: {err}"),
            CeError::Verification(msg) => write!(f, "verification error: {msg}"),
        }
    }
}

impl std::error::Error for CeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CeError::Io(err) => Some(err),
            CeError::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CeError {
    fn from(err: std::io::Error) -> Self {
        CeError::Io(err)
    }
}

impl From<serde_json::Error> for CeError {
    fn from(err: serde_json::Error) -> Self {
        CeError::Json(err)
    }
}

impl CeError {
    /// Maps this error to a process exit code per invariant #7: 1 = runtime,
    /// 2 = usage, 3 = state, 4 = I/O, 5 = network, 6 = verification.
    pub fn exit_code(&self) -> i32 {
        match self {
            CeError::Usage(_) => 2,
            CeError::Verification(_) => 6,
            CeError::State(_) => 3,
            CeError::Io(_) => 4,
            CeError::Network(_) => 5,
            CeError::Runtime(_) | CeError::Json(_) => 1,
        }
    }
}

/// Maps a command result to a process exit code per invariant #7.
pub fn result_exit_code(result: &Result<(), CeError>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(err) => err.exit_code(),
    }
}

#[cfg(test)]
#[path = "tests/error.rs"]
mod tests;
