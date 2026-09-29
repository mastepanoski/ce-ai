//! Incremental Claude Code JSONL usage reader.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::capture::ledger::UsageRecord;
use crate::error::CeError;

use super::UsageAdapter;

/// Claude Code transcript usage ingestion adapter.
pub struct ClaudeUsageAdapter;

impl UsageAdapter for ClaudeUsageAdapter {
    fn harness_name(&self) -> &'static str {
        "claude"
    }

    fn is_available(&self, home: &Path) -> bool {
        home.join(".claude/projects").is_dir()
    }

    fn read_usage(
        &self,
        home: &Path,
        author: &str,
        since: Option<&str>,
        cwd_filter: Option<&str>,
    ) -> Result<Vec<UsageRecord>, CeError> {
        let projects_dir = home.join(".claude/projects");
        read_usage_from_dir(&projects_dir, since, author, cwd_filter)
    }
}

/// Extracts (timestamp, model, input, cache_read, cache_write, output) from
/// a JSONL transcript line that carries usage fields.
fn extract_usage(line: &str, cwd_basename: &str, author: &str) -> Option<UsageRecord> {
    let d: serde_json::Value = serde_json::from_str(line).ok()?;
    let msg = d.get("message")?;
    let usage = msg.get("usage")?;
    let ts = d.get("timestamp")?.as_str()?;
    let model = msg
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown");
    let sid = d
        .get("sessionId")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");

    let inp = usage
        .get("input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let cr = usage
        .get("cache_read_input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let cw = usage
        .get("cache_creation_input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let out = usage
        .get("output_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if inp == 0 && out == 0 && cr == 0 && cw == 0 {
        return None;
    }

    Some(UsageRecord {
        author: author.to_string(),
        timestamp: ts.to_string(),
        harness: "claude".to_string(),
        session_id: sid.to_string(),
        cwd_basename: cwd_basename.to_string(),
        model: model.to_string(),
        input_tokens: inp,
        output_tokens: out,
        cache_read: cr,
        cache_write: cw,
        reasoning_tokens: 0,
    })
}

/// Reads all JSONL files under `projects_dir` newer than `since` (ISO timestamp string).
/// Returns extracted records normalized as `UsageRecord`s.
pub fn read_usage_from_dir(
    projects_dir: &Path,
    since: Option<&str>,
    author: &str,
    cwd_filter: Option<&str>,
) -> Result<Vec<UsageRecord>, CeError> {
    let mut records = Vec::new();
    if !projects_dir.exists() {
        return Ok(records);
    }
    for entry in walkdir(projects_dir) {
        if !entry.extension().map(|e| e == "jsonl").unwrap_or(false) {
            continue;
        }
        let file = std::fs::File::open(&entry)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            if let Some(ts) = since {
                // Skip lines older than the marker.
                let d = match serde_json::from_str::<serde_json::Value>(&line) {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                let t = d.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
                if !t.is_empty() && t < ts {
                    continue;
                }
            }
            let fname = entry.to_string_lossy();
            let base = fname.rsplit('/').next().unwrap_or("");
            let base = base.trim_end_matches(".jsonl");
            let parts: Vec<&str> = base.split("--").collect();
            let proj = parts.last().copied().unwrap_or("").trim_matches('-');
            if let Some(f) = cwd_filter {
                if proj != f {
                    continue;
                }
            }
            if let Some(rec) = extract_usage(&line, proj, author) {
                records.push(rec);
            }
        }
    }
    Ok(records)
}

/// Backwards-compatible alias for `read_usage_from_dir`.
pub fn read_usage(
    projects_dir: &Path,
    since: Option<&str>,
    author: &str,
    cwd_filter: Option<&str>,
) -> Result<Vec<UsageRecord>, CeError> {
    read_usage_from_dir(projects_dir, since, author, cwd_filter)
}

/// Shallow recursive JSONL collector.
fn walkdir(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walkdir(&p));
            } else if p.extension().map(|e| e == "jsonl").unwrap_or(false) {
                out.push(p);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_valid_claude_usage_line() {
        let line = r#"{"timestamp":"2026-09-28T20:00:00Z","sessionId":"sess-abc","message":{"model":"claude-3-5-sonnet-20241022","usage":{"input_tokens":120,"output_tokens":45,"cache_read_input_tokens":30,"cache_creation_input_tokens":15}}}"#;
        let record = extract_usage(line, "my-repo", "alice").expect("should extract valid record");

        assert_eq!(record.author, "alice");
        assert_eq!(record.timestamp, "2026-09-28T20:00:00Z");
        assert_eq!(record.harness, "claude");
        assert_eq!(record.session_id, "sess-abc");
        assert_eq!(record.cwd_basename, "my-repo");
        assert_eq!(record.model, "claude-3-5-sonnet-20241022");
        assert_eq!(record.input_tokens, 120);
        assert_eq!(record.output_tokens, 45);
        assert_eq!(record.cache_read, 30);
        assert_eq!(record.cache_write, 15);
        assert_eq!(record.reasoning_tokens, 0);
    }

    #[test]
    fn test_extract_zero_usage_returns_none() {
        let line = r#"{"timestamp":"2026-09-28T20:00:00Z","sessionId":"sess-abc","message":{"model":"claude-3-5-sonnet-20241022","usage":{"input_tokens":0,"output_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}"#;
        assert!(extract_usage(line, "my-repo", "alice").is_none());
    }

    #[test]
    fn test_claude_adapter_availability() {
        let temp = tempfile::tempdir().unwrap();
        let adapter = ClaudeUsageAdapter;

        assert!(!adapter.is_available(temp.path()));

        std::fs::create_dir_all(temp.path().join(".claude/projects")).unwrap();
        assert!(adapter.is_available(temp.path()));
    }
}
