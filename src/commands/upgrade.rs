//! `ce-ai upgrade`: resolve a newer CE source (SU-5) then sync. The default
//! path fetches the latest GitHub release; `--to <tag>` resolves from the local
//! cache; `--source <path>` uses a local tree. Network paths are exercised by
//! the Phase 7 E2E gate — integration tests use cache/local resolution only.

use std::path::{Path, PathBuf};

use crate::commands::{sync, Context};
use crate::error::CeError;
use crate::source::archive::extract_safe;
use crate::source::cache::Cache;
use crate::source::release::{
    github_token_from_env, main_tarball_url, resolve_latest_release, tag_tarball_url,
};
use crate::state::state::State;

#[derive(clap::Args)]
pub struct Args {
    /// Target release tag; resolves from the local cache instead of GitHub.
    #[arg(long)]
    pub to: Option<String>,
    /// Local CE source tree; bypasses release fetching and the cache.
    #[arg(long)]
    pub source: Option<PathBuf>,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    if let Some(path) = &args.source {
        let version = "local".to_string();
        let source_json = serde_json::json!({ "kind": "local", "path": path });
        return sync::sync_with(ctx, path, &version, source_json);
    }
    if let Some(tag) = &args.to {
        let tarball = cached_tarball(ctx)?;
        return sync_from_extracted(ctx, &tarball, tag, tag);
    }
    // Default: fetch the latest release from GitHub, cache it, then sync (SU-5).
    let client = reqwest::blocking::Client::new();
    let token = github_token_from_env();
    let tag = resolve_latest_release(&client, token.as_deref())?;
    let (version, url) = match tag {
        Some(tag) => (tag.clone(), tag_tarball_url(&tag)),
        None => ("main".to_string(), main_tarball_url()),
    };
    let bytes = client
        .get(&url)
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
        .map_err(|err| CeError::Runtime(format!("release download failed: {err}")))?
        .bytes()
        .map_err(|err| CeError::Runtime(err.to_string()))?;
    let tarball = Cache::new(ctx.config_dir.join("cache"))
        .cache_tarball(&bytes, &ctx.config_dir.join("state.json"))?;
    sync_from_extracted(ctx, &tarball, &version, &version)
}

/// Locates the cached tarball recorded as `managed_asset_digest.tarball`
/// (SF-3); `upgrade --to` uses it to stay offline in tests and hermetic runs.
fn cached_tarball(ctx: &Context) -> Result<PathBuf, CeError> {
    let state = State::load(&ctx.config_dir.join("state.json"))?;
    let hex = state
        .managed_asset_digest
        .get("tarball")
        .and_then(|digest| digest.strip_prefix("sha256:"))
        .ok_or_else(|| {
            CeError::Runtime("no cached tarball digest in state.json — run upgrade without --to to fetch from GitHub".into())
        })?;
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
    Ok(tarball)
}

/// Extracts a tarball, locates the source root, runs sync, and cleans up the
/// dry-run temp tree so the dry-run writes nothing on the managed surface.
fn sync_from_extracted(
    ctx: &Context,
    tarball: &Path,
    tag: &str,
    version: &str,
) -> Result<(), CeError> {
    let (root, tmp) = extract_to_source(ctx, tarball, tag)?;
    let source_json = serde_json::json!({ "kind": "github-release", "tag": version, "tree": root });
    let result = sync::sync_with(ctx, &root, version, source_json);
    if let Some(tmp) = tmp {
        let _ = std::fs::remove_dir_all(tmp);
    }
    result
}

/// Real runs persist the extracted tree under `<config-dir>/cache/trees/<tag>`
/// so later `sync` runs resolve it from the manifest; dry-runs extract to a
/// system temp dir that is removed afterwards.
fn extract_to_source(
    ctx: &Context,
    tarball: &Path,
    tag: &str,
) -> Result<(PathBuf, Option<PathBuf>), CeError> {
    let safe_tag: String = tag
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect();
    let dest = if ctx.dry_run {
        std::env::temp_dir().join(format!("ce-ai-upgrade-{}", std::process::id()))
    } else {
        ctx.config_dir.join("cache/trees").join(&safe_tag)
    };
    let _ = std::fs::remove_dir_all(&dest);
    extract_safe(tarball, &dest)?;
    let root = find_source_root(&dest)?;
    let tmp = if ctx.dry_run { Some(dest) } else { None };
    Ok((root, tmp))
}

/// GitHub tarballs nest the tree under a `<repo>-<ref>/` top dir; the source
/// root is the extracted dir itself when it holds `.opencode`, else its single
/// subdirectory.
fn find_source_root(dir: &Path) -> Result<PathBuf, CeError> {
    if dir.join(".opencode").is_dir() {
        return Ok(dir.to_path_buf());
    }
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    if dirs.len() == 1 {
        return Ok(dirs.remove(0));
    }
    Err(CeError::Runtime(format!(
        "cannot locate .opencode source tree under {}",
        dir.display()
    )))
}
