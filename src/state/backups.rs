//! Timestamped backups under `backups/<utc-ts>/` and newest-first restore (CC-3, OI-1).

use crate::error::CeError;
use crate::state::write_atomic;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackupEntry {
    pub id: String,
    pub timestamp_rfc3339: String,
    pub harness: String,
    pub file_name: String,
    pub size_bytes: u64,
    pub path: PathBuf,
}

fn backup_ts() -> String {
    Utc::now().format("%Y%m%dT%H%M%S%.6fZ").to_string()
}

/// Resolves harness name from backup file path or filename.
pub fn harness_from_filename(name: &str) -> String {
    harness_from_path(Path::new(name))
}

/// Resolves harness name from backup file path.
pub fn harness_from_path(path: &Path) -> String {
    let lower = path.to_string_lossy().to_lowercase();
    if lower.contains(".cursor") || lower.contains("cursor") {
        "cursor".to_string()
    } else if lower.contains("opencode") {
        "opencode".to_string()
    } else if lower.contains("claude") {
        "claude".to_string()
    } else if lower.contains("pi") {
        "pi".to_string()
    } else if lower.contains("copilot") {
        "copilot".to_string()
    } else if lower.contains("codex") {
        "codex".to_string()
    } else if lower.contains("grok") {
        "grok".to_string()
    } else if lower.contains("kimi") {
        "kimi".to_string()
    } else if lower.contains("gemini") || lower.contains("agy") {
        "agy".to_string()
    } else if lower.contains("deepseek") {
        "deepseek".to_string()
    } else if lower.contains("fx") {
        "fx".to_string()
    } else {
        "custom".to_string()
    }
}

/// Formats UTC directory timestamp (`%Y%m%dT%H%M%S...Z`) to clean display string.
fn format_ts_display(id: &str) -> String {
    if id.len() >= 15 && id.contains('T') {
        let parts: Vec<&str> = id.split('T').collect();
        if parts.len() == 2 && parts[0].len() == 8 {
            let date = &parts[0];
            let time_part = parts[1].trim_end_matches('Z');
            if time_part.len() >= 6 {
                return format!(
                    "{}-{}-{} {}:{}:{} UTC",
                    &date[0..4],
                    &date[4..6],
                    &date[6..8],
                    &time_part[0..2],
                    &time_part[2..4],
                    &time_part[4..6]
                );
            }
        }
    }
    id.to_string()
}

/// Copies `source` into a fresh timestamped backup dir under `root`.
pub fn backup_file(root: &Path, source: &Path) -> Result<PathBuf, CeError> {
    let raw_name = source
        .file_name()
        .ok_or_else(|| CeError::Runtime("backup source has no file name".to_string()))?
        .to_string_lossy();
    let file_name = if source.to_string_lossy().contains(".cursor") && !raw_name.contains("cursor")
    {
        format!("cursor-{raw_name}")
    } else if (source.to_string_lossy().contains(".claude")
        || source.file_name().and_then(|n| n.to_str()) == Some(".claude.json"))
        && !raw_name.contains("claude")
    {
        format!("claude-{raw_name}")
    } else {
        raw_name.to_string()
    };
    let dest = root.join(backup_ts()).join(file_name);
    write_atomic(&dest, &std::fs::read(source)?)?;
    Ok(dest)
}

/// Lists all historical backup snapshots under `root`, optionally filtered by harness target.
pub fn list_backups(
    root: &Path,
    harness_filter: Option<&str>,
) -> Result<Vec<BackupEntry>, CeError> {
    let mut entries = Vec::new();
    let read_dir = match std::fs::read_dir(root) {
        Ok(rd) => rd,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };

    for item in read_dir {
        let entry = match item {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        let sub_entries = match std::fs::read_dir(&path) {
            Ok(rd) => rd,
            Err(_) => continue,
        };

        for sub_item in sub_entries {
            let sub = match sub_item {
                Ok(s) => s,
                Err(_) => continue,
            };
            let sub_path = sub.path();
            if !sub_path.is_file() {
                continue;
            }

            let file_name = match sub_path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let harness = harness_from_filename(&file_name);
            if let Some(filter) = harness_filter {
                if filter != "all" && !harness.eq_ignore_ascii_case(filter) {
                    continue;
                }
            }

            let size_bytes = std::fs::metadata(&sub_path).map(|m| m.len()).unwrap_or(0);
            let display_ts = format_ts_display(&dir_name);

            entries.push(BackupEntry {
                id: dir_name.clone(),
                timestamp_rfc3339: display_ts,
                harness,
                file_name,
                size_bytes,
                path: sub_path,
            });
        }
    }

    entries.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(entries)
}

/// Returns the newest backup entry under `root` for a specific harness target, if any.
pub fn newest_backup_for_harness(
    root: &Path,
    harness: &str,
) -> Result<Option<BackupEntry>, CeError> {
    let entries = list_backups(root, Some(harness))?;
    Ok(entries.into_iter().next())
}

/// Restores a specific backup snapshot by ID onto `target`.
pub fn restore_backup_by_id(root: &Path, id: &str, target: &Path) -> Result<BackupEntry, CeError> {
    // Path Traversal Security Hardening
    if id.contains("..") || id.contains('/') || id.contains('\\') || id.contains('\0') {
        return Err(CeError::Usage(format!(
            "invalid backup ID '{}': path traversal sequences rejected",
            id
        )));
    }

    let backup_dir = root.join(id);
    if !backup_dir.is_dir() {
        return Err(CeError::Usage(format!(
            "backup snapshot '{}' not found under {}",
            id,
            root.display()
        )));
    }

    let sub_entries: Vec<PathBuf> = std::fs::read_dir(&backup_dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            std::fs::symlink_metadata(p)
                .map(|m| m.file_type().is_file())
                .unwrap_or(false)
        })
        .collect();

    let backup_file_path = sub_entries.first().ok_or_else(|| {
        CeError::Runtime(format!(
            "backup snapshot '{}' is empty",
            backup_dir.display()
        ))
    })?;

    let file_name = backup_file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("config.json")
        .to_string();

    let content = std::fs::read(backup_file_path)?;
    if content.is_empty() {
        return Err(CeError::Runtime(format!(
            "backup file '{}' is empty (0 bytes)",
            backup_file_path.display()
        )));
    }

    // JSON Validation for JSON-based harness configurations
    if file_name.ends_with(".json") {
        serde_json::from_slice::<serde_json::Value>(&content).map_err(|err| {
            CeError::Runtime(format!(
                "backup file '{}' contains invalid JSON: {}",
                backup_file_path.display(),
                err
            ))
        })?;
    }

    // Pre-restore safety backup of live target config if present
    if target.exists() {
        backup_file(root, target)?;
    }

    write_atomic(target, &content)?;

    let harness = harness_from_filename(&file_name);
    let size_bytes = content.len() as u64;
    let display_ts = format_ts_display(id);

    Ok(BackupEntry {
        id: id.to_string(),
        timestamp_rfc3339: display_ts,
        harness,
        file_name,
        size_bytes,
        path: backup_file_path.clone(),
    })
}

/// Restores the most recent backup dir's file onto `target`.
pub fn restore_latest(root: &Path, target: &Path) -> Result<(), CeError> {
    let dir = newest_backup_dir(root)?
        .ok_or_else(|| CeError::Runtime(format!("no backups under {}", root.display())))?;
    let dir_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| CeError::Runtime("backup dir has no name".to_string()))?;
    restore_backup_by_id(root, dir_name, target)?;
    Ok(())
}

/// Returns the most recent backup dir, if any.
pub fn newest_backup_dir(root: &Path) -> Result<Option<PathBuf>, CeError> {
    let mut dirs: Vec<PathBuf> = match std::fs::read_dir(root) {
        Ok(entries) => entries
            .map(|e| e.map(|e| e.path()))
            .filter_map(Result::ok)
            .filter(|p| p.is_dir())
            .collect(),
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

    use crate::state::backups::{backup_file, list_backups, restore_backup_by_id, restore_latest};

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
    fn list_backups_returns_sorted_entries_with_harness_filter() {
        let dir = tempdir().unwrap();
        let backups = dir.path().join("backups");
        write_file(&backups, "20260821T000001Z/opencode.json", r#"{"a":1}"#);
        write_file(&backups, "20260821T000002Z/claude.json", r#"{"b":2}"#);

        let all = list_backups(&backups, None).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].id, "20260821T000002Z");
        assert_eq!(all[0].harness, "claude");
        assert_eq!(all[1].id, "20260821T000001Z");
        assert_eq!(all[1].harness, "opencode");

        let filtered = list_backups(&backups, Some("opencode")).unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].harness, "opencode");
    }

    #[test]
    fn restore_backup_by_id_restores_snapshot_and_rejects_path_traversal() {
        let dir = tempdir().unwrap();
        let backups = dir.path().join("backups");
        let target = dir.path().join("opencode.json");

        write_file(
            &backups,
            "20260821T000001Z/opencode.json",
            r#"{"state":"v1"}"#,
        );
        write_file(
            &backups,
            "20260821T000002Z/opencode.json",
            r#"{"state":"v2"}"#,
        );
        write_file(dir.path(), "opencode.json", r#"{"state":"current"}"#);

        let restored = restore_backup_by_id(&backups, "20260821T000001Z", &target).unwrap();
        assert_eq!(restored.id, "20260821T000001Z");
        assert_eq!(fs::read_to_string(&target).unwrap(), r#"{"state":"v1"}"#);

        // Path Traversal Security Check
        assert!(restore_backup_by_id(&backups, "../etc/passwd", &target).is_err());
    }

    #[test]
    fn restore_latest_restores_newest_backup() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("opencode.json");
        let backups = dir.path().join("backups");
        write_file(dir.path(), "opencode.json", r#"{"v":"old"}"#);
        backup_file(&backups, &source).unwrap();
        write_file(dir.path(), "opencode.json", r#"{"v":"new"}"#);
        std::thread::sleep(std::time::Duration::from_millis(5));
        backup_file(&backups, &source).unwrap();
        write_file(dir.path(), "opencode.json", r#"{"v":"corrupted"}"#);
        restore_latest(&backups, &source).unwrap();
        assert_eq!(fs::read_to_string(&source).unwrap(), r#"{"v":"new"}"#);
    }

    #[test]
    fn restore_latest_without_backups_is_error() {
        let dir = tempdir().unwrap();
        let backups = dir.path().join("backups");
        assert!(restore_latest(&backups, &dir.path().join("opencode.json")).is_err());
    }
}
