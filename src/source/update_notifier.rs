//! Automated background update notifier for ce-ai CLI & harness releases (Issue #425).
//!
//! Provides non-blocking, throttled update checks against GitHub Releases API,
//! atomic cache persistence at `<config_dir>/cache/update_check.json`, and
//! non-intrusive terminal notifications rendered strictly to `stderr` upon command completion.

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::commands::Context;
use crate::error::CeError;
use crate::state::state::State;

/// Cached update metadata stored at `<config_dir>/cache/update_check.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateCheckCache {
    pub last_checked_at: String,
    pub latest_version: String,
    pub latest_tag: String,
    pub release_url: String,
}

/// Returns the path to the cached update metadata file.
pub fn update_cache_path(config_dir: &Path) -> PathBuf {
    config_dir.join("cache").join("update_check.json")
}

/// Reads and parses the cached update check metadata if present and valid.
pub fn read_cache(config_dir: &Path) -> Option<UpdateCheckCache> {
    let path = update_cache_path(config_dir);
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Writes update check metadata atomically to `<config_dir>/cache/update_check.json`.
pub fn write_cache(config_dir: &Path, cache: &UpdateCheckCache) -> Result<(), CeError> {
    let path = update_cache_path(config_dir);
    let bytes = serde_json::to_vec_pretty(cache)?;
    crate::state::write_atomic(&path, &bytes)
}

/// Determines if the cached update metadata is older than `interval_hours`.
pub fn is_cache_stale(cache: &UpdateCheckCache, interval_hours: u64) -> bool {
    let parsed = match chrono::DateTime::parse_from_rfc3339(&cache.last_checked_at) {
        Ok(dt) => dt.with_timezone(&chrono::Utc),
        Err(_) => return true,
    };
    let now = chrono::Utc::now();
    let age = match now.signed_duration_since(parsed).to_std() {
        Ok(duration) => duration,
        Err(_) => return true,
    };
    age >= Duration::from_secs(interval_hours.saturating_mul(3600))
}

/// Checks whether update checking and notifications are suppressed.
///
/// Hierarchy (any true suppresses):
/// 1. Global CLI flag `--quiet` / `-q`
/// 2. `CE_NO_UPDATE_NOTIFIER=1`
/// 3. CI environments (`CI=true`, `GITHUB_ACTIONS=true`, `CONTINUOUS_INTEGRATION=true`)
/// 4. Non-interactive stderr (`!std::io::stderr().is_terminal()`)
/// 5. Config opt-out in `state.json` (`update_notifier.enabled == false`)
pub fn is_suppressed(ctx: &Context, state: &State) -> bool {
    let env_no_update = std::env::var("CE_NO_UPDATE_NOTIFIER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let force_notifier = std::env::var("CE_FORCE_UPDATE_NOTIFIER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let is_ci = !force_notifier
        && (std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
            || std::env::var("CONTINUOUS_INTEGRATION").is_ok());
    let is_terminal = force_notifier || std::io::stderr().is_terminal();

    is_suppressed_internal(
        ctx.quiet,
        env_no_update,
        is_ci,
        is_terminal,
        state.update_notifier().enabled,
    )
}

/// Pure internal evaluation of suppression conditions for testing.
pub fn is_suppressed_internal(
    quiet: bool,
    env_suppressed: bool,
    is_ci: bool,
    is_terminal: bool,
    config_enabled: bool,
) -> bool {
    if quiet {
        return true;
    }
    if env_suppressed {
        return true;
    }
    if is_ci {
        return true;
    }
    if !is_terminal {
        return true;
    }
    if !config_enabled {
        return true;
    }
    false
}

/// Determines if `latest_version` represents a newer release than `current_version`.
pub fn is_newer_version(latest_version: &str, current_version: &str) -> bool {
    crate::source::binary_release::compare_cli_versions(latest_version, current_version)
        == std::cmp::Ordering::Greater
}

/// Synchronously checks for the latest release and writes cache atomically.
pub fn check_and_update_cache_sync(
    config_dir: &Path,
    client: &reqwest::blocking::Client,
    token: Option<&str>,
) -> Result<UpdateCheckCache, CeError> {
    let release = crate::source::binary_release::resolve_latest_cli_release(client, token)?;
    let cache = UpdateCheckCache {
        last_checked_at: chrono::Utc::now().to_rfc3339(),
        latest_version: release.version,
        latest_tag: release.tag.clone(),
        release_url: format!(
            "https://github.com/mastepanoski/ce-ai/releases/tag/{}",
            release.tag
        ),
    };
    let _ = write_cache(config_dir, &cache);
    Ok(cache)
}

/// Spawns a detached background thread to check for updates and update the cache.
/// Operates with a 3-second timeout and fails closed completely silently.
pub fn spawn_background_check(config_dir: PathBuf, token: Option<String>) {
    std::thread::spawn(move || {
        let client_res = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(3))
            .build();
        if let Ok(client) = client_res {
            let _ = check_and_update_cache_sync(&config_dir, &client, token.as_deref());
        }
    });
}

/// Triggers a background check if not suppressed and cache is missing or stale.
pub fn maybe_trigger_background_check(ctx: &Context, state: &State) {
    if is_suppressed(ctx, state) {
        return;
    }
    let interval = state.update_notifier().interval_hours;
    let needs_check = match read_cache(&ctx.config_dir) {
        Some(cache) => is_cache_stale(&cache, interval),
        None => true,
    };
    if needs_check {
        let token = crate::source::release::resolve_github_token();
        spawn_background_check(ctx.config_dir.clone(), token);
    }
}

/// Formats the notification box banner using Unicode line drawing.
pub fn format_update_banner(current_version: &str, latest_tag: &str) -> String {
    let clean_current = current_version.strip_prefix('v').unwrap_or(current_version);
    let target_display = if latest_tag.starts_with('v') {
        latest_tag.to_string()
    } else {
        format!("v{latest_tag}")
    };

    let line1 = format!("Update available: ce-ai v{clean_current} → {target_display}");
    let line2 = "Run 'ce-ai self-update' or 'brew upgrade ce-ai' to upgrade".to_string();

    let content_len = line1.len().max(line2.len());
    let inner_width = content_len + 4; // 2 leading + 2 trailing spaces

    let top = format!("╭{}╮", "─".repeat(inner_width));
    let bottom = format!("╰{}╯", "─".repeat(inner_width));

    let pad1 = " ".repeat(inner_width.saturating_sub(line1.len() + 2));
    let pad2 = " ".repeat(inner_width.saturating_sub(line2.len() + 2));

    let row1 = format!("│  {line1}{pad1}│");
    let row2 = format!("│  {line2}{pad2}│");

    format!("{top}\n{row1}\n{row2}\n{bottom}")
}

/// Renders the update notification banner to stderr if a newer version is cached and not suppressed.
pub fn maybe_print_update_notification(ctx: &Context, state: &State) {
    if is_suppressed(ctx, state) {
        return;
    }
    if let Some(cache) = read_cache(&ctx.config_dir) {
        let current_version = env!("CARGO_PKG_VERSION");
        if is_newer_version(&cache.latest_version, current_version) {
            let banner = format_update_banner(current_version, &cache.latest_tag);
            eprintln!("\n{banner}\n");
        }
    }
}

#[cfg(test)]
#[path = "tests/update_notifier.rs"]
mod tests;
