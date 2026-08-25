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

/// Reads the optional `CE_AI_GITHUB_TOKEN` environment variable.
pub fn github_token_from_env() -> Option<String> {
    std::env::var("CE_AI_GITHUB_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
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

/// Resolves the latest CE release tag via the GitHub API (SF-1); network call,
/// exercised by E2E rather than unit tests. Every transport, HTTP, or payload
/// failure is an explicit [`CeError::Network`]; `Ok(None)` means the API
/// answered successfully but no `compound-engineering-v*` release exists.
pub fn resolve_latest_release(
    client: &reqwest::blocking::Client,
    token: Option<&str>,
) -> Result<Option<String>, CeError> {
    let mut request = client.get(releases_api_url());
    if let Some(header) = auth_header(token) {
        request = request.header(reqwest::header::AUTHORIZATION, header);
    }
    let guidance = "pin a tag with '--to <tag>' or use a local tree with '--source <path>'; tip: set CE_AI_GITHUB_TOKEN to authenticate requests";
    let response = match request
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
    {
        Ok(res) => res,
        Err(err) => {
            return Err(CeError::Network(format!(
                "GitHub API release query failed ({err}) — ce-ai never falls back to the mutable main branch; {guidance}"
            )));
        }
    };

    if !response.status().is_success() {
        return Err(CeError::Network(format!(
            "GitHub API returned HTTP {} when querying releases — ce-ai never falls back to the mutable main branch; pin a tag with '--to <tag>' or use a local tree with '--source <path>'; tip: set CE_AI_GITHUB_TOKEN to authenticate or raise rate limits",
            response.status()
        )));
    }
    let body = match response.bytes() {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err(CeError::Network(format!(
                "failed to read GitHub API response bytes ({err}) — ce-ai never falls back to the mutable main branch; pin a tag with '--to <tag>' or use a local tree with '--source <path>'"
            )));
        }
    };
    latest_ce_release(&body).map_err(|err| {
        CeError::Network(format!("failed to parse GitHub API release payload: {err}"))
    })
}

#[cfg(test)]
mod tests {
    use crate::source::release::{
        auth_header, github_token_from_env, latest_ce_release, pinned_version_and_url,
        tag_tarball_url,
    };

    #[test]
    fn picks_latest_compound_engineering_release_tag() {
        let payload = br#"[
            {"tag_name": "v2.5.0"},
            {"tag_name": "compound-engineering-v3.10.0"},
            {"tag_name": "compound-engineering-v3.4.2"},
            {"tag_name": "v1.0.0"},
            {"tag_name": "compound-engineering-v2.9.1"}
        ]"#;
        assert_eq!(
            latest_ce_release(payload).unwrap().as_deref(),
            Some("compound-engineering-v3.10.0")
        );
    }

    #[test]
    fn no_matching_release_yields_none() {
        let payload = br#"[{"tag_name": "v2.5.0"}, {"tag_name": "v1.0.0"}]"#;
        assert_eq!(latest_ce_release(payload).unwrap(), None);
    }

    #[test]
    fn pinned_version_resolves_tag_tarball_url() {
        let (version, url) =
            pinned_version_and_url(Some("compound-engineering-v3.4.2".to_string())).unwrap();
        assert_eq!(version, "compound-engineering-v3.4.2");
        assert_eq!(url, tag_tarball_url("compound-engineering-v3.4.2"));
    }

    #[test]
    fn missing_release_is_a_usage_error_never_main_fallback() {
        let err = pinned_version_and_url(None).unwrap_err();
        assert_eq!(err.exit_code(), 2);
        assert!(err.to_string().contains("--to <tag>"));
        assert!(!err.to_string().contains("main.tar.gz"));
    }

    #[test]
    fn tag_tarball_url_points_at_release_tag() {
        let url = tag_tarball_url("compound-engineering-v3.4.2");
        assert!(url.ends_with("/archive/refs/tags/compound-engineering-v3.4.2.tar.gz"));
    }

    #[test]
    fn github_token_builds_bearer_header() {
        assert_eq!(
            auth_header(Some("secret")),
            Some("Bearer secret".to_string())
        );
        assert_eq!(auth_header(None), None);
    }

    #[test]
    fn github_token_reads_from_environment() {
        std::env::set_var("CE_AI_GITHUB_TOKEN", "tok-123");
        assert_eq!(github_token_from_env().as_deref(), Some("tok-123"));
        std::env::remove_var("CE_AI_GITHUB_TOKEN");
        assert_eq!(github_token_from_env(), None);
    }
}
