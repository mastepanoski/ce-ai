//! Error type and process exit-code mapping for the ce-ai CLI.
//!
//! Exit-code contract (design.md §File Changes): `0` = ok, `1` = runtime
//! error, `2` = usage error.

use std::fmt;

/// Errors that can occur while running ce-ai.
#[derive(Debug)]
pub enum CeError {
    /// Invalid CLI usage; mapped to process exit code 2.
    Usage(String),
    /// Runtime failure (state I/O, network, source fetch); mapped to exit 1.
    Runtime(String),
    /// Filesystem I/O failure; mapped to exit 1.
    Io(std::io::Error),
    /// JSON (de)serialization failure; mapped to exit 1.
    Json(serde_json::Error),
}

impl fmt::Display for CeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CeError::Usage(msg) => write!(f, "usage error: {msg}"),
            CeError::Runtime(msg) => write!(f, "runtime error: {msg}"),
            CeError::Io(err) => write!(f, "I/O error: {err}"),
            CeError::Json(err) => write!(f, "JSON error: {err}"),
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
    /// Maps this error to a process exit code: 1 = runtime, 2 = usage.
    pub fn exit_code(&self) -> i32 {
        unimplemented!("exit_code mapping not implemented yet")
    }
}

/// Maps a command result to a process exit code: 0 = ok, 1 = runtime,
/// 2 = usage.
pub fn result_exit_code(result: &Result<(), CeError>) -> i32 {
    unimplemented!("result_exit_code mapping not implemented yet")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_maps_to_exit_zero() {
        assert_eq!(result_exit_code(&Ok(())), 0);
    }

    #[test]
    fn runtime_errors_map_to_exit_one() {
        assert_eq!(CeError::Runtime("boom".into()).exit_code(), 1);
        let io = std::io::Error::new(std::io::ErrorKind::Other, "disk full");
        assert_eq!(CeError::Io(io).exit_code(), 1);
        let json = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        assert_eq!(CeError::Json(json).exit_code(), 1);
    }

    #[test]
    fn usage_error_maps_to_exit_two() {
        assert_eq!(CeError::Usage("missing subcommand".into()).exit_code(), 2);
    }

    #[test]
    fn display_includes_error_context() {
        assert!(CeError::Usage("missing subcommand".into())
            .to_string()
            .contains("missing subcommand"));
        assert!(CeError::Runtime("no state file".into())
            .to_string()
            .contains("no state file"));
    }
}