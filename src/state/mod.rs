//! Persistent state: state.json, profiles, snapshots, backups, and the sync diff engine.
//! Wired into CLI commands in later PRs; until then items are exercised by unit tests.

pub mod backups;
pub mod diff;
pub mod profiles;
// `state::state` holds the State type; module_inception is intentional.
#[allow(clippy::module_inception)]
pub mod state;

use crate::error::CeError;
use std::io::Write;
use std::path::Path;

/// Reads a harness JSON config file. Missing file → empty object; invalid
/// JSON → hard-fail with fix guidance (D4) — never silently overwrite a
/// broken config. Neutral layer shared by every harness backend.
pub fn read_config(path: &Path) -> Result<serde_json::Value, CeError> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).map_err(|err| {
            CeError::Runtime(format!(
                "{} is not valid JSON: {err}. Refusing to overwrite it. Fix the file manually, then re-run.",
                path.display()
            ))
        }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(serde_json::json!({})),
        Err(err) => Err(err.into()),
    }
}

/// Atomic write via a temp file + rename in the same directory.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), CeError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("file");
    let tmp = parent.join(format!(".{name}.tmp{}", std::process::id()));
    let mut file = std::fs::File::create(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Reports a best-effort removal without swallowing real failures: `Ok` and
/// `NotFound` stay silent (removing an absent artifact is success); any
/// other error prints a stderr warning naming the path. Returns true when
/// a warning was emitted.
pub(crate) fn report_best_effort_remove(path: impl AsRef<Path>, res: std::io::Result<()>) -> bool {
    match res {
        Ok(()) => false,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => false,
        Err(err) => {
            eprintln!(
                "warning: cleanup of {} failed: {err}",
                path.as_ref().display()
            );
            true
        }
    }
}

/// Reports a best-effort config rewrite without hiding failures. Returns
/// true when a warning was emitted.
pub(crate) fn report_best_effort_write(path: impl AsRef<Path>, res: Result<(), CeError>) -> bool {
    match res {
        Ok(()) => false,
        Err(err) => {
            eprintln!(
                "warning: update of {} failed: {err}",
                path.as_ref().display()
            );
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{report_best_effort_remove, report_best_effort_write};
    use tempfile::tempdir;

    #[test]
    fn remove_reporter_is_silent_on_ok_and_not_found() {
        assert!(!report_best_effort_remove("x", Ok(())));

        let tmp = tempdir().unwrap();
        let absent = tmp.path().join("absent.txt");
        assert!(!report_best_effort_remove(
            &absent,
            std::fs::remove_file(&absent)
        ));
    }

    #[test]
    fn remove_reporter_warns_on_unexpected_io_error() {
        // Removing a non-empty directory fails with DirectoryNotEmpty —
        // an unexpected condition for a best-effort cleanup.
        let dir = tempdir().unwrap();
        let nested = dir.path().join("nested");
        std::fs::create_dir_all(nested.join("child")).unwrap();
        assert!(report_best_effort_remove(
            dir.path().join("nested"),
            std::fs::remove_dir(&nested)
        ));
    }

    #[test]
    fn write_reporter_warns_on_error_and_stays_silent_on_ok() {
        assert!(!report_best_effort_write("x", Ok(())));
        assert!(report_best_effort_write(
            "/unreachable/path.json",
            Err(crate::error::CeError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "denied",
            )))
        ));
    }
}
