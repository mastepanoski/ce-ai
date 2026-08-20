//! Timestamped backups under `backups/<utc-ts>/` and newest-first restore (CC-3, OI-1).

use std::path::{Path, PathBuf};
use chrono::Utc;
use crate::error::CeError;
use crate::state::write_atomic;

fn backup_ts() -> String { Utc::now().format("%Y%m%dT%H%M%S%.6fZ").to_string() }

/// Copies `source` into a fresh timestamped backup dir under `root`.
pub fn backup_file(root: &Path, source: &Path) -> Result<PathBuf, CeError> {
    let file_name = source.file_name().ok_or_else(|| CeError::Runtime("backup source has no file name".to_string()))?;
    let dest = root.join(backup_ts()).join(file_name);
    write_atomic(&dest, &std::fs::read(source)?)?;
    Ok(dest)
}

/// Restores the most recent backup dir's file onto `target`.
pub fn restore_latest(root: &Path, target: &Path) -> Result<(), CeError> {
    let dir = newest_backup_dir(root)?.ok_or_else(|| CeError::Runtime(format!("no backups under {}", root.display())))?;
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)?.map(|e| e.map(|e| e.path())).collect::<Result<_, _>>()?;
    files.sort();
    let newest = files.pop().ok_or_else(|| CeError::Runtime("backup dir is empty".to_string()))?;
    write_atomic(target, &std::fs::read(newest)?)
}

/// Returns the most recent backup dir, if any.
pub fn newest_backup_dir(root: &Path) -> Result<Option<PathBuf>, CeError> {
    let mut dirs: Vec<PathBuf> = match std::fs::read_dir(root) {
        Ok(entries) => entries.map(|e| e.map(|e| e.path())).filter_map(Result::ok).filter(|p| p.is_dir()).collect(),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.into()),
    };
    dirs.sort();
    Ok(dirs.pop())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    use crate::state::backups::{backup_file, restore_latest};

    fn write_file(root: &Path, rel: &str, content: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn backup_creates_timestamped_dir_with_copy() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("opencode.json");
        write_file(dir.path(), "opencode.json", r#"{"version":1}"#);
        let dest = backup_file(&dir.path().join("backups"), &source).unwrap();
        assert!(dest.starts_with(dir.path().join("backups")));
        assert_eq!(fs::read_to_string(&dest).unwrap(), r#"{"version":1}"#);
        assert_eq!(fs::read_to_string(&source).unwrap(), r#"{"version":1}"#);
    }

    #[test]
    fn restore_latest_restores_newest_backup() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("opencode.json");
        let backups = dir.path().join("backups");
        write_file(dir.path(), "opencode.json", "old");
        backup_file(&backups, &source).unwrap();
        write_file(dir.path(), "opencode.json", "new");
        std::thread::sleep(std::time::Duration::from_millis(5));
        backup_file(&backups, &source).unwrap();
        write_file(dir.path(), "opencode.json", "corrupted");
        restore_latest(&backups, &source).unwrap();
        assert_eq!(fs::read_to_string(&source).unwrap(), "new");
    }

    #[test]
    fn restore_latest_without_backups_is_error() {
        let dir = tempdir().unwrap();
        let backups = dir.path().join("backups");
        assert!(restore_latest(&backups, &dir.path().join("opencode.json")).is_err());
    }
}