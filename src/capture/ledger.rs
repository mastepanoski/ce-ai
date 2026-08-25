//! Shard-per-author usage ledger (`.ce-ai/usage/<dev>.jsonl`).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CeError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub author: String,
    pub timestamp: String,
    pub harness: String,
    pub session_id: String,
    pub cwd_basename: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub reasoning_tokens: u64,
}

/// Dedup key: harness + session_id + timestamp.
fn dedup_key(r: &UsageRecord) -> String {
    format!("{}|{}|{}", r.harness, r.session_id, r.timestamp)
}

pub fn shard_dir(config_dir: &Path) -> PathBuf {
    config_dir.join("usage")
}

pub fn shard_path(config_dir: &Path, author: &str) -> PathBuf {
    let slug: String = author
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    shard_dir(config_dir).join(format!("{slug}.jsonl"))
}

pub fn read_shard(path: &Path) -> Result<Vec<UsageRecord>, CeError> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let text = std::fs::read_to_string(path)?;
    let mut out = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<UsageRecord>(line) {
            Ok(r) => out.push(r),
            Err(e) => eprintln!(
                "warning: skipping malformed ledger line in {}: {e}",
                path.display()
            ),
        }
    }
    Ok(out)
}

pub fn append_records(
    config_dir: &Path,
    author: &str,
    new_records: &[UsageRecord],
) -> Result<usize, CeError> {
    let existing = read_shard(&shard_path(config_dir, author))?;
    let seen: std::collections::HashSet<String> = existing.iter().map(dedup_key).collect();
    let path = shard_path(config_dir, author);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut appended = 0usize;
    for rec in new_records {
        let key = dedup_key(rec);
        if seen.contains(&key) {
            continue;
        }
        let line = format!("{}\n", serde_json::to_string(rec)?);
        append_line_atomic(&path, line.as_bytes())?;
        appended += 1;
    }
    Ok(appended)
}

fn append_line_atomic(path: &Path, bytes: &[u8]) -> Result<(), CeError> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(bytes)?;
    Ok(())
}
