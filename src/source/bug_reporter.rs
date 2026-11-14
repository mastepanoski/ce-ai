//! Intelligent Upstream Bug Detection, Deduplication & Reporting (Issue #426).
//!
//! Provides zero-data-leakage privacy sanitization (ISO/IEC 27001 compliant),
//! GitHub issue template formatting, upstream issue deduplication, and `gh` CLI
//! execution with web browser fallback URLs.

use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::CeError;

pub const UPSTREAM_REPO: &str = "mastepanoski/ce-ai";

/// Diagnostic bundle compiled for an internal ce-ai bug report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BugReportBundle {
    pub ce_version: String,
    pub os: String,
    pub arch: String,
    pub target_harness: String,
    pub command_invoked: String,
    pub error_message: String,
    pub logs_sanitized: String,
}

/// Upstream GitHub issue match found during deduplication search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamIssueMatch {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub state: String,
}

/// Status of the GitHub CLI (`gh`) on the host machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GhStatus {
    Ready,
    Unauthenticated,
    NotInstalled,
}

/// Percent-encodes text conforming to RFC 3986 for safe URL query parameters.
pub fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

/// Sanitizes logs and traces by stripping proprietary paths, API keys, tokens, and secrets.
/// Complies with ISO/IEC 27001 data confidentiality and NIST AI RMF governance controls.
pub fn sanitize_text(raw: &str, home: Option<&Path>, project_root: Option<&Path>) -> String {
    crate::security::pii::redact_pii_and_secrets_with_paths(raw, home, project_root)
}

/// Formats a `BugReportBundle` into markdown conforming to `.github/ISSUE_TEMPLATE/bug_report.yml`.
pub fn format_github_issue_body(bundle: &BugReportBundle) -> String {
    format!(
        "### Bug Description\n{}\n\n### Operating System\n{} ({})\n\n### Target Harness\n{}\n\n### Steps To Reproduce\n1. Invoked command: `{}`\n2. Encountered error: `{}`\n\n### Expected Behavior\nCommand completes successfully without internal errors or crashes.\n\n### CLI Logs & Error Output\n```shell\n{}\n```\n",
        bundle.error_message,
        bundle.os,
        bundle.arch,
        bundle.target_harness,
        bundle.command_invoked,
        bundle.error_message,
        bundle.logs_sanitized
    )
}

/// Generates a prefilled GitHub web URL for creating an issue with pre-populated title and body.
pub fn generate_web_issue_url(title: &str, body: &str) -> String {
    let enc_title = url_encode(title);
    let enc_body = url_encode(body);
    format!(
        "https://github.com/{}/issues/new?title={}&body={}&labels=bug",
        UPSTREAM_REPO, enc_title, enc_body
    )
}

/// Checks the installation and authentication readiness of the GitHub CLI (`gh`).
pub fn check_gh_status() -> GhStatus {
    match Command::new("gh").args(["auth", "status"]).output() {
        Ok(out) if out.status.success() => GhStatus::Ready,
        Ok(_) => GhStatus::Unauthenticated,
        Err(_) => GhStatus::NotInstalled,
    }
}

/// Returns platform-specific installation command guidance for the GitHub CLI.
pub fn install_instructions(os: &str) -> &'static str {
    match os {
        "macos" => "brew install gh && gh auth login",
        "linux" => {
            "sudo apt install gh && gh auth login (or use your distribution package manager)"
        }
        "windows" => "winget install --id GitHub.cli && gh auth login",
        _ => "Visit https://cli.github.com for installation instructions",
    }
}

/// Searches `mastepanoski/ce-ai` for open or recent issues matching `query`.
pub fn search_upstream_issues(query: &str) -> Vec<UpstreamIssueMatch> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    // Try gh issue list --search
    if let Ok(out) = Command::new("gh")
        .args([
            "issue",
            "list",
            "--repo",
            UPSTREAM_REPO,
            "--search",
            query,
            "--limit",
            "5",
            "--json",
            "number,title,url,state",
        ])
        .output()
    {
        if out.status.success() {
            if let Ok(issues) = serde_json::from_slice::<Vec<UpstreamIssueMatch>>(&out.stdout) {
                return issues;
            }
        }
    }

    Vec::new()
}

/// Submits a bug report directly to `mastepanoski/ce-ai` via the GitHub CLI.
pub fn submit_issue_via_gh(title: &str, body: &str) -> Result<String, CeError> {
    let out = Command::new("gh")
        .args([
            "issue",
            "create",
            "--repo",
            UPSTREAM_REPO,
            "--title",
            title,
            "--body",
            body,
            "--label",
            "bug",
        ])
        .output()
        .map_err(|e| CeError::Runtime(format!("Failed to execute 'gh issue create': {e}")))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(CeError::Runtime(format!(
            "gh issue create failed: {stderr}"
        )));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.trim().to_string())
}

#[cfg(test)]
#[path = "tests/bug_reporter.rs"]
mod tests;
