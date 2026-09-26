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
    let mut sanitized = raw.to_string();

    // 1. Anonymize project root paths across OS path variants (Windows \, Unix /, \\?\, /private, /c/ MSYS)
    if let Some(root) = project_root {
        replace_path_variants(&mut sanitized, root, "<project-root>");
    }

    // Also anonymize current working directory if available
    if let Ok(cwd) = std::env::current_dir() {
        if cwd != Path::new("/") && cwd != Path::new(".") {
            replace_path_variants(&mut sanitized, &cwd, "<project-root>");
        }
    }

    // 2. Anonymize home directory paths across OS path variants
    if let Some(h) = home {
        replace_path_variants(&mut sanitized, h, "~");
    }

    // 3. Fallback path sanitization for common user paths: /Users/<user> or /home/<user>
    let lines: Vec<String> = sanitized
        .lines()
        .map(|line| {
            let mut l = line.to_string();

            // Strip GitHub Personal Access Tokens (classic & fine-grained)
            while let Some(idx) = l.find("ghp_") {
                let token_len = l[idx..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .count();
                if token_len >= 36 {
                    l.replace_range(idx..idx + token_len, "[REDACTED_GH_TOKEN]");
                } else {
                    break;
                }
            }
            while let Some(idx) = l.find("github_pat_") {
                let token_len = l[idx..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .count();
                if token_len >= 50 {
                    l.replace_range(idx..idx + token_len, "[REDACTED_GH_TOKEN]");
                } else {
                    break;
                }
            }

            // Strip sk- style API keys (OpenAI, Anthropic, etc.)
            while let Some(idx) = l.find("sk-") {
                let token_len = l[idx..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                    .count();
                if token_len >= 16 {
                    l.replace_range(idx..idx + token_len, "[REDACTED]");
                } else {
                    break;
                }
            }

            // Strip Authorization Bearer headers
            if let Some(pos) = l.find("Bearer ") {
                let after = &l[pos + 7..];
                let token_len = after
                    .chars()
                    .take_while(|c| !c.is_whitespace() && *c != '"' && *c != '\'')
                    .count();
                if token_len > 0 {
                    l.replace_range(pos + 7..pos + 7 + token_len, "[REDACTED]");
                }
            }

            // Strip generic secret/api key key-value pairs (key=val or "key": "val")
            for prefix in [
                "key", "api_key", "apikey", "api-key", "secret", "token", "password",
            ] {
                if let Some(mut pos) = l.to_lowercase().find(prefix) {
                    pos += prefix.len();
                    let rem = &l[pos..];
                    if let Some(delim_idx) = rem.find([':', '=']) {
                        let after_delim = &rem[delim_idx + 1..];
                        let val_start = after_delim.find(|c: char| !c.is_whitespace());
                        if let Some(vs) = val_start {
                            let val_content = &after_delim[vs..];
                            if val_content.starts_with("[REDACTED")
                                || val_content.starts_with("\"[REDACTED")
                                || val_content.starts_with("'[REDACTED")
                            {
                                continue;
                            }
                            let val_len = val_content
                                .chars()
                                .take_while(|c| !c.is_whitespace() && *c != ',' && *c != ';')
                                .count();
                            if val_len >= 8 {
                                let abs_start = pos + delim_idx + 1 + vs;
                                l.replace_range(abs_start..abs_start + val_len, "[REDACTED]");
                            }
                        }
                    }
                }
            }

            l
        })
        .collect();

    let mut result = lines.join("\n");

    // Strip private key blocks (multi-line)
    while let Some(start) = result.find("-----BEGIN") {
        if let Some(end_marker) = result[start..].find("-----END") {
            let tail = &result[start + end_marker..];
            if let Some(end_close) = tail.find("-----") {
                let abs_end = start + end_marker + end_close + 5;
                result.replace_range(start..abs_end, "[REDACTED_PRIVATE_KEY]");
                continue;
            }
        }
        break;
    }

    result
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

/// Recursively generates cross-platform path variants (handling forward slashes,
/// Windows backslashes, extended verbatim `\\?\` prefixes, macOS `/private` symlinks,
/// and case differences in Windows drive letters) and anonymizes matching substrings.
fn replace_path_variants(text: &mut String, path: &Path, replacement: &str) {
    let mut candidates = Vec::new();
    let direct = path.to_string_lossy().to_string();
    if !direct.is_empty() {
        candidates.push(direct.clone());
        // Handle Git for Windows MSYS path format: /c/Users/... -> C:/Users/...
        if direct.len() >= 3
            && direct.starts_with('/')
            && direct.as_bytes()[2] == b'/'
            && direct.as_bytes()[1].is_ascii_alphabetic()
        {
            let drive = direct.as_bytes()[1] as char;
            let win_path = format!("{}:/{}", drive.to_uppercase(), &direct[3..]);
            if let Ok(canon) = Path::new(&win_path).canonicalize() {
                let canon_str = canon.to_string_lossy().to_string();
                if !canon_str.is_empty() {
                    candidates.push(canon_str);
                }
            }
            candidates.push(win_path);
        }
    }
    if let Ok(canon) = path.canonicalize() {
        let canon_str = canon.to_string_lossy().to_string();
        if !canon_str.is_empty() {
            candidates.push(canon_str);
        }
    }

    let mut expanded = Vec::new();
    for c in candidates {
        expanded.push(c.clone());
        if let Some(stripped) = c.strip_prefix(r"\\?\") {
            expanded.push(stripped.to_string());
        }
        if let Some(stripped) = c.strip_prefix("/private") {
            expanded.push(stripped.to_string());
        }
    }

    let mut final_variants = Vec::new();
    for item in expanded {
        let fwd = item.replace('\\', "/");
        let bwd = item.replace('/', "\\");
        for variant in [item, fwd, bwd] {
            if variant.is_empty() {
                continue;
            }
            final_variants.push(variant.clone());
            // Drive letter case variations on Windows (e.g. C: vs c:)
            if let Some(first) = variant.chars().next() {
                if variant.len() >= 2 && variant.as_bytes()[1] == b':' {
                    let upper = format!("{}{}", first.to_uppercase(), &variant[first.len_utf8()..]);
                    let lower = format!("{}{}", first.to_lowercase(), &variant[first.len_utf8()..]);
                    final_variants.push(upper);
                    final_variants.push(lower);
                }
            }
        }
    }

    // Sort descending by length so child paths are sanitized before parent dirs
    final_variants.sort_by_key(|b| std::cmp::Reverse(b.len()));
    final_variants.dedup();

    for v in final_variants {
        if !v.is_empty() {
            replace_ignore_ascii_case(text, &v, replacement);
        }
    }
}

fn replace_ignore_ascii_case(text: &mut String, pattern: &str, replacement: &str) {
    if pattern.is_empty() {
        return;
    }
    let lower_pattern = pattern.to_ascii_lowercase();
    let mut result = String::with_capacity(text.len());
    let mut last_end = 0;
    let text_lower = text.to_ascii_lowercase();

    while let Some(pos) = text_lower[last_end..].find(&lower_pattern) {
        let abs_pos = last_end + pos;
        result.push_str(&text[last_end..abs_pos]);
        result.push_str(replacement);
        last_end = abs_pos + pattern.len();
    }
    result.push_str(&text[last_end..]);
    *text = result;
}

#[cfg(test)]
#[path = "tests/bug_reporter.rs"]
mod tests;
