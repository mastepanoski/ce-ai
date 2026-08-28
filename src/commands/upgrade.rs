//! `ce-ai upgrade`: resolve a newer CE source (SU-5) then sync. The default
//! path fetches the latest GitHub release; `--to <tag>` resolves from the
//! local cache only when its recorded provenance matches the requested tag;
//! `--source <path>` uses a local tree. Network paths are exercised by the
//! Phase 7 E2E gate — integration tests use cache/local resolution only.

use std::path::{Path, PathBuf};

use crate::commands::{sync, Context};
use crate::error::CeError;
use crate::source::archive::extract_to_source;
use crate::source::cache::{record_tarball_provenance, Cache};
use crate::source::release::{
    github_token_from_env, pinned_version_and_url, resolve_latest_release,
};
use crate::state::diff::sha256_hex;
use crate::state::state::{ReleaseProvenance, State};

#[derive(clap::Args)]
pub struct Args {
    /// Target release tag; resolves from the local cache only when it matches
    /// the recorded release provenance and passes integrity verification.
    #[arg(long)]
    pub to: Option<String>,
    /// Local CE source tree; bypasses release fetching and the cache.
    #[arg(long)]
    pub source: Option<PathBuf>,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load(&state_path)?;

    // MH-2: Upgrade converts local source installations to latest GitHub release.
    let is_local_installed = state.installed_harnesses.iter().any(|h| {
        h.get("source")
            .and_then(|s| s.get("kind"))
            .and_then(|k| k.as_str())
            == Some("local")
    });
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
        return sync_from_extracted(ctx, &tarball, tag, tag, None);
    }
    // Default: fetch the latest release from GitHub, cache it with full
    // provenance, then sync (SU-5). No implicit fallback: every failure is an
    // explicit error and the resolved tag is immutable.
    let client = reqwest::blocking::Client::new();
    let token = github_token_from_env();
    let tag = resolve_latest_release(&client, token.as_deref())?;
    let (version, url) = pinned_version_and_url(tag)?;
    let bytes = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
        .map_err(|err| CeError::Network(format!("release download failed: {err}")))?
        .bytes()
        .map_err(|err| CeError::Runtime(err.to_string()))?;
    let (tarball, hex, _dry_run_tmp) = if ctx.dry_run {
        let tmp = tempfile::TempDir::new()?;
        let tarball_path = tmp.path().join("dry_run.tar.gz");
        std::fs::write(&tarball_path, &bytes)?;
        use sha2::Digest;
        let hex = format!("{:x}", sha2::Sha256::digest(&bytes));
        (tarball_path, hex, Some(tmp))
    } else {
        let (tarball, hex) = Cache::new(ctx.config_dir.join("cache")).cache_tarball(&bytes)?;
        (tarball, hex, None)
    };
    sync_from_extracted(
        ctx,
        &tarball,
        &version,
        &version,
        if ctx.dry_run {
            None
        } else {
            Some(FetchMeta { url, sha256: hex })
        },
    )
}
/// Download metadata needed to record release provenance after extraction.
struct FetchMeta {
    url: String,
    sha256: String,
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

/// Extracts a tarball, records fresh provenance when this run fetched a new
/// archive, locates the source root, runs sync, and cleans up the dry-run
/// temp tree so the dry-run writes nothing on the managed surface.
fn sync_from_extracted(
    ctx: &Context,
    tarball: &Path,
    tag: &str,
    version: &str,
    fetch: Option<FetchMeta>,
) -> Result<(), CeError> {
    let (root, tmp) = extract_to_source(&ctx.config_dir, ctx.dry_run, tarball, tag)?;
    if let (Some(meta), false) = (fetch, ctx.dry_run) {
        record_tarball_provenance(
            &ctx.config_dir.join("state.json"),
            ReleaseProvenance {
                tag: version.to_string(),
                url: meta.url,
                archive_sha256: meta.sha256,
                extraction_path: root.clone(),
            },
        )?;
    }
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
