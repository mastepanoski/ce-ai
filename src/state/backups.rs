#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::tempdir;

    use crate::state::backups::{backup_file, restore_latest};

    #[test]
    fn backup_creates_timestamped_dir_with_copy() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("opencode.json");
        fs::write(&source, r#"{"version":1}"#).unwrap();
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
        fs::write(&source, "old").unwrap();
        backup_file(&backups, &source).unwrap();
        fs::write(&source, "new").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        backup_file(&backups, &source).unwrap();
        fs::write(&source, "corrupted").unwrap();
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
