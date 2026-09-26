//! `ce-ai self-update`: securely downloads, verifies, and atomically replaces
//! the running CLI binary itself (Issue #341).

use std::path::PathBuf;

use crate::commands::Context;
use crate::error::CeError;
use crate::source::binary_release::{
    asset_name_for_target, compare_cli_versions, download_release_asset_and_sums, extract_binary,
    replace_executable, resolve_cli_release_by_tag, resolve_latest_cli_release, verify_checksum,
};

#[derive(clap::Args, Debug, Clone)]
pub struct Args {
    /// Check for available CLI binary updates without applying them.
    #[arg(long)]
    pub check: bool,

    /// Target a specific release tag (e.g. v1.50.0) instead of latest.
    #[arg(long)]
    pub to: Option<String>,

    /// Force replacement even if current version is up to date or newer.
    #[arg(long)]
    pub force: bool,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let target = crate::source::binary_release::current_target();
    let current_exe = std::env::current_exe().ok();
    run_internal(ctx, args, target, current_exe, None)
}

/// Internal execution entry point supporting dependency injection for testing.
pub fn run_internal(
    ctx: &Context,
    args: &Args,
    target: Option<&str>,
    target_exe: Option<PathBuf>,
    client_opt: Option<reqwest::blocking::Client>,
) -> Result<(), CeError> {
    let target = target.ok_or_else(|| {
        CeError::Usage(
            "Unsupported host architecture or operating system for self-update. Please install manually or build from source."
                .to_string(),
        )
    })?;

    let target_exe = match target_exe {
        Some(p) => p,
        None => std::env::current_exe().map_err(CeError::Io)?,
    };
    let target_exe = target_exe.canonicalize().unwrap_or(target_exe);

    // Warn if running in a package-manager directory (e.g. Homebrew Cellar)
    let exe_str = target_exe.to_string_lossy();
    if (exe_str.contains("/Cellar/")
        || exe_str.contains("/homebrew/")
        || exe_str.contains("\\homebrew\\"))
        && !ctx.quiet
    {
        println!(
            "notice: ce-ai binary appears to be located in a package manager directory ({}). Consider using your package manager (e.g. 'brew upgrade ce-ai') instead.",
            target_exe.display()
        );
    }

    let client = client_opt.unwrap_or_default();
    let token = crate::source::release::resolve_github_token();

    if !ctx.quiet {
        println!("Checking for ce-ai CLI updates...");
    }

    let release = match &args.to {
        Some(tag) => resolve_cli_release_by_tag(&client, token.as_deref(), tag)?,
        None => resolve_latest_cli_release(&client, token.as_deref())?,
    };

    // Refresh update notifier cache on explicit checks or upgrades (#425)
    let update_cache = crate::source::update_notifier::UpdateCheckCache {
        last_checked_at: chrono::Utc::now().to_rfc3339(),
        latest_version: release.version.clone(),
        latest_tag: release.tag.clone(),
        release_url: format!(
            "https://github.com/mastepanoski/ce-ai/releases/tag/{}",
            release.tag
        ),
    };
    let _ = crate::source::update_notifier::write_cache(&ctx.config_dir, &update_cache);

    let current_version = env!("CARGO_PKG_VERSION");
    let cmp = compare_cli_versions(&release.version, current_version);
    let is_newer = cmp == std::cmp::Ordering::Greater;

    if args.check {
        if is_newer {
            println!(
                "A newer release of ce-ai is available: v{current_version} -> {} (target: {target}).\nRun 'ce-ai self-update' to upgrade.",
                release.tag
            );
        } else {
            println!("ce-ai is up to date (v{current_version} is the latest version).");
        }
        return Ok(());
    }

    if !is_newer && !args.force {
        println!(
            "ce-ai is already up to date (v{current_version} is the latest version). Use --force to reinstall."
        );
        return Ok(());
    }

    if !ctx.quiet {
        println!(
            "Upgrading ce-ai: v{current_version} -> {} (target: {target})...",
            release.tag
        );
    }

    let asset_name = asset_name_for_target(target);
    if ctx.dry_run {
        println!(
            "[dry-run] Would download release asset {asset_name}, verify SHA256 integrity, and replace {}",
            target_exe.display()
        );
        return Ok(());
    }

    if !ctx.quiet {
        println!("Downloading {asset_name} and verifying SHA256 integrity...");
    }
    let (archive_bytes, expected_sha256) =
        download_release_asset_and_sums(&client, &release, target)?;

    verify_checksum(&archive_bytes, &expected_sha256)?;

    if !ctx.quiet {
        println!(
            "Extracting binary and replacing executable at {}...",
            target_exe.display()
        );
    }
    let is_zip = target.contains("windows");
    let binary_bytes = extract_binary(&archive_bytes, is_zip)?;

    replace_executable(&target_exe, &binary_bytes)?;

    println!(
        "Successfully updated ce-ai from v{current_version} to {}!",
        release.tag
    );
    println!("Run 'ce-ai doctor' to verify health.");

    Ok(())
}

#[cfg(test)]
#[path = "tests/self_update.rs"]
mod tests;
