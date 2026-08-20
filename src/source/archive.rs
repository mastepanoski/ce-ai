//! Safe tarball extraction (RED — tests only; implementation lands in task 3.3).
//!
//! Security contract (design threat note, task 3.1): absolute and `..` entry
//! paths MUST be rejected BEFORE any file is written (zip-slip).

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
        let archive =
            write_archive(dir.path(), "abs.tar", &tar_with(&[("safe.txt", "benign"), ("/etc/evil.txt", "evil")]));
        assert!(extract_safe(&archive, &dest).is_err());
        assert!(dir_has_no_files(&dest), "no entry may be written before rejection");
    }

    #[test]
    fn parent_traversal_entry_rejected_before_any_write() {
        let dir = tempdir().unwrap();
        let dest = dir.path().join("out");
        let archive =
            write_archive(dir.path(), "dotdot.tar", &tar_with(&[("safe.txt", "benign"), ("../evil.txt", "evil")]));
        assert!(extract_safe(&archive, &dest).is_err());
        assert!(dir_has_no_files(&dest), "no entry may be written before rejection");
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
        assert!(dir_has_no_files(&dest), "no entry may be written before rejection");
    }

    #[test]
    fn safe_archive_extracts_all_entries() {
        let dir = tempdir().unwrap();
        let dest = dir.path().join("out");
        let archive = write_archive(
            dir.path(),
            "safe.tar",
            &tar_with(&[("ce.js", "loader"), ("skills/ce-brainstorm/SKILL.md", "# skill")]),
        );
        extract_safe(&archive, &dest).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("ce.js")).unwrap(), "loader");
        assert_eq!(
            std::fs::read_to_string(dest.join("skills/ce-brainstorm/SKILL.md")).unwrap(),
            "# skill"
        );
    }
}