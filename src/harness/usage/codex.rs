//! Codex session transcript usage ingestion adapter.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::capture::ledger::UsageRecord;
use crate::error::CeError;

use super::UsageAdapter;

/// Codex transcript usage ingestion adapter.
pub struct CodexUsageAdapter;

impl UsageAdapter for CodexUsageAdapter {
    fn harness_name(&self) -> &'static str {
        "codex"
    }

    fn is_available(&self, home: &Path) -> bool {
        resolve_codex_sessions_dir(home).is_dir()
    }

    fn read_usage(
        &self,
        home: &Path,
        author: &str,
        since: Option<&str>,
        cwd_filter: Option<&str>,
    ) -> Result<Vec<UsageRecord>, CeError> {
        let sessions_dir = resolve_codex_sessions_dir(home);
        read_usage_from_dir(&sessions_dir, since, author, cwd_filter)
    }
}

/// Resolves the sessions directory for Codex transcripts.
pub fn resolve_codex_sessions_dir(home: &Path) -> PathBuf {
    if let Some(config_env) = std::env::var_os("CODEX_HOME") {
        return PathBuf::from(config_env).join("sessions");
    }
    home.join(".codex/sessions")
}

/// Reads all JSONL files under `sessions_dir` and extracts normalized usage records.
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

    let files = walk_jsonl_files(sessions_dir);
    for file in files {
        let fallback_sid = file
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        if let Ok(f) = std::fs::File::open(&file) {
            let reader = BufReader::new(f);
            for line in reader.lines().map_while(Result::ok) {
                if let Some(record) =
                    extract_usage_line(&line, &fallback_sid, author, since, cwd_filter)
                {
                    records.push(record);
                }
            }
        }
    }

    Ok(records)
}

fn extract_usage_line(
    line: &str,
    fallback_sid: &str,
    author: &str,
    since: Option<&str>,
    cwd_filter: Option<&str>,
) -> Option<UsageRecord> {
    let d: serde_json::Value = serde_json::from_str(line).ok()?;

    let timestamp = d
        .get("timestamp")
        .or_else(|| d.get("createdAt"))
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

    let cwd = d
        .get("cwd")
        .or_else(|| d.get("workspace"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let cwd_basename = Path::new(cwd)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if let Some(cf) = cwd_filter {
        if cwd_basename != cf {
            return None;
        }
    }

    let session_id = d
        .get("sessionId")
        .or_else(|| d.get("session_id"))
        .or_else(|| d.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_sid);

    let resp = d.get("response").or_else(|| d.get("message")).unwrap_or(&d);

    let usage = resp.get("usage").or_else(|| d.get("usage"))?;

    let model = resp
        .get("model")
        .or_else(|| d.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let inp = usage
        .get("prompt_tokens")
        .or_else(|| usage.get("input_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let out = usage
        .get("completion_tokens")
        .or_else(|| usage.get("output_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let cr = usage
        .get("prompt_tokens_details")
        .and_then(|details| details.get("cached_tokens"))
        .or_else(|| usage.get("cache_read"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let cw = usage
        .get("cache_creation_input_tokens")
        .or_else(|| usage.get("cache_write"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let reas = usage
        .get("completion_tokens_details")
        .and_then(|details| details.get("reasoning_tokens"))
        .or_else(|| usage.get("reasoning"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if inp == 0 && out == 0 && cr == 0 && cw == 0 && reas == 0 {
        return None;
    }

    Some(UsageRecord {
        author: author.to_string(),
        timestamp: timestamp.to_string(),
        harness: "codex".to_string(),
        session_id: session_id.to_string(),
        cwd_basename,
        model: model.to_string(),
        input_tokens: inp,
        output_tokens: out,
        cache_read: cr,
        cache_write: cw,
        reasoning_tokens: reas,
    })
}

fn walk_jsonl_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                files.push(path);
            } else if path.is_dir() {
                files.extend(walk_jsonl_files(&path));
            }
        }
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_fixture_extraction() {
        let fixture_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/usage/codex/session.jsonl");
        let parent = fixture_path.parent().unwrap();
        let records = read_usage_from_dir(parent, None, "bob", None).unwrap();

        assert_eq!(records.len(), 1);
        let rec = &records[0];
        assert_eq!(rec.author, "bob");
        assert_eq!(rec.harness, "codex");
        assert_eq!(rec.session_id, "codex-sess-100");
        assert_eq!(rec.cwd_basename, "codex-app");
        assert_eq!(rec.model, "o3-mini");
        assert_eq!(rec.input_tokens, 2500);
        assert_eq!(rec.output_tokens, 800);
        assert_eq!(rec.cache_read, 1200);
        assert_eq!(rec.cache_write, 0);
        assert_eq!(rec.reasoning_tokens, 400);
    }

    #[test]
    fn test_codex_since_filter() {
        let fixture_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/usage/codex/session.jsonl");
        let parent = fixture_path.parent().unwrap();

        let empty = read_usage_from_dir(parent, Some("2026-09-28T13:00:00Z"), "bob", None).unwrap();
        assert!(empty.is_empty());

        let matched =
            read_usage_from_dir(parent, Some("2026-09-28T11:00:00Z"), "bob", None).unwrap();
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn test_codex_cwd_filter() {
        let fixture_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/usage/codex/session.jsonl");
        let parent = fixture_path.parent().unwrap();

        let filtered = read_usage_from_dir(parent, None, "bob", Some("different-app")).unwrap();
        assert!(filtered.is_empty());

        let matched = read_usage_from_dir(parent, None, "bob", Some("codex-app")).unwrap();
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn test_codex_adapter_availability() {
        let temp = tempfile::tempdir().unwrap();
        let adapter = CodexUsageAdapter;

        assert!(!adapter.is_available(temp.path()));

        std::fs::create_dir_all(temp.path().join(".codex/sessions")).unwrap();
        assert!(adapter.is_available(temp.path()));
    }
}
