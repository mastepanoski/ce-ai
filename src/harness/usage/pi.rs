//! Pi agent session transcript usage ingestion adapter.

use std::path::{Path, PathBuf};

use crate::capture::ledger::UsageRecord;
use crate::error::CeError;

use super::UsageAdapter;

/// Pi agent session usage ingestion adapter.
pub struct PiUsageAdapter;

impl UsageAdapter for PiUsageAdapter {
    fn harness_name(&self) -> &'static str {
        "pi"
    }

    fn is_available(&self, home: &Path) -> bool {
        resolve_pi_sessions_dir(home).is_dir()
    }

    fn read_usage(
        &self,
        home: &Path,
        author: &str,
        since: Option<&str>,
        cwd_filter: Option<&str>,
    ) -> Result<Vec<UsageRecord>, CeError> {
        let sessions_dir = resolve_pi_sessions_dir(home);
        read_usage_from_dir(&sessions_dir, since, author, cwd_filter)
    }
}

/// Resolves standard directory where Pi agent stores session files.
pub fn resolve_pi_sessions_dir(home: &Path) -> PathBuf {
    let primary = home.join(".pi/agent/sessions");
    if primary.is_dir() {
        return primary;
    }
    let secondary = home.join(".pi/sessions");
    if secondary.is_dir() {
        return secondary;
    }
    primary
}

/// Reads all JSON session files in `sessions_dir` and extracts normalized usage records.
pub fn read_usage_from_dir(
    sessions_dir: &Path,
    since: Option<&str>,
    author: &str,
    cwd_filter: Option<&str>,
) -> Result<Vec<UsageRecord>, CeError> {
    let mut records = Vec::new();
    if !sessions_dir.exists() {
        return Ok(records);
    }

    let files = walk_json_files(sessions_dir);
    for file in files {
        if let Some(mut file_records) = parse_session_file(&file, since, author, cwd_filter) {
            records.append(&mut file_records);
        }
    }

    Ok(records)
}

fn parse_session_file(
    path: &Path,
    since: Option<&str>,
    author: &str,
    cwd_filter: Option<&str>,
) -> Option<Vec<UsageRecord>> {
    let content = std::fs::read_to_string(path).ok()?;
    let root: serde_json::Value = serde_json::from_str(&content).ok()?;

    let fallback_sid = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let session_id = root
        .get("id")
        .or_else(|| root.get("sessionId"))
        .and_then(|v| v.as_str())
        .unwrap_or(&fallback_sid);

    let workspace = root
        .get("workspace")
        .or_else(|| root.get("cwd"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let cwd_basename = Path::new(workspace)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if let Some(cf) = cwd_filter {
        if cwd_basename != cf {
            return None;
        }
    }

    let session_model = root
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let mut records = Vec::new();

    let turns = root
        .get("turns")
        .or_else(|| root.get("messages"))
        .and_then(|v| v.as_array())?;

    for turn in turns {
        if let Some(rec) = extract_turn_usage(
            turn,
            session_id,
            &cwd_basename,
            author,
            session_model,
            since,
        ) {
            records.push(rec);
        }
    }

    Some(records)
}

fn extract_turn_usage(
    turn: &serde_json::Value,
    session_id: &str,
    cwd_basename: &str,
    author: &str,
    fallback_model: &str,
    since: Option<&str>,
) -> Option<UsageRecord> {
    let usage = turn.get("usage").or_else(|| turn.get("tokens"))?;

    let inp = usage
        .get("input_tokens")
        .or_else(|| usage.get("input"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let out = usage
        .get("output_tokens")
        .or_else(|| usage.get("output"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let cr = usage
        .get("cache_read")
        .or_else(|| usage.get("cache_read_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let cw = usage
        .get("cache_write")
        .or_else(|| usage.get("cache_write_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let reas = usage
        .get("reasoning")
        .or_else(|| usage.get("reasoning_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if inp == 0 && out == 0 && cr == 0 && cw == 0 && reas == 0 {
        return None;
    }

    let timestamp = turn
        .get("timestamp")
        .or_else(|| turn.get("createdAt"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if timestamp.is_empty() {
        return None;
    }

    if let Some(since_str) = since {
        if timestamp < since_str {
            return None;
        }
    }

    let model = turn
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_model);

    Some(UsageRecord {
        author: author.to_string(),
        timestamp: timestamp.to_string(),
        harness: "pi".to_string(),
        session_id: session_id.to_string(),
        cwd_basename: cwd_basename.to_string(),
        model: model.to_string(),
        input_tokens: inp,
        output_tokens: out,
        cache_read: cr,
        cache_write: cw,
        reasoning_tokens: reas,
    })
}

fn walk_json_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "json").unwrap_or(false) {
                files.push(path);
            } else if path.is_dir() {
                files.extend(walk_json_files(&path));
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi_fixture_extraction() {
        let fixture_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/usage/pi/session.json");
        let parent = fixture_path.parent().unwrap();
        let records = read_usage_from_dir(parent, None, "charlie", None).unwrap();

        assert_eq!(records.len(), 1);
        let rec = &records[0];
        assert_eq!(rec.author, "charlie");
        assert_eq!(rec.harness, "pi");
        assert_eq!(rec.session_id, "pi-sess-500");
        assert_eq!(rec.cwd_basename, "pi-project");
        assert_eq!(rec.model, "claude-3-5-sonnet");
        assert_eq!(rec.input_tokens, 1800);
        assert_eq!(rec.output_tokens, 520);
        assert_eq!(rec.cache_read, 600);
        assert_eq!(rec.cache_write, 150);
        assert_eq!(rec.reasoning_tokens, 80);
    }

    #[test]
    fn test_pi_since_filter() {
        let fixture_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/usage/pi/session.json");
        let parent = fixture_path.parent().unwrap();

        let empty =
            read_usage_from_dir(parent, Some("2026-09-28T17:00:00Z"), "charlie", None).unwrap();
        assert!(empty.is_empty());

        let matched =
            read_usage_from_dir(parent, Some("2026-09-28T15:00:00Z"), "charlie", None).unwrap();
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn test_pi_cwd_filter() {
        let fixture_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/usage/pi/session.json");
        let parent = fixture_path.parent().unwrap();

        let filtered = read_usage_from_dir(parent, None, "charlie", Some("other")).unwrap();
        assert!(filtered.is_empty());

        let matched = read_usage_from_dir(parent, None, "charlie", Some("pi-project")).unwrap();
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn test_pi_adapter_availability() {
        let temp = tempfile::tempdir().unwrap();
        let adapter = PiUsageAdapter;

        assert!(!adapter.is_available(temp.path()));

        std::fs::create_dir_all(temp.path().join(".pi/agent/sessions")).unwrap();
        assert!(adapter.is_available(temp.path()));
    }
}
