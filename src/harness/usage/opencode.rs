//! OpenCode session transcript usage ingestion adapter.

use std::path::{Path, PathBuf};

use crate::capture::ledger::UsageRecord;
use crate::error::CeError;

use super::UsageAdapter;

/// OpenCode session usage ingestion adapter.
pub struct OpenCodeUsageAdapter;

impl UsageAdapter for OpenCodeUsageAdapter {
    fn harness_name(&self) -> &'static str {
        "opencode"
    }

    fn is_available(&self, home: &Path) -> bool {
        !resolve_opencode_dirs(home).is_empty()
    }

    fn read_usage(
        &self,
        home: &Path,
        author: &str,
        since: Option<&str>,
        cwd_filter: Option<&str>,
    ) -> Result<Vec<UsageRecord>, CeError> {
        let mut records = Vec::new();
        for dir in resolve_opencode_dirs(home) {
            records.extend(read_usage_from_dir(&dir, since, author, cwd_filter)?);
        }
        Ok(records)
    }
}

/// Resolves standard directories where OpenCode stores session state.
pub fn resolve_opencode_dirs(home: &Path) -> Vec<PathBuf> {
    let candidates = [
        home.join(".local/share/opencode/sessions"),
        home.join(".config/opencode/sessions"),
        home.join("AppData/Roaming/opencode/sessions"),
        home.join("AppData/Local/opencode/sessions"),
    ];
    candidates.into_iter().filter(|p| p.is_dir()).collect()
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

    let files = walk_session_files(sessions_dir);
    for file in files {
        if let Some(mut file_records) = parse_session_file(&file, since, author, cwd_filter) {
            records.append(&mut file_records);
        }
    }

    Ok(records)
}

/// Parses a single OpenCode session JSON file.
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

    // Check both root `messages` and root `events`
    if let Some(messages) = root.get("messages").and_then(|v| v.as_array()) {
        for msg in messages {
            if let Some(record) =
                extract_message_usage(msg, session_id, &cwd_basename, author, session_model, since)
            {
                records.push(record);
            }
        }
    } else if let Some(events) = root.get("events").and_then(|v| v.as_array()) {
        for ev in events {
            let msg = ev.get("message").unwrap_or(ev);
            if let Some(record) =
                extract_message_usage(msg, session_id, &cwd_basename, author, session_model, since)
            {
                records.push(record);
            }
        }
    }

    Some(records)
}

fn extract_message_usage(
    msg: &serde_json::Value,
    session_id: &str,
    cwd_basename: &str,
    author: &str,
    fallback_model: &str,
    since: Option<&str>,
) -> Option<UsageRecord> {
    let tokens = msg.get("tokens").or_else(|| msg.get("usage"))?;

    let inp = tokens
        .get("input")
        .or_else(|| tokens.get("input_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let out = tokens
        .get("output")
        .or_else(|| tokens.get("output_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let cr = tokens
        .get("cache_read")
        .or_else(|| tokens.get("cache_read_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let cw = tokens
        .get("cache_write")
        .or_else(|| tokens.get("cache_write_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let reas = tokens
        .get("reasoning")
        .or_else(|| tokens.get("reasoning_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if inp == 0 && out == 0 && cr == 0 && cw == 0 && reas == 0 {
        return None;
    }

    let timestamp = msg
        .get("timestamp")
        .or_else(|| msg.get("createdAt"))
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

    let model = msg
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_model);

    Some(UsageRecord {
        author: author.to_string(),
        timestamp: timestamp.to_string(),
        harness: "opencode".to_string(),
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

fn walk_session_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "json").unwrap_or(false) {
                files.push(path);
            } else if path.is_dir() {
                files.extend(walk_session_files(&path));
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opencode_fixture_extraction() {
        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/usage/opencode/session.json");
        let parent = fixture_path.parent().unwrap();
        let records = read_usage_from_dir(parent, None, "alice", None).unwrap();

        assert_eq!(records.len(), 1);
        let rec = &records[0];
        assert_eq!(rec.author, "alice");
        assert_eq!(rec.harness, "opencode");
        assert_eq!(rec.session_id, "session-oc-9876");
        assert_eq!(rec.cwd_basename, "sample-app");
        assert_eq!(rec.model, "claude-3-5-sonnet");
        assert_eq!(rec.input_tokens, 1250);
        assert_eq!(rec.output_tokens, 420);
        assert_eq!(rec.cache_read, 300);
        assert_eq!(rec.cache_write, 100);
        assert_eq!(rec.reasoning_tokens, 60);
    }

    #[test]
    fn test_opencode_since_filter() {
        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/usage/opencode/session.json");
        let parent = fixture_path.parent().unwrap();

        // Future timestamp should filter out the 10:01:30Z record
        let empty =
            read_usage_from_dir(parent, Some("2026-09-28T11:00:00Z"), "alice", None).unwrap();
        assert!(empty.is_empty());

        // Earlier timestamp should include it
        let matched =
            read_usage_from_dir(parent, Some("2026-09-28T10:00:00Z"), "alice", None).unwrap();
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn test_opencode_cwd_filter() {
        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/usage/opencode/session.json");
        let parent = fixture_path.parent().unwrap();

        let filtered = read_usage_from_dir(parent, None, "alice", Some("other-app")).unwrap();
        assert!(filtered.is_empty());

        let matched = read_usage_from_dir(parent, None, "alice", Some("sample-app")).unwrap();
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn test_opencode_adapter_availability() {
        let temp = tempfile::tempdir().unwrap();
        let adapter = OpenCodeUsageAdapter;

        assert!(!adapter.is_available(temp.path()));

        std::fs::create_dir_all(temp.path().join(".local/share/opencode/sessions")).unwrap();
        assert!(adapter.is_available(temp.path()));
    }
}
