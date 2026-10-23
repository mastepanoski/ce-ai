//! Typed compatibility and resolution layer for architecture directories and assets.
//!
//! Provides canonical path resolution preferring modern lean architecture under `docs/arch/`
//! (`system/`, `adr/`, `contracts/`, `archive/`, `deltas/`) while providing graceful,
//! transparent fallback to legacy `openspec/` directories for unmigrated host repositories.

use std::path::{Path, PathBuf};

/// Origin source of an architecture asset directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchSource {
    /// Modern lean structure under `docs/arch/`.
    DocsArch,
    /// Legacy SDD structure under `openspec/`.
    OpenSpecLegacy,
    /// Default modern path (neither directory exists physically on disk yet).
    DefaultModern,
}

/// Returns the root directory for architecture assets: `<repo_root>/docs/arch`.
pub fn arch_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("docs").join("arch")
}

/// Returns the canonical directory for living domain specifications (`<repo_root>/docs/arch/system`).
///
/// If `<repo_root>/docs/arch/system` exists as a directory, it is returned.
/// If not, but legacy `<repo_root>/openspec/specs` exists as a directory, the legacy path is returned.
/// If the project has an existing legacy `<repo_root>/openspec` directory, returns `<repo_root>/openspec/specs`.
/// If neither exists, returns the modern `<repo_root>/docs/arch/system` by default.
pub fn system_specs_dir(repo_root: &Path) -> PathBuf {
    let modern = arch_dir(repo_root).join("system");
    if modern.is_dir() {
        return modern;
    }

    let legacy = repo_root.join("openspec").join("specs");
    if legacy.is_dir() || repo_root.join("openspec").is_dir() {
        return legacy;
    }

    modern
}

/// Detects the source of living domain specifications.
pub fn detect_system_specs_source(repo_root: &Path) -> ArchSource {
    let modern = arch_dir(repo_root).join("system");
    if modern.is_dir() {
        return ArchSource::DocsArch;
    }

    let legacy = repo_root.join("openspec").join("specs");
    if legacy.is_dir() || repo_root.join("openspec").is_dir() {
        return ArchSource::OpenSpecLegacy;
    }

    ArchSource::DefaultModern
}

/// Returns the directory for historical change archives.
///
/// Prefers `<repo_root>/docs/arch/archive`.
/// Falls back to `<repo_root>/openspec/changes/archive` if only the legacy directory exists.
pub fn archive_dir(repo_root: &Path) -> PathBuf {
    let modern = arch_dir(repo_root).join("archive");
    if modern.is_dir() {
        return modern;
    }

    let legacy = repo_root.join("openspec").join("changes").join("archive");
    if legacy.is_dir() {
        return legacy;
    }

    modern
}

/// Returns the directory for transient delta changes.
///
/// Prefers `<repo_root>/docs/arch/deltas`.
/// Falls back to `<repo_root>/openspec/changes` if only the legacy directory exists.
pub fn deltas_dir(repo_root: &Path) -> PathBuf {
    let modern = arch_dir(repo_root).join("deltas");
    if modern.is_dir() {
        return modern;
    }

    let legacy = repo_root.join("openspec").join("changes");
    if legacy.is_dir() {
        return legacy;
    }

    modern
}

/// Returns the canonical directory for Architecture Decision Records (ADRs): `<repo_root>/docs/arch/adr`.
pub fn adrs_dir(repo_root: &Path) -> PathBuf {
    arch_dir(repo_root).join("adr")
}

/// Returns the canonical directory for cached multi-repo contracts: `<repo_root>/docs/arch/contracts`.
pub fn contracts_dir(repo_root: &Path) -> PathBuf {
    arch_dir(repo_root).join("contracts")
}

/// Returns the path to the repository constellation manifest: `<repo_root>/docs/arch/system.yaml`.
pub fn system_manifest_path(repo_root: &Path) -> PathBuf {
    arch_dir(repo_root).join("system.yaml")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_system_specs_dir_prefers_docs_arch_system() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        let modern = root.join("docs").join("arch").join("system");
        let legacy = root.join("openspec").join("specs");

        fs::create_dir_all(&modern).unwrap();
        fs::create_dir_all(&legacy).unwrap();

        assert_eq!(system_specs_dir(root), modern);
        assert_eq!(detect_system_specs_source(root), ArchSource::DocsArch);
    }

    #[test]
    fn test_system_specs_dir_fallbacks_to_openspec() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        let legacy = root.join("openspec").join("specs");
        fs::create_dir_all(&legacy).unwrap();

        assert_eq!(system_specs_dir(root), legacy);
        assert_eq!(detect_system_specs_source(root), ArchSource::OpenSpecLegacy);
    }

    #[test]
    fn test_system_specs_dir_infers_openspec_when_openspec_root_exists() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        let legacy_changes = root.join("openspec").join("changes");
        fs::create_dir_all(&legacy_changes).unwrap();

        assert_eq!(system_specs_dir(root), root.join("openspec").join("specs"));
        assert_eq!(detect_system_specs_source(root), ArchSource::OpenSpecLegacy);
    }

    #[test]
    fn test_system_specs_dir_defaults_to_docs_arch_system_when_neither() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        let modern = root.join("docs").join("arch").join("system");
        assert_eq!(system_specs_dir(root), modern);
        assert_eq!(detect_system_specs_source(root), ArchSource::DefaultModern);
    }

    #[test]
    fn test_archive_dir_dual_path() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        let modern = root.join("docs").join("arch").join("archive");
        let legacy = root.join("openspec").join("changes").join("archive");

        // Neither exists -> default modern
        assert_eq!(archive_dir(root), modern);

        // Only legacy exists -> returns legacy
        fs::create_dir_all(&legacy).unwrap();
        assert_eq!(archive_dir(root), legacy);

        // Both exist -> prefers modern
        fs::create_dir_all(&modern).unwrap();
        assert_eq!(archive_dir(root), modern);
    }

    #[test]
    fn test_deltas_dir_dual_path() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        let modern = root.join("docs").join("arch").join("deltas");
        let legacy = root.join("openspec").join("changes");

        assert_eq!(deltas_dir(root), modern);

        fs::create_dir_all(&legacy).unwrap();
        assert_eq!(deltas_dir(root), legacy);

        fs::create_dir_all(&modern).unwrap();
        assert_eq!(deltas_dir(root), modern);
    }

    #[test]
    fn test_fixed_paths() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        assert_eq!(arch_dir(root), root.join("docs").join("arch"));
        assert_eq!(adrs_dir(root), root.join("docs").join("arch").join("adr"));
        assert_eq!(
            contracts_dir(root),
            root.join("docs").join("arch").join("contracts")
        );
        assert_eq!(
            system_manifest_path(root),
            root.join("docs").join("arch").join("system.yaml")
        );
    }
}
