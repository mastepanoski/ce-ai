//! Built-in fallback skills embedded directly into the binary.
//!
//! Provides compile-time embedded assets for core skills (such as `sequential-thinking`)
//! ensuring they can be seeded and indexed even when upstream release tarballs
//! or local sources do not yet package them.

use std::path::{Path, PathBuf};

use crate::error::CeError;
use crate::opencode::manifest::ManifestFile;
use crate::state::diff::sha256_hex;
use crate::state::write_atomic;

/// Canonical embedded sequential-thinking skill markdown content.
pub const BUILTIN_SEQUENTIAL_THINKING_SKILL: &str =
    include_str!("../../skills/sequential-thinking/SKILL.md");

/// Relative path for sequential-thinking within managed trees (matching `managed_tree`).
pub const SEQUENTIAL_THINKING_REL_PATH: &str = "skills/sequential-thinking/SKILL.md";

/// Returns all built-in fallback skills as `(managed_relative_path, content)`.
pub fn all_builtin_skills() -> &'static [(&'static str, &'static str)] {
    &[(
        SEQUENTIAL_THINKING_REL_PATH,
        BUILTIN_SEQUENTIAL_THINKING_SKILL,
    )]
}

/// Target path for a built-in skill in a standard harness managed directory.
pub fn builtin_skill_target(managed_dir: &Path, rel_path: &str) -> PathBuf {
    managed_dir.join(rel_path)
}

/// Target path for a built-in skill in a custom harness skills directory.
pub fn custom_builtin_skill_target(skills_dir: &Path, rel_path: &str) -> PathBuf {
    let skill_subpath = rel_path.strip_prefix("skills/").unwrap_or(rel_path);
    skills_dir.join(skill_subpath)
}

/// Seeds missing built-in skills into a standard harness managed directory
/// (`<config_dir>/compound-engineering`).
///
/// If `!dry_run`, writes the embedded content atomically and returns the `ManifestFile` record.
pub fn seed_builtin_skill(
    managed_dir: &Path,
    rel_path: &str,
    content: &str,
    dry_run: bool,
) -> Result<ManifestFile, CeError> {
    let target = builtin_skill_target(managed_dir, rel_path);
    let bytes = content.as_bytes();
    let hash = sha256_hex(bytes);
    if !dry_run {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_atomic(&target, bytes)?;
    }
    Ok(ManifestFile {
        path: rel_path.to_string(),
        sha256: hash,
    })
}

/// Seeds missing built-in skills into a custom harness skills directory
/// (`<cfg.skills_dir>`).
pub fn seed_custom_builtin_skill(
    skills_dir: &Path,
    rel_path: &str,
    content: &str,
    dry_run: bool,
) -> Result<ManifestFile, CeError> {
    let target = custom_builtin_skill_target(skills_dir, rel_path);
    let bytes = content.as_bytes();
    let hash = sha256_hex(bytes);
    if !dry_run {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_atomic(&target, bytes)?;
    }
    Ok(ManifestFile {
        path: rel_path.to_string(),
        sha256: hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::registry::parse_skill_frontmatter;

    #[test]
    fn test_builtin_sequential_thinking_is_valid_and_non_empty() {
        assert!(!BUILTIN_SEQUENTIAL_THINKING_SKILL.is_empty());
        assert!(BUILTIN_SEQUENTIAL_THINKING_SKILL.contains("# Sequential Thinking Protocol"));

        let fm = parse_skill_frontmatter(BUILTIN_SEQUENTIAL_THINKING_SKILL);
        assert_eq!(fm.name, "sequential-thinking");
        assert_eq!(fm.scope, "global");
        assert!(!fm.description.is_empty());
        assert!(fm.triggers.contains(&"complex reasoning".to_string()));
        assert!(fm.triggers.contains(&"sequential thought".to_string()));
    }

    #[test]
    fn test_seed_builtin_skill_lifecycle_and_dry_run() {
        let temp = tempfile::tempdir().unwrap();
        let managed_dir = temp.path().join("compound-engineering");

        // Dry-run should return ManifestFile but write nothing
        let manifest = seed_builtin_skill(
            &managed_dir,
            SEQUENTIAL_THINKING_REL_PATH,
            BUILTIN_SEQUENTIAL_THINKING_SKILL,
            true,
        )
        .unwrap();
        assert_eq!(manifest.path, SEQUENTIAL_THINKING_REL_PATH);
        assert!(!managed_dir.join(SEQUENTIAL_THINKING_REL_PATH).exists());

        // Real write should create file atomically
        let manifest_real = seed_builtin_skill(
            &managed_dir,
            SEQUENTIAL_THINKING_REL_PATH,
            BUILTIN_SEQUENTIAL_THINKING_SKILL,
            false,
        )
        .unwrap();
        assert_eq!(manifest_real.path, SEQUENTIAL_THINKING_REL_PATH);
        assert_eq!(manifest_real.sha256, manifest.sha256);

        let target = managed_dir.join(SEQUENTIAL_THINKING_REL_PATH);
        assert!(target.exists());
        let read_back = std::fs::read_to_string(&target).unwrap();
        assert_eq!(read_back, BUILTIN_SEQUENTIAL_THINKING_SKILL);
    }

    #[test]
    fn test_seed_custom_builtin_skill_lifecycle() {
        let temp = tempfile::tempdir().unwrap();
        let skills_dir = temp.path().join("custom_skills");

        let manifest = seed_custom_builtin_skill(
            &skills_dir,
            SEQUENTIAL_THINKING_REL_PATH,
            BUILTIN_SEQUENTIAL_THINKING_SKILL,
            false,
        )
        .unwrap();
        assert_eq!(manifest.path, SEQUENTIAL_THINKING_REL_PATH);

        let target = skills_dir.join("sequential-thinking").join("SKILL.md");
        assert!(target.exists());
        let read_back = std::fs::read_to_string(&target).unwrap();
        assert_eq!(read_back, BUILTIN_SEQUENTIAL_THINKING_SKILL);
    }
}
