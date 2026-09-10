//! Binary self-update engine: target detection, release resolution,
//! integrity verification, safe archive extraction, and running executable replacement.

use crate::error::CeError;
use crate::state::diff::sha256_hex;

/// Returns the official target triple string for the host architecture and OS,
/// matching `.github/workflows/release.yml`. Returns `None` on unsupported platforms.
pub fn current_target() -> Option<&'static str> {
    if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        Some("x86_64-unknown-linux-musl")
    } else if cfg!(all(target_arch = "aarch64", target_os = "linux")) {
        Some("aarch64-unknown-linux-musl")
    } else if cfg!(all(target_arch = "x86_64", target_os = "macos")) {
        Some("x86_64-apple-darwin")
    } else if cfg!(all(target_arch = "aarch64", target_os = "macos")) {
        Some("aarch64-apple-darwin")
    } else if cfg!(all(target_arch = "x86_64", target_os = "windows")) {
        Some("x86_64-pc-windows-msvc")
    } else if cfg!(all(target_arch = "aarch64", target_os = "windows")) {
        Some("aarch64-pc-windows-msvc")
    } else {
        None
    }
}

/// Formats the expected release asset archive name for a target triple.
pub fn asset_name_for_target(target: &str) -> String {
    if target.contains("windows") {
        format!("ce-ai-{target}.zip")
    } else {
        format!("ce-ai-{target}.tar.gz")
    }
}

/// Extracts the expected SHA256 hex digest for `asset_name` from SHA256SUMS.txt content.
pub fn parse_sha256sums(content: &str, asset_name: &str) -> Option<String> {
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == asset_name {
            return Some(parts[0].to_lowercase());
        }
    }
    None
}

/// Verifies that `bytes` hashes to `expected_sha256`.
/// Returns `CeError::Verification` (exit code 6) on mismatch.
pub fn verify_checksum(bytes: &[u8], expected_sha256: &str) -> Result<(), CeError> {
    let actual_hex = sha256_hex(bytes);
    if actual_hex.eq_ignore_ascii_case(expected_sha256) {
        Ok(())
    } else {
        Err(CeError::Verification(format!(
            "SHA256 checksum mismatch: expected {expected_sha256}, got {actual_hex}"
        )))
    }
}

/// Opens archive bytes as a tar reader, transparently handling gzip via magic bytes.
fn tar_reader(bytes: &[u8]) -> Box<dyn std::io::Read + '_> {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        Box::new(flate2::read::GzDecoder::new(bytes))
    } else {
        Box::new(bytes)
    }
}

/// Safely extracts the `ce-ai` (or `ce-ai.exe`) binary bytes from a `.tar.gz` (or uncompressed `.tar`) archive.
/// All entries are checked against Zip-Slip path traversal before extraction.
pub fn extract_binary_from_tar(bytes: &[u8]) -> Result<Vec<u8>, CeError> {
    let mut archive = tar::Archive::new(tar_reader(bytes));
    let mut binary_content: Option<Vec<u8>> = None;

    let entries = archive
        .entries()
        .map_err(|e| CeError::Runtime(format!("tar error: {e}")))?;

    for entry in entries {
        let mut entry = entry.map_err(|e| CeError::Runtime(format!("tar error: {e}")))?;
        let path = entry.path()?.into_owned();
        if !crate::source::archive::is_safe_relative_path(&path) {
            return Err(CeError::Runtime(format!(
                "unsafe archive entry path: {}",
                path.display()
            )));
        }
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if (file_name == "ce-ai" || file_name == "ce-ai.exe")
            && (entry.header().entry_type().is_file()
                || entry.header().entry_type() == tar::EntryType::Continuous)
        {
            let mut buf = Vec::new();
            use std::io::Read;
            entry.read_to_end(&mut buf)?;
            binary_content = Some(buf);
        }
    }

    binary_content.ok_or_else(|| CeError::Runtime("ce-ai binary not found in archive".to_string()))
}

/// Safely extracts the `ce-ai.exe` (or `ce-ai`) binary bytes from a `.zip` archive.
/// All entries are checked against Zip-Slip path traversal before extraction.
pub fn extract_binary_from_zip(bytes: &[u8]) -> Result<Vec<u8>, CeError> {
    let cursor = std::io::Cursor::new(bytes);
    let mut zip =
        zip::ZipArchive::new(cursor).map_err(|e| CeError::Runtime(format!("zip error: {e}")))?;
    let mut binary_content: Option<Vec<u8>> = None;

    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|e| CeError::Runtime(format!("zip error: {e}")))?;

        let enclosed = match file.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => {
                return Err(CeError::Runtime(format!(
                    "unsafe archive entry path: {}",
                    file.name()
                )));
            }
        };

        if !crate::source::archive::is_safe_relative_path(&enclosed) {
            return Err(CeError::Runtime(format!(
                "unsafe archive entry path: {}",
                enclosed.display()
            )));
        }

        let file_name = enclosed.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if (file_name == "ce-ai" || file_name == "ce-ai.exe") && file.is_file() {
            let mut buf = Vec::new();
            use std::io::Read;
            file.read_to_end(&mut buf)?;
            binary_content = Some(buf);
        }
    }

    binary_content
        .ok_or_else(|| CeError::Runtime("ce-ai binary not found in zip archive".to_string()))
}

/// Dispatches binary extraction based on whether the archive is a `.zip` or `.tar.gz`.
pub fn extract_binary(archive_bytes: &[u8], is_zip: bool) -> Result<Vec<u8>, CeError> {
    if is_zip {
        extract_binary_from_zip(archive_bytes)
    } else {
        extract_binary_from_tar(archive_bytes)
    }
}

/// Replaces the target executable with `binary_bytes`.
///
/// On POSIX systems, writes to a temporary file in the same directory, applies `0o755` permissions,
/// and performs an atomic `rename(2)` over `target_path`.
///
/// On Windows, renames the running executable to `<target_path>.old` (since Windows permits renaming open files
/// with FILE_SHARE_DELETE), moves the replacement file into `target_path`, and attempts deletion of `.old`.
pub fn replace_executable(
    target_path: &std::path::Path,
    binary_bytes: &[u8],
) -> Result<(), CeError> {
    let parent = target_path.parent().ok_or_else(|| {
        CeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "Executable path has no parent directory: {}",
                target_path.display()
            ),
        ))
    })?;

    if !parent.is_dir() {
        return Err(CeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Directory does not exist: {}", parent.display()),
        )));
    }

    #[cfg(unix)]
    {
        replace_executable_posix(parent, target_path, binary_bytes)
    }

    #[cfg(windows)]
    {
        replace_executable_windows(parent, target_path, binary_bytes)
    }
}

#[cfg(unix)]
fn replace_executable_posix(
    parent: &std::path::Path,
    target_path: &std::path::Path,
    binary_bytes: &[u8],
) -> Result<(), CeError> {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let mut temp_file = tempfile::Builder::new()
        .prefix(".ce-ai-self-update-")
        .tempfile_in(parent)?;

    temp_file.write_all(binary_bytes)?;
    temp_file.flush()?;

    let perms = std::fs::Permissions::from_mode(0o755);
    std::fs::set_permissions(temp_file.path(), perms)?;

    temp_file
        .persist(target_path)
        .map_err(|e| CeError::Io(e.error))?;

    Ok(())
}

#[cfg(windows)]
fn replace_executable_windows(
    parent: &std::path::Path,
    target_path: &std::path::Path,
    binary_bytes: &[u8],
) -> Result<(), CeError> {
    use std::io::Write;

    let old_path = parent.join(format!(
        "{}.old",
        target_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("ce-ai.exe")
    ));
    if old_path.exists() {
        let _ = std::fs::remove_file(&old_path);
    }
    let target_old = if old_path.exists() {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        parent.join(format!(
            "{}.old.{}",
            target_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("ce-ai.exe"),
            ts
        ))
    } else {
        old_path
    };

    let temp_file = tempfile::Builder::new()
        .prefix(".ce-ai-self-update-")
        .suffix(".exe")
        .tempfile_in(parent)?;
    let temp_path = temp_file.path().to_path_buf();
    let (mut file, _) = temp_file.keep().map_err(|e| CeError::Io(e.error))?;
    file.write_all(binary_bytes)?;
    file.flush()?;
    drop(file);

    if target_path.exists() {
        std::fs::rename(target_path, &target_old)?;
    }

    if let Err(err) = std::fs::rename(&temp_path, target_path) {
        if target_old.exists() {
            let _ = std::fs::rename(&target_old, target_path);
        }
        let _ = std::fs::remove_file(&temp_path);
        return Err(CeError::Io(err));
    }

    let _ = std::fs::remove_file(&target_old);
    Ok(())
}

/// Cleans up any leftover temporary files or old executables from previous updates.
pub fn cleanup_stale_update_files(dir: &std::path::Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with(".ce-ai-self-update-")
                    || file_name.ends_with(".old")
                    || file_name.contains(".old.")
                {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
}

/// GitHub repository for the ce-ai CLI.
pub const CLI_REPO: &str = "mastepanoski/ce-ai";

pub fn cli_releases_api_url() -> String {
    format!("https://api.github.com/repos/{CLI_REPO}/releases/latest")
}

pub fn cli_release_tag_api_url(tag: &str) -> String {
    format!("https://api.github.com/repos/{CLI_REPO}/releases/tags/{tag}")
}

pub fn cli_releases_latest_web_url() -> String {
    format!("https://github.com/{CLI_REPO}/releases/latest")
}

pub fn cli_releases_atom_feed_url() -> String {
    format!("https://github.com/{CLI_REPO}/releases.atom")
}

#[derive(Debug, Clone)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
}

#[derive(Debug, Clone)]
pub struct CliRelease {
    pub tag: String,
    pub version: String,
    pub assets: Vec<ReleaseAsset>,
}

fn version_components(v: &str) -> Option<Vec<u64>> {
    let clean = v.strip_prefix('v').unwrap_or(v);
    clean
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect()
}

pub fn compare_cli_versions(a: &str, b: &str) -> std::cmp::Ordering {
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

pub fn parse_cli_release_payload(payload: &[u8]) -> Result<CliRelease, CeError> {
    let val: serde_json::Value = serde_json::from_slice(payload)?;
    let tag = val
        .get("tag_name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| CeError::Runtime("missing 'tag_name' in GitHub release payload".into()))?
        .to_string();

    let version = tag.strip_prefix('v').unwrap_or(&tag).to_string();

    let mut assets = Vec::new();
    if let Some(arr) = val.get("assets").and_then(serde_json::Value::as_array) {
        for item in arr {
            let name = item
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string();
            let download_url = item
                .get("browser_download_url")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_string();
            if !name.is_empty() && !download_url.is_empty() {
                assets.push(ReleaseAsset { name, download_url });
            }
        }
    }

    Ok(CliRelease {
        tag,
        version,
        assets,
    })
}

pub fn extract_tag_from_cli_redirect_url(url: &str) -> Option<String> {
    let marker = "/releases/tag/";
    if let Some(pos) = url.find(marker) {
        let after = &url[pos + marker.len()..];
        let tag = after
            .trim_matches('/')
            .split(['?', '#'])
            .next()
            .unwrap_or("");
        if tag.starts_with('v') && tag.len() > 1 && !tag.contains('/') {
            return Some(tag.to_string());
        }
    }
    None
}

pub fn extract_latest_tag_from_cli_atom_feed(feed: &str) -> Option<String> {
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
            if tag.starts_with('v') && tag.len() > 1 && !matching_tags.contains(&tag) {
                matching_tags.push(tag);
            }
        }
    }

    matching_tags
        .into_iter()
        .max_by(|a, b| compare_cli_versions(a, b))
}

pub fn resolve_latest_cli_release(
    client: &reqwest::blocking::Client,
    token: Option<&str>,
) -> Result<CliRelease, CeError> {
    let mut request = client.get(cli_releases_api_url());
    if let Some(header) = crate::source::release::auth_header(token) {
        request = request.header(reqwest::header::AUTHORIZATION, header);
    }

    let api_result = request
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send();

    if let Ok(response) = api_result {
        if response.status().is_success() {
            if let Ok(bytes) = response.bytes() {
                if let Ok(release) = parse_cli_release_payload(&bytes) {
                    return Ok(release);
                }
            }
        }
    }

    // Zero-friction web fallback
    if let Ok(res) = client
        .get(cli_releases_latest_web_url())
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
    {
        if let Some(tag) = extract_tag_from_cli_redirect_url(res.url().as_str()) {
            return resolve_cli_release_by_tag(client, token, &tag);
        }
    }

    if let Ok(res) = client
        .get(cli_releases_atom_feed_url())
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
    {
        if res.status().is_success() {
            if let Ok(text) = res.text() {
                if let Some(tag) = extract_latest_tag_from_cli_atom_feed(&text) {
                    return resolve_cli_release_by_tag(client, token, &tag);
                }
            }
        }
    }

    Err(CeError::Network(
        "Failed to resolve latest ce-ai CLI release from GitHub (API & web fallback exhausted)"
            .to_string(),
    ))
}

pub fn resolve_cli_release_by_tag(
    client: &reqwest::blocking::Client,
    token: Option<&str>,
    tag: &str,
) -> Result<CliRelease, CeError> {
    let mut request = client.get(cli_release_tag_api_url(tag));
    if let Some(header) = crate::source::release::auth_header(token) {
        request = request.header(reqwest::header::AUTHORIZATION, header);
    }

    let response = request
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
        .map_err(|e| CeError::Network(format!("failed to fetch release for {tag}: {e}")))?;

    if response.status().is_success() {
        let bytes = response.bytes().map_err(|e| {
            CeError::Network(format!("failed to read release response for {tag}: {e}"))
        })?;
        return parse_cli_release_payload(&bytes);
    }

    // Fallback: construct synthesized release asset URLs for known tag
    let version = tag.strip_prefix('v').unwrap_or(tag).to_string();
    let known_targets = [
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-pc-windows-msvc",
        "aarch64-pc-windows-msvc",
    ];

    let mut assets = Vec::new();
    for target in known_targets {
        let name = asset_name_for_target(target);
        let download_url = format!("https://github.com/{CLI_REPO}/releases/download/{tag}/{name}");
        assets.push(ReleaseAsset { name, download_url });
    }
    assets.push(ReleaseAsset {
        name: "SHA256SUMS.txt".to_string(),
        download_url: format!(
            "https://github.com/{CLI_REPO}/releases/download/{tag}/SHA256SUMS.txt"
        ),
    });

    Ok(CliRelease {
        tag: tag.to_string(),
        version,
        assets,
    })
}

pub fn download_release_asset_and_sums(
    client: &reqwest::blocking::Client,
    release: &CliRelease,
    target: &str,
) -> Result<(Vec<u8>, String), CeError> {
    let expected_asset_name = asset_name_for_target(target);

    // Fallback: if release.assets doesn't contain the musl asset, check for older gnu asset
    let fallback_asset_name = if target == "x86_64-unknown-linux-musl" {
        Some("ce-ai-x86_64-unknown-linux-gnu.tar.gz")
    } else if target == "aarch64-unknown-linux-musl" {
        Some("ce-ai-aarch64-unknown-linux-gnu.tar.gz")
    } else {
        None
    };

    let (chosen_asset_name, asset_url) = if let Some(asset) = release
        .assets
        .iter()
        .find(|a| a.name == expected_asset_name)
    {
        (expected_asset_name.clone(), asset.download_url.clone())
    } else if let Some(fallback) =
        fallback_asset_name.and_then(|f| release.assets.iter().find(|a| a.name == f))
    {
        (fallback.name.clone(), fallback.download_url.clone())
    } else {
        (
            expected_asset_name.clone(),
            format!(
                "https://github.com/{CLI_REPO}/releases/download/{}/{expected_asset_name}",
                release.tag
            ),
        )
    };

    let sums_url = release
        .assets
        .iter()
        .find(|a| a.name == "SHA256SUMS.txt")
        .map(|a| a.download_url.clone())
        .unwrap_or_else(|| {
            format!(
                "https://github.com/{CLI_REPO}/releases/download/{}/SHA256SUMS.txt",
                release.tag
            )
        });

    // Download archive
    let archive_resp = client
        .get(&asset_url)
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
        .map_err(|e| CeError::Network(format!("failed to download {asset_url}: {e}")))?;

    if !archive_resp.status().is_success() {
        return Err(CeError::Network(format!(
            "failed to download release asset {chosen_asset_name} (HTTP {})",
            archive_resp.status()
        )));
    }

    let archive_bytes = archive_resp
        .bytes()
        .map_err(|e| CeError::Network(format!("failed to read release asset bytes: {e}")))?
        .to_vec();

    // Download SHA256SUMS.txt
    let sums_resp = client
        .get(&sums_url)
        .header(reqwest::header::USER_AGENT, "ce-ai/0.1.0")
        .send()
        .map_err(|e| CeError::Network(format!("failed to download {sums_url}: {e}")))?;

    if !sums_resp.status().is_success() {
        return Err(CeError::Network(format!(
            "failed to download SHA256SUMS.txt (HTTP {})",
            sums_resp.status()
        )));
    }

    let sums_text = sums_resp
        .text()
        .map_err(|e| CeError::Network(format!("failed to read SHA256SUMS.txt: {e}")))?;

    let expected_sha256 = parse_sha256sums(&sums_text, &chosen_asset_name).ok_or_else(|| {
        CeError::Verification(format!(
            "asset '{chosen_asset_name}' not listed in SHA256SUMS.txt"
        ))
    })?;

    Ok((archive_bytes, expected_sha256))
}

#[cfg(test)]
#[path = "tests/binary_release.rs"]
mod tests;
