//! `ce-ai report-bug`: intelligent upstream bug detection, deduplication,
//! and reporting with user consent (Issue #426).

use std::io::{stdin, BufRead, IsTerminal};
use std::process::Command;

use crate::commands::Context;
use crate::error::CeError;
use crate::source::bug_reporter::{
    check_gh_status, format_github_issue_body, generate_web_issue_url, install_instructions,
    sanitize_text, search_upstream_issues, submit_issue_via_gh, BugReportBundle, GhStatus,
};

#[derive(clap::Args, Debug, Clone, Default)]
pub struct Args {
    /// Custom issue title
    #[arg(long)]
    pub title: Option<String>,

    /// Specific error message or stack trace to report
    #[arg(long)]
    pub error: Option<String>,

    /// Target AI coding harness affected (e.g. opencode, claude, cursor)
    #[arg(long, default_value = "All Harnesses")]
    pub harness: String,

    /// Preview sanitized report without submitting or opening browser
    #[arg(long)]
    pub dry_run: bool,

    /// Output diagnostic bundle as machine-readable JSON
    #[arg(long)]
    pub json: bool,

    /// Open prefilled issue directly in the default web browser
    #[arg(long)]
    pub web: bool,

    /// Submit directly via gh without interactive confirmation if gh is ready
    #[arg(short = 'y', long)]
    pub yes: bool,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let home = crate::harness::home_dir_from_ctx(ctx);
    let repo_root = ctx.repo_root();
    let project_root = Some(repo_root.as_path());

    let raw_error = args
        .error
        .clone()
        .unwrap_or_else(|| "Unexpected internal ce-ai error".to_string());

    let sanitized_error = sanitize_text(&raw_error, Some(&home), project_root);
    let sanitized_logs = sanitize_text(&raw_error, Some(&home), project_root);
    let raw_cmd = std::env::args().collect::<Vec<_>>().join(" ");
    let sanitized_command = sanitize_text(&raw_cmd, Some(&home), project_root);

    let bundle = BugReportBundle {
        ce_version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        target_harness: args.harness.clone(),
        command_invoked: sanitized_command,
        error_message: sanitized_error.clone(),
        logs_sanitized: sanitized_logs,
    };

    let raw_title = args
        .title
        .clone()
        .unwrap_or_else(|| format!("[BUG]: {sanitized_error}"));
    let title = sanitize_text(&raw_title, Some(&home), project_root);
    let body = format_github_issue_body(&bundle);

    // 1. JSON output mode
    if args.json {
        let json = serde_json::to_string_pretty(&bundle)
            .map_err(|e| CeError::Runtime(format!("Failed to serialize bundle: {e}")))?;
        println!("{json}");
        return Ok(());
    }

    // 2. Dry-run mode
    if args.dry_run {
        println!("== [Sanitized Bug Report Draft (Dry Run)] ==\n");
        println!("Title: {title}\n");
        println!("{body}");
        return Ok(());
    }

    // 3. Deduplication search against mastepanoski/ce-ai
    let matches = search_upstream_issues(&sanitized_error);
    if !matches.is_empty() {
        println!(
            "Found {} existing issue(s) matching this error signature:",
            matches.len()
        );
        for m in &matches {
            println!("  • #{} [{}] {} -> {}", m.number, m.state, m.title, m.url);
        }
        println!("\nPlease review the existing issue before filing a duplicate.\n");
    }

    let web_url = generate_web_issue_url(&title, &body);

    // 4. Direct web browser opening mode
    if args.web {
        open_web_url(&web_url)?;
        return Ok(());
    }

    let gh_status = check_gh_status();

    // 5. Non-interactive or auto-yes submission
    if args.yes {
        if gh_status == GhStatus::Ready {
            let issue_url = submit_issue_via_gh(&title, &body)?;
            println!("Issue created successfully: {issue_url}");
            return Ok(());
        } else {
            println!("GitHub CLI (gh) is not authenticated or not installed.");
            println!(
                "Installation guidance: {}",
                install_instructions(std::env::consts::OS)
            );
            println!("Direct web submission URL:\n  {web_url}");
            return Ok(());
        }
    }

    // 6. Interactive consent prompt if stdin is a terminal
    if stdin().is_terminal() {
        println!("⚠️  ce-ai Internal Bug Reporting Gate");
        println!("Title: {title}");
        println!("\nOptions:");
        println!("  [1] Submit issue directly to GitHub (via gh CLI)");
        println!("  [2] Open pre-filled submission draft in browser");
        println!("  [3] Preview sanitized markdown draft");
        println!("  [4] Dismiss / Cancel");
        print!("\nSelect an option [1-4] (default: 4): ");
        use std::io::Write;
        let _ = std::io::stdout().flush();

        let mut input = String::new();
        let _ = stdin().lock().read_line(&mut input);
        let choice = input.trim();

        match choice {
            "1" => {
                if gh_status == GhStatus::Ready {
                    let issue_url = submit_issue_via_gh(&title, &body)?;
                    println!("Issue created successfully: {issue_url}");
                } else {
                    println!("\nGitHub CLI (gh) is not ready.");
                    println!("Guidance: {}", install_instructions(std::env::consts::OS));
                    println!("Opening web browser fallback...\n");
                    open_web_url(&web_url)?;
                }
            }
            "2" => {
                open_web_url(&web_url)?;
            }
            "3" => {
                println!("\n---\n{body}\n---");
                println!("Web URL:\n  {web_url}\n");
            }
            _ => {
                println!("Bug report dismissed. No data was transmitted.");
            }
        }
    } else {
        // Non-interactive fallback: emit the sanitized report and web URL to stderr/stdout
        println!("ce-ai internal bug detected. Web submission URL:\n  {web_url}");
    }

    Ok(())
}

fn open_web_url(url: &str) -> Result<(), CeError> {
    println!("Web submission link:\n  {url}\n");

    #[cfg(target_os = "macos")]
    let res = Command::new("open").arg(url).spawn();

    #[cfg(target_os = "linux")]
    let res = Command::new("xdg-open").arg(url).spawn();

    #[cfg(target_os = "windows")]
    let res = Command::new("cmd").args(["/c", "start", url]).spawn();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    let res: std::io::Result<std::process::Child> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "unsupported platform",
    ));

    match res {
        Ok(_) => Ok(()),
        Err(_) => {
            println!("Could not launch browser automatically. Please open the URL above manually.");
            Ok(())
        }
    }
}
