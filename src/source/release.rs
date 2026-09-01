//! GitHub releases resolution for the CE plugin (SF-1) with optional
//! `CE_AI_GITHUB_TOKEN` passthrough. Resolution never falls back to the
//! mutable `main` branch: every failure is an explicit error. Unit tests
//! never touch the network — parsing is exercised on fixture payloads.

use std::cmp::Ordering;

use serde_json::Value;

use crate::error::CeError;

/// GitHub owner/repo for the compound-engineering plugin.
pub const PLUGIN_REPO: &str = "everyinc/compound-engineering-plugin";

/// Releases API endpoint (SF-1).
pub fn releases_api_url() -> String {
    format!("https://api.github.com/repos/{PLUGIN_REPO}/releases")
}

/// Tarball URL for a specific release tag (upgrade).
pub fn tag_tarball_url(tag: &str) -> String {
    format!("https://github.com/{PLUGIN_REPO}/archive/refs/tags/{tag}.tar.gz")
}

/// Splits a version like `3.4.2` into comparable numeric components.
fn version_components(version: &str) -> Option<Vec<u64>> {
    version
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect()
}

/// Numeric (semver-style) comparison; falls back to lexical when either side
/// is not purely numeric. Makes `v3.10.0` sort after `v3.4.2`.
fn compare_versions(a: &str, b: &str) -> Ordering {
    match (version_components(a), version_components(b)) {
        (Some(va), Some(vb)) => {
            for (pa, pb) in va.iter().zip(vb.iter()) {
                if pa != pb {
                    return pa.cmp(pb);
                }
            }
            va.len().cmp(&vb.len())
        }
        _ => a.cmp(b),
    }
}

/// Extracts `X.Y.Z` from a release tag, if it matches `compound-engineering-v*`.
fn ce_version(tag: &str) -> Option<&str> {
    tag.strip_prefix("compound-engineering-v")
        .filter(|v| !v.is_empty())
}

/// Returns the latest release tag matching `compound-engineering-v*` from a
/// GitHub releases API payload (SF-1), or `None` when no release matches.
pub fn latest_ce_release(payload: &[u8]) -> Result<Option<String>, CeError> {
    let releases: Vec<Value> = serde_json::from_slice(payload)?;
    let mut latest: Option<(String, String)> = None; // (version, tag)
    for release in releases {
        if let Some(tag) = release.get("tag_name").and_then(Value::as_str) {
            if let Some(version) = ce_version(tag) {
                let current = latest.as_ref().map(|(v, _)| v.as_str()).unwrap_or("");
                if latest.is_none() || compare_versions(version, current) == Ordering::Greater {
                    latest = Some((version.to_string(), tag.to_string()));
                }
            }
        }
    }
    Ok(latest.map(|(_, tag)| tag))
}

/// Builds the GitHub API bearer header from the `CE_AI_GITHUB_TOKEN` value.
pub fn auth_header(token: Option<&str>) -> Option<String> {
    token
        .filter(|t| !t.is_empty())
        .map(|t| format!("Bearer {t}"))
}

/// Reads the optional GitHub token from `CE_AI_GITHUB_TOKEN`, `GITHUB_TOKEN`,
/// or `GH_TOKEN` environment variables.
pub fn github_token_from_env() -> Option<String> {
    for var in ["CE_AI_GITHUB_TOKEN", "GITHUB_TOKEN", "GH_TOKEN"] {
        if let Ok(tok) = std::env::var(var) {
            let trimmed = tok.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Resolves a GitHub token from environment variables or falls back to
/// GitHub CLI (`gh auth token`) when available.
pub fn resolve_github_token() -> Option<String> {
    github_token_from_env().or_else(|| {
        std::process::Command::new("gh")
            .args(["auth", "token"])
            .output()
            .ok()
            .filter(|out| out.status.success())
            .and_then(|out| {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            })
    })
}

/// Maps a resolved release tag to its pinned `(version, url)` pair. A
/// missing tag is a usage error — ce-ai never falls back to the mutable
/// `main` branch.
pub fn pinned_version_and_url(tag: Option<String>) -> Result<(String, String), CeError> {
    match tag {
        Some(tag) => {
            let url = tag_tarball_url(&tag);
            Ok((tag, url))
        }
        None => Err(CeError::Usage(
            "no 'compound-engineering-v*' release found on GitHub — install a pinned tag with '--to <tag>' or a local tree with '--source <path>'".to_string(),
        )),
    }
}

/// Public releases web page route (resolves to latest tag via 302 redirect).
pub fn releases_latest_web_url() -> String {
    format!("https://github.com/{PLUGIN_REPO}/releases/latest")
}

/// Public releases Atom feed (contains list of recent release tags without API rate limits).
pub fn releases_atom_feed_url() -> String {
    format!("https://github.com/{PLUGIN_REPO}/releases.atom")
}

/// Extracts a `compound-engineering-v*` tag from a redirect URL (e.g. `.../releases/tag/compound-engineering-v3.24.0`).
pub fn extract_tag_from_redirect_url(url: &str) -> Option<String> {
    let marker = "/releases/tag/";
    if let Some(pos) = url.find(marker) {
        let after = &url[pos + marker.len()..];
        let tag = after
            .trim_matches('/')
            .split(['?', '#'])
            .next()
            .unwrap_or("");
        if tag.starts_with("compound-engineering-v") && !tag.is_empty() {
            return Some(tag.to_string());
        }
    }
    None
}

/// Extracts the newest `compound-engineering-v*` release tag from an Atom feed payload.
pub fn extract_latest_tag_from_atom_feed(feed: &str) -> Option<String> {
    let mut matching_tags: Vec<String> = Vec::new();
    let marker = "/releases/tag/";
    for line in feed.lines() {
        if let Some(pos) = line.find(marker) {
            let after = &line[pos + marker.len()..];
            let tag: String = after
                .chars()
                .take_while(|c| {
                    *c != '"' && *c != '\'' && *c != '<' && *c != '/' && *c != '?' && *c != '#'
                })
                .collect();
            if tag.starts_with("compound-engineering-v")
                && !tag.is_empty()
                && !matching_tags.contains(&tag)
            {
                matching_tags.push(tag);
            }
        }
    }

    matching_tags.into_iter().max_by(|a, b| {
        let va = ce_version(a).unwrap_or(a);
        let vb = ce_version(b).unwrap_or(b);
        compare_versions(va, vb)
    })
}

/// Fallback release resolver using unauthenticated web redirect and Atom feed.
/// Free of GitHub REST API rate limits (zero friction for end users).
pub fn resolve_latest_release_fallback(
    client: &reqwest::blocking::Client,
) -> Result<Option<String>, CeError> {
    // Attempt 1: Web redirect from /releases/latest
    if let Ok(res) = client
        .get(releases_latest_web_url())
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
    {
        if let Some(tag) = extract_tag_from_redirect_url(res.url().as_str()) {
            return Ok(Some(tag));
        }
    }

    // Attempt 2: Atom feed from /releases.atom
    if let Ok(res) = client
        .get(releases_atom_feed_url())
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
    {
        if res.status().is_success() {
            if let Ok(text) = res.text() {
                if let Some(tag) = extract_latest_tag_from_atom_feed(&text) {
                    return Ok(Some(tag));
                }
            }
        }
    }

    Ok(None)
}

/// Resolves the latest CE release tag via the GitHub API with seamless zero-friction
/// web fallback when unauthenticated or rate-limited (SF-1). Resolution never falls
/// back to the mutable `main` branch.
pub fn resolve_latest_release(
    client: &reqwest::blocking::Client,
    token: Option<&str>,
) -> Result<Option<String>, CeError> {
    let mut request = client.get(releases_api_url());
    if let Some(header) = auth_header(token) {
        request = request.header(reqwest::header::AUTHORIZATION, header);
    }
    let guidance = "pin a tag with '--to <tag>' or use a local tree with '--source <path>'";

    // 1. Attempt REST API
    let api_result = request
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send();

    if let Ok(response) = api_result {
        if response.status().is_success() {
            if let Ok(bytes) = response.bytes() {
                if let Ok(Some(tag)) = latest_ce_release(&bytes) {
                    return Ok(Some(tag));
                }
            }
        }
    }

    // 2. Zero-friction web fallback (resilient against 403 rate limits)
    if let Ok(Some(tag)) = resolve_latest_release_fallback(client) {
        return Ok(Some(tag));
    }

    Err(CeError::Network(format!(
        "Failed to resolve latest Compound Engineering release from GitHub (API & web fallback exhausted) — {guidance}"
    )))
}

#[cfg(test)]
#[path = "tests/release.rs"]
mod tests;
