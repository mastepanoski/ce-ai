//! Safe tarball extraction — rejects absolute and `..` entry paths BEFORE any
//! file is written (zip-slip; design threat note, tasks 3.1/3.3).

use std::io::Read;
use std::path::{Component, Path};

use crate::error::CeError;

/// True when `path` is safe to extract: relative, no parent (`..`)
/// components, and no Windows drive-letter prefix.
fn is_safe_relative_path(path: &Path) -> bool {
    if path.is_absolute() {
        return false;
    }
    let raw = path.to_string_lossy();
    if raw.starts_with('/') || raw.starts_with('\\') {
        return false;
    }
    let bytes = raw.as_bytes();
    if bytes.len() >= 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes.len() == 2 || bytes[2] == b'/' || bytes[2] == b'\\')
    {
        return false; // drive-letter absolute, e.g. `C:\x` or `C:/x`
    }
    !path.components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    })
}

/// Opens archive bytes as a tar reader, transparently handling gzip (GitHub
/// `*.tar.gz`) and plain tar (tests) via gzip magic bytes.
fn tar_reader(bytes: &[u8]) -> Box<dyn Read + '_> {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        Box::new(flate2::read::GzDecoder::new(bytes))
    } else {
        Box::new(bytes)
    }
}

/// Validates every entry path in the archive without writing anything.
fn validate_all_paths(bytes: &[u8]) -> Result<(), CeError> {
    let mut archive = tar::Archive::new(tar_reader(bytes));
    for entry in archive
        .entries()
        .map_err(|e| CeError::Runtime(format!("tar error: {e}")))?
    {
        let entry = entry.map_err(|e| CeError::Runtime(format!("tar error: {e}")))?;
        let path = entry.path()?.into_owned();
        if !is_safe_relative_path(&path) {
            return Err(CeError::Runtime(format!(
                "unsafe archive entry path: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

/// Extracts a (possibly gzipped) tar archive into `dest`. All entry paths are
/// validated first; the archive is rejected before ANY file is written when an
/// absolute or `..` path is present. Safe entries are extracted; directory
/// entries create directories; symlinks/specials are never materialized.
pub fn extract_safe(archive: &Path, dest: &Path) -> Result<(), CeError> {
    let bytes = std::fs::read(archive)?;
    validate_all_paths(&bytes)?; // reject-before-any-write security gate
    std::fs::create_dir_all(dest)?;
    let mut archive = tar::Archive::new(tar_reader(&bytes));
    for entry in archive
        .entries()
        .map_err(|e| CeError::Runtime(format!("tar error: {e}")))?
    {
        let mut entry = entry.map_err(|e| CeError::Runtime(format!("tar error: {e}")))?;
        let entry_path = entry.path()?.into_owned();
        if !is_safe_relative_path(&entry_path) {
            return Err(CeError::Runtime(format!(
                "unsafe archive entry path: {}",
                entry_path.display()
            )));
        }
        let target = dest.join(&entry_path);
        match entry.header().entry_type() {
            tar::EntryType::Directory => std::fs::create_dir_all(&target)?,
            tar::EntryType::Regular => {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut out = std::fs::File::create(&target)?;
                std::io::copy(&mut entry, &mut out)?;
            }
            _ => {} // symlinks and specials are never written
        }
    }
    Ok(())
}

/// Real runs persist the extracted tree under `<config-dir>/cache/trees/<tag>`
/// so later `sync` runs resolve it from the manifest; dry-runs extract to a
/// system temp dir that is removed afterwards.
pub fn extract_to_source(
    config_dir: &Path,
    dry_run: bool,
    tarball: &Path,
    tag: &str,
) -> Result<(std::path::PathBuf, Option<std::path::PathBuf>), CeError> {
    let safe_tag: String = tag
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect();
    let dest = if dry_run {
        std::env::temp_dir().join(format!("ce-ai-extracted-{}", std::process::id()))
    } else {
        config_dir.join("cache/trees").join(&safe_tag)
    };
    let _ = std::fs::remove_dir_all(&dest);
    extract_safe(tarball, &dest)?;
    let root = find_source_root(&dest)?;
    let tmp = if dry_run { Some(dest) } else { None };
    Ok((root, tmp))
}

/// GitHub tarballs nest the tree under a `<repo>-<ref>/` top dir; the source
/// root is the extracted dir itself when it holds `.opencode`, else its single
/// subdirectory.
pub fn find_source_root(dir: &Path) -> Result<std::path::PathBuf, CeError> {
    if dir.join(".opencode").is_dir() {
        return Ok(dir.to_path_buf());
    }
    let mut dirs: Vec<std::path::PathBuf> = std::fs::read_dir(dir)?
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

#[cfg(test)]
#[path = "tests/archive.rs"]
mod tests;
