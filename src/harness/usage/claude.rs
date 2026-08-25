//! Incremental Claude Code JSONL usage reader.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Extracts (timestamp, model, input, cache_read, cache_write, output) from
/// a JSONL transcript line that carries usage fields.
fn extract_usage(line: &str, cwd_basename: &str, author: &str) -> Option<serde_json::Value> {
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

    Some(serde_json::json!({
        "author": author,
        "timestamp": ts,
        "harness": "claude",
        "session_id": sid,
        "cwd_basename": cwd_basename,
        "model": model,
        "input_tokens": inp,
        "output_tokens": out,
        "cache_read": cr,
        "cache_write": cw,
        "reasoning_tokens": 0
    }))
}

/// Reads all JSONL files under `projects_dir` newer than `since` (ISO timestamp string).
/// Returns extracted records as JSON values.
pub fn read_usage(
    projects_dir: &PathBuf,
    since: Option<&str>,
    author: &str,
    cwd_filter: Option<&str>,
) -> Result<Vec<serde_json::Value>, crate::error::CeError> {
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

/// Shallow recursive JSONL collector.
fn walkdir(dir: &PathBuf) -> Vec<PathBuf> {
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
