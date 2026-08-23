//! Safe tarball extraction — rejects absolute and `..` entry paths BEFORE any
//! file is written (zip-slip; design threat note, tasks 3.1/3.3).

use std::io::Read;
use std::path::{Component, Path};

use crate::error::CeError;

/// Recursively copy a directory from `src` to `dst`.
pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

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
mod tests {
    use std::path::Path;

    use tempfile::tempdir;

    use crate::source::archive::extract_safe;

    /// Appends an entry whose name bytes are written verbatim into the tar
    /// header, bypassing `tar::Header::set_path` validation so tests can craft
    /// malicious fixtures (absolute / `..` paths).
    fn append_raw_entry(builder: &mut tar::Builder<Vec<u8>>, raw_path: &[u8], data: &[u8]) {
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o644);
        header.set_size(data.len() as u64);
        let name = &mut header.as_old_mut().name;
        name[..raw_path.len()].copy_from_slice(raw_path);
        header.set_cksum();
        builder.append(&header, data).unwrap();
    }

    /// Builds a plain tar with the given `(raw_path, content)` entries.
    fn tar_with(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut builder = tar::Builder::new(Vec::new());
        for (path, content) in entries {
            append_raw_entry(&mut builder, path.as_bytes(), content.as_bytes());
        }
        builder.finish().unwrap();
        builder.into_inner().unwrap()
    }

    fn write_archive(dir: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    /// True when `dir` was never created or holds no entries — proves that no
    /// file was written during a rejected extraction.
    fn dir_has_no_files(dir: &Path) -> bool {
        !dir.exists() || std::fs::read_dir(dir).unwrap().next().is_none()
    }

    #[test]
    fn absolute_path_entry_rejected_before_any_write() {
        let dir = tempdir().unwrap();
        let dest = dir.path().join("out");
        let archive = write_archive(
            dir.path(),
            "abs.tar",
            &tar_with(&[("safe.txt", "benign"), ("/etc/evil.txt", "evil")]),
        );
        assert!(extract_safe(&archive, &dest).is_err());
        assert!(
            dir_has_no_files(&dest),
            "no entry may be written before rejection"
        );
    }

    #[test]
    fn parent_traversal_entry_rejected_before_any_write() {
        let dir = tempdir().unwrap();
        let dest = dir.path().join("out");
        let archive = write_archive(
            dir.path(),
            "dotdot.tar",
            &tar_with(&[("safe.txt", "benign"), ("../evil.txt", "evil")]),
        );
        assert!(extract_safe(&archive, &dest).is_err());
        assert!(
            dir_has_no_files(&dest),
            "no entry may be written before rejection"
        );
    }

    #[test]
    fn nested_parent_traversal_rejected_before_any_write() {
        let dir = tempdir().unwrap();
        let dest = dir.path().join("out");
        let archive = write_archive(
            dir.path(),
            "nested.tar",
            &tar_with(&[("safe.txt", "benign"), ("a/../../evil.txt", "evil")]),
        );
        assert!(extract_safe(&archive, &dest).is_err());
        assert!(
            dir_has_no_files(&dest),
            "no entry may be written before rejection"
        );
    }

    #[test]
    fn safe_archive_extracts_all_entries() {
        let dir = tempdir().unwrap();
        let dest = dir.path().join("out");
        let archive = write_archive(
            dir.path(),
            "safe.tar",
            &tar_with(&[
                ("ce.js", "loader"),
                ("skills/ce-brainstorm/SKILL.md", "# skill"),
            ]),
        );
        extract_safe(&archive, &dest).unwrap();
        assert_eq!(
            std::fs::read_to_string(dest.join("ce.js")).unwrap(),
            "loader"
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("skills/ce-brainstorm/SKILL.md")).unwrap(),
            "# skill"
        );
    }
}
