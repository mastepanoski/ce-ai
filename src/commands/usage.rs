//! `ce-ai usage`: capture, report, hours.

use crate::commands::Context;
use crate::error::CeError;

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub command: UsageCommand,
}

#[derive(clap::Subcommand)]
pub enum UsageCommand {
    /// Capture usage from local harness sources into the ledger.
    Sync,
    /// Aggregate ledger entries with filters.
    Report {
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long)]
        by: Option<String>,
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
    let claude_projects = std::path::PathBuf::from(&home).join(".claude/projects");

    let records = crate::harness::usage::claude::read_usage(&claude_projects, None, &author, None)?;

    if records.is_empty() {
        if !ctx.quiet {
            println!("usage: no new records");
        }
        return Ok(());
    }

    let typed: Vec<crate::capture::ledger::UsageRecord> = records
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    let count = crate::capture::ledger::append_records(&ctx.config_dir, &author, &typed)?;
    if !ctx.quiet {
        println!("usage: captured {count} record(s) for {author}");
    }
    Ok(())
}

fn report(
    ctx: &Context,
    _from: Option<&str>,
    _to: Option<&str>,
    _by: Option<&str>,
    json: bool,
) -> Result<(), CeError> {
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
