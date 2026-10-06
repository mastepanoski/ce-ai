//! `ce-ai upgrade`: resolve a newer CE source (SU-5) then sync. The default
//! path fetches the latest GitHub release; `--to <tag>` resolves from the
//! local cache only when its recorded provenance matches the requested tag;
//! `--source <path>` uses a local tree. Network paths are exercised by the
//! Phase 7 E2E gate — integration tests use cache/local resolution only.

use std::path::{Path, PathBuf};

use crate::commands::{sync, Context};
use crate::error::CeError;
use crate::source::archive::extract_to_source;
use crate::source::release::{resolve_github_token, resolve_latest_release};
use crate::state::diff::sha256_hex;
use crate::state::state::State;

#[derive(clap::Args)]
pub struct Args {
    /// Target release tag; resolves from the local cache only when it matches
    /// the recorded release provenance and passes integrity verification.
    #[arg(long)]
    pub to: Option<String>,
    /// Local CE source tree; bypasses release fetching and the cache.
    #[arg(long)]
    pub source: Option<PathBuf>,
    /// Also update the ce-ai CLI binary itself to the latest release.
    #[arg(long)]
    pub bin: bool,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    eprintln!(
        "[DEPRECATION] 'ce-ai upgrade' is deprecated in CE-AI v2.0.\n\
         In v2, harnesses are updated through native plugin managers.\n\
         Use 'ce-ai fleet pin <version>' followed by 'ce-ai fleet sync' to govern version upgrades across harnesses."
    );
    if args.bin {
        crate::commands::self_update::run(
            ctx,
            &crate::commands::self_update::Args {
                check: false,
                to: None,
                force: false,
            },
        )?;
        println!();
    }

    let state_path = ctx.config_dir.join("state.json");
    let is_local_installed = if state_path.exists() {
        State::load(&state_path)
            .map(|state| {
                state.installed_harnesses.iter().any(|h| {
                    h.get("source")
                        .and_then(|s| s.get("kind"))
                        .and_then(|k| k.as_str())
                        == Some("local")
                })
            })
            .unwrap_or(false)
    } else {
        false
    };
    if is_local_installed && args.source.is_none() && args.to.is_none() {
        println!("notice: upgrading harnesses with local source to latest GitHub release.");
    }

    if let Some(path) = &args.source {
        let version = "local".to_string();
        let source_json = serde_json::json!({ "kind": "local", "path": path });
        return sync::sync_with(ctx, path, &version, source_json);
    }
    if let Some(tag) = &args.to {
        let tarball = cached_tarball_for(ctx, tag)?;
        return sync_from_extracted(ctx, &tarball, tag, tag);
    }

    // In v2, bare upgrade pins the latest release for the fleet and syncs native harnesses.
    let client = reqwest::blocking::Client::new();
    let token = resolve_github_token();
    let tag = resolve_latest_release(&client, token.as_deref())?.ok_or_else(|| {
        CeError::Usage(
            "no 'compound-engineering-v*' release found on GitHub — pin a specific version with 'ce-ai fleet pin <version>'".to_string(),
        )
    })?;
    crate::commands::fleet::handle_pin(ctx, &tag)?;
    crate::commands::fleet::handle_sync(ctx, ctx.dry_run)?;
    Ok(())
}

/// Resolves `--to <tag>` strictly against the recorded release provenance
/// (Issue #161): the cached artifact is used only when its tag matches and
/// its on-disk bytes still hash to the recorded digest — never relabelled.
fn cached_tarball_for(ctx: &Context, requested_tag: &str) -> Result<PathBuf, CeError> {
    let state = State::load(&ctx.config_dir.join("state.json"))?;
    let prov = state.release_provenance.as_ref().ok_or_else(|| {
        CeError::Usage(format!(
            "no release provenance in state.json for '{requested_tag}' — run 'ce-ai upgrade' without --to to fetch a release first"
        ))
    })?;
    if prov.tag != requested_tag {
        return Err(CeError::Usage(format!(
            "cached release is '{}' but '--to {}' was requested; ce-ai never relabels artifacts — run 'ce-ai upgrade' without --to to fetch '{}', or use '--to {}'",
            prov.tag, requested_tag, requested_tag, prov.tag
        )));
    }
    let hex = &prov.archive_sha256;
    match state.managed_asset_digest.get("tarball") {
        Some(digest) if digest == &format!("sha256:{hex}") => {}
        _ => {
            return Err(CeError::Verification(format!(
                "state.json digest/provenance mismatch for tarball '{hex}' — re-run 'ce-ai upgrade' without --to to refresh provenance"
            )));
        }
    }
    let tarball = ctx
        .config_dir
        .join("cache")
        .join(format!("ce-{hex}.tar.gz"));
    if !tarball.exists() {
        return Err(CeError::Runtime(format!(
            "cached tarball not found at {} — run upgrade without --to to fetch from GitHub",
            tarball.display()
        )));
    }
    let actual = sha256_hex(&std::fs::read(&tarball)?);
    if actual != *hex {
        return Err(CeError::Verification(format!(
            "cached archive integrity check failed for '{}': expected sha256:{hex}, got sha256:{actual} — delete {} and re-run 'ce-ai upgrade' to re-fetch",
            requested_tag,
            tarball.display()
        )));
    }
    Ok(tarball)
}

/// Extracts a tarball, locates the source root, runs sync, and cleans up the dry-run
/// temp tree so the dry-run writes nothing on the managed surface.
fn sync_from_extracted(
    ctx: &Context,
    tarball: &Path,
    tag: &str,
    version: &str,
) -> Result<(), CeError> {
    let (root, tmp) = extract_to_source(&ctx.config_dir, ctx.dry_run, tarball, tag)?;
    let source_json = serde_json::json!({ "kind": "github-release", "tag": version, "tree": root });
    let result = sync::sync_with(ctx, &root, version, source_json);
    if let Some(tmp) = tmp {
        crate::state::report_best_effort_remove(&tmp, std::fs::remove_dir_all(&tmp));
    }
    result
}

#[cfg(test)]
#[path = "tests/upgrade.rs"]
mod tests;
