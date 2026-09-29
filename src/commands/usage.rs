//! `ce-ai usage`: capture, report, hours.

use chrono::{DateTime, Utc};

use crate::capture::ledger::UsageRecord;
use crate::commands::Context;
use crate::error::CeError;

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub command: UsageCommand,
}

#[derive(clap::Subcommand)]
pub enum UsageCommand {
    /// Capture usage from local Claude Code transcripts into the ledger.
    Sync,
    /// Render ledger entries with optional inclusive RFC 3339 date filters.
    Report {
        /// Include records at or after this RFC 3339 timestamp.
        #[arg(long)]
        from: Option<String>,
        /// Include records at or before this RFC 3339 timestamp.
        #[arg(long)]
        to: Option<String>,
        /// Output grouping; only `record` is currently supported.
        #[arg(long)]
        by: Option<String>,
        /// Render matching records as JSON.
        #[arg(long)]
        json: bool,
    },
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let cmd = &args.command;
    match cmd {
        UsageCommand::Sync => sync(ctx),
        UsageCommand::Report { from, to, by, json } => {
            report(ctx, from.as_deref(), to.as_deref(), by.as_deref(), *json)
        }
    }
}

fn sync(ctx: &Context) -> Result<(), CeError> {
    let author = git_user()?;
    let home = std::env::var("HOME").unwrap_or_default();
    let home_path = std::path::Path::new(&home);

    let mut total_records = 0;
    for adapter in crate::harness::usage::all_adapters() {
        if !adapter.is_available(home_path) {
            continue;
        }
        let records = adapter.read_usage(home_path, &author, None, None)?;
        if !records.is_empty() {
            let count = crate::capture::ledger::append_records(&ctx.config_dir, &author, &records)?;
            total_records += count;
        }
    }

    if total_records == 0 {
        if !ctx.quiet {
            println!("usage: no new records");
        }
        return Ok(());
    }

    if !ctx.quiet {
        println!("usage: captured {total_records} record(s) for {author}");
    }
    Ok(())
}

fn report(
    ctx: &Context,
    from: Option<&str>,
    to: Option<&str>,
    by: Option<&str>,
    json: bool,
) -> Result<(), CeError> {
    validate_group_by(by)?;
    let shard_dir = crate::capture::ledger::shard_dir(&ctx.config_dir);
    if !shard_dir.exists() {
        println!("no usage data");
        return Ok(());
    }
    let mut all = Vec::new();
    for entry in std::fs::read_dir(&shard_dir)?.flatten() {
        let p = entry.path();
        if p.extension().map(|e| e == "jsonl").unwrap_or(false) {
            all.extend(crate::capture::ledger::read_shard(&p)?);
        }
    }
    let all = filter_records(all, from, to)?;
    if all.is_empty() {
        println!("no usage data");
        return Ok(());
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&all)?);
        return Ok(());
    }

    for r in &all {
        println!(
            "{} {} {} in={} out={} cache_r={} cache_w={}",
            r.timestamp,
            r.harness,
            r.model,
            r.input_tokens,
            r.output_tokens,
            r.cache_read,
            r.cache_write
        );
    }
    Ok(())
}

/// Validates the report rendering mode. Aggregated groupings are not yet
/// implemented, so accepting them would make the CLI promise a no-op.
fn validate_group_by(by: Option<&str>) -> Result<(), CeError> {
    match by.map(str::trim).filter(|value| !value.is_empty()) {
        None | Some("record") => Ok(()),
        Some(value) => Err(CeError::Usage(format!(
            "unsupported usage report grouping '{value}'. Only 'record' is currently supported"
        ))),
    }
}

/// Filters ledger records by an inclusive RFC 3339 interval.
fn filter_records(
    records: Vec<UsageRecord>,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Vec<UsageRecord>, CeError> {
    let from = parse_bound(from, "--from")?;
    let to = parse_bound(to, "--to")?;

    if let (Some(from), Some(to)) = (from, to) {
        if from > to {
            return Err(CeError::Usage(
                "--from must be earlier than or equal to --to".into(),
            ));
        }
    }

    let filtered = records
        .into_iter()
        .filter_map(|record| {
            let timestamp = match DateTime::parse_from_rfc3339(&record.timestamp) {
                Ok(value) => value.with_timezone(&Utc),
                Err(err) => {
                    eprintln!(
                        "warning: skipping usage record with invalid timestamp '{}': {err}",
                        record.timestamp
                    );
                    return None;
                }
            };
            if from.is_some_and(|bound| timestamp < bound)
                || to.is_some_and(|bound| timestamp > bound)
            {
                None
            } else {
                Some(record)
            }
        })
        .collect();
    Ok(filtered)
}

fn parse_bound(raw: Option<&str>, flag: &str) -> Result<Option<DateTime<Utc>>, CeError> {
    raw.map(|value| {
        DateTime::parse_from_rfc3339(value)
            .map(|parsed| parsed.with_timezone(&Utc))
            .map_err(|err| {
                CeError::Usage(format!(
                    "invalid {flag} RFC 3339 timestamp '{value}': {err}"
                ))
            })
    })
    .transpose()
}

fn git_user() -> Result<String, CeError> {
    let out = std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()?;
    if out.status.success() {
        let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !name.is_empty() {
            return Ok(name);
        }
    }
    Ok(std::env::var("USER").unwrap_or_else(|_| "unknown".into()))
}

#[cfg(test)]
#[path = "tests/usage.rs"]
mod tests;
