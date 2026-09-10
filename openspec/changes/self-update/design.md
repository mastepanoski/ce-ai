# Design: Architecture & Specification for Binary Self-Update

## 1. System Architecture & Module Structure

```
src/
├── main.rs                      # Startup hook: cleanup_stale_update_files
├── error.rs                     # Standard exit codes (Verification = 6)
├── commands/
│   ├── mod.rs                   # Shared Context
│   ├── registry.rs              # Clap Commands::SelfUpdate registration & dispatch
│   ├── self_update.rs           # `ce-ai self-update` command workflow
│   └── upgrade.rs               # Extension: --bin flag delegating to self-update
└── source/
    └── binary_release.rs        # Core engine: target resolution, fetch, verify, extract, replace
```

---

## 2. Core Engine Components (`src/source/binary_release.rs`)

### 2.1 Target Triple Resolution

```rust
/// Returns the official target triple string for the host architecture and OS,
/// matching .github/workflows/release.yml. Returns None on unsupported platforms.
pub fn current_target() -> Option<&'static str> {
    if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
        Some("x86_64-unknown-linux-gnu")
    } else if cfg!(all(target_arch = "aarch64", target_os = "linux")) {
        Some("aarch64-unknown-linux-gnu")
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
```

---

### 2.2 Release Resolution & Provenance

Releases are sourced exclusively from `mastepanoski/ce-ai`.

- API Endpoint: `https://api.github.com/repos/mastepanoski/ce-ai/releases/latest`
- Unauthenticated Web Fallback: `https://github.com/mastepanoski/ce-ai/releases/latest` (via 302 redirect) and `https://github.com/mastepanoski/ce-ai/releases.atom`.
- Tag schema: `vX.Y.Z` (e.g. `v1.49.0`, `v1.50.0`).
- Asset metadata structure:
```rust
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
```

---

### 2.3 Checksum Verification & Parsing (`SHA256SUMS.txt`)

```rust
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

/// Verifies that `bytes` hashes to `expected_sha256`. Returns CeError::Verification on mismatch.
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
```

---

### 2.4 Safe Archive Extraction (Zip-Slip Defense)

Extracts the raw binary bytes from either `.tar.gz` (Unix) or `.zip` (Windows) archive bytes.

- Rejects path traversal (`..`), absolute paths, or Windows drive prefixes.
- Locates the single executable file (`ce-ai` or `ce-ai.exe`).
- Returns `Result<Vec<u8>, CeError>`.

```rust
pub fn extract_binary(archive_bytes: &[u8], is_zip: bool) -> Result<Vec<u8>, CeError>;
```

---

### 2.5 Cross-Platform Replacement Engine

```rust
pub fn replace_executable(
    target_path: &Path,
    binary_bytes: &[u8],
) -> Result<(), CeError> {
    let parent = target_path.parent().ok_or_else(|| {
        CeError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Target executable has no parent directory",
        ))
    })?;

    #[cfg(unix)]
    {
        replace_executable_posix(parent, target_path, binary_bytes)
    }

    #[cfg(windows)]
    {
        replace_executable_windows(parent, target_path, binary_bytes)
    }
}
```

#### POSIX Implementation (`replace_executable_posix`):
1. Create temporary file in `parent` using `tempfile::Builder::new().prefix(".ce-ai-self-update-").tempfile_in(parent)?`.
2. Write `binary_bytes` to temp file.
3. Set permissions `0o755` using `std::os::unix::fs::PermissionsExt`.
4. Flush & close temp file handle.
5. Atomically rename temp file over `target_path` (`std::fs::rename`).

#### Windows Implementation (`replace_executable_windows`):
1. Clean up any stale `target_path.old` in `parent` if writable.
2. If `target_path.old` cannot be deleted, generate `target_path.old.<timestamp>`.
3. Create temporary file `target_path.new` in `parent`.
4. Write `binary_bytes` and flush.
5. Rename running `target_path` -> `target_path.old`.
6. Rename `target_path.new` -> `target_path`.
7. Attempt `std::fs::remove_file(&target_path_old)` (best effort, ignore errors).

---

### 2.6 Startup Stale File Cleanup (`src/main.rs`)

```rust
pub fn cleanup_stale_update_files(dir: &Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with(".ce-ai-self-update-") || file_name.ends_with(".old") {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }
}
```

---

## 3. CLI Subcommand Contract (`ce-ai self-update`)

### Arguments:
```rust
#[derive(clap::Args, Debug, Clone)]
pub struct Args {
    /// Check for available updates without applying them.
    #[arg(long)]
    pub check: bool,

    /// Target a specific release tag (e.g. v1.50.0) instead of latest.
    #[arg(long)]
    pub to: Option<String>,

    /// Force replacement even if current version is up to date or newer.
    #[arg(long)]
    pub force: bool,
}
```

### Exit Codes:
- `0`: Success (updated or already up to date).
- `1`: Runtime error.
- `2`: Usage error (unsupported host target).
- `4`: I/O error (unwritable path / permission denied).
- `5`: Network error (release API or asset download failed).
- `6`: Verification error (SHA256 integrity check failed).
