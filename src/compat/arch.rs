//! Typed compatibility and resolution layer for architecture directories and assets.
//!
//! Provides canonical path resolution preferring modern lean architecture under `docs/arch/`
//! (`system/`, `adr/`, `contracts/`, `archive/`, `deltas/`) while providing graceful,
//! transparent fallback to legacy `openspec/` directories for unmigrated host repositories.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CeError;

/// Declarative multi-repo constellation manifest (`docs/arch/system.yaml`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemManifest {
    pub system: String,
    pub component: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<SystemDependency>,
}

/// A declared inter-component dependency within a constellation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemDependency {
    pub component: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub remote_repo: Option<String>,
    #[serde(default)]
    pub contract: Option<String>,
}

/// Status of inter-component dependency resolution in a multi-repo constellation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyResolutionStatus {
    Local(PathBuf),
    RemoteCached(PathBuf),
    RemoteMissing,
}

impl DependencyResolutionStatus {
    pub fn display_label(&self) -> String {
        match self {
            Self::Local(p) => format!("(local: {})", p.display()),
            Self::RemoteCached(_) => "(remote-contract: cached)".to_string(),
            Self::RemoteMissing => "(remote-contract: missing)".to_string(),
        }
    }
}

/// Validates that a remote repository URL does not contain embedded credentials (OWASP LLM-02).
pub fn validate_remote_repo_url(url: &str) -> Result<(), CeError> {
    let lower = url.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        if let Some(rest) = url.split("://").nth(1) {
            let authority = rest.split('/').next().unwrap_or(rest);
            if authority.contains('@') {
                return Err(CeError::Usage(format!(
                    "remote_repo URL '{url}' contains embedded credentials, which is forbidden by security policy (OWASP LLM-02)"
                )));
            }
        }
    } else if lower.starts_with("ssh://") {
        if let Some(rest) = url.split("://").nth(1) {
            let authority = rest.split('/').next().unwrap_or(rest);
            if let Some(userinfo) = authority.split('@').next() {
                if userinfo.contains(':') {
                    return Err(CeError::Usage(format!(
                        "remote_repo URL '{url}' contains embedded credentials/password, which is forbidden by security policy (OWASP LLM-02)"
                    )));
                }
            }
        }
    }
    Ok(())
}

/// Parses raw `system.yaml` content into a `SystemManifest`.
pub fn parse_system_manifest(content: &str) -> Result<SystemManifest, CeError> {
    let mut system = None;
    let mut component = None;
    let mut role = None;
    let mut dependencies = Vec::new();

    let mut in_dependencies = false;
    let mut current_dep: Option<SystemDependency> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed == "dependencies:" {
            in_dependencies = true;
            if let Some(dep) = current_dep.take() {
                dependencies.push(dep);
            }
            continue;
        }

        if in_dependencies {
            let is_item_start = trimmed.starts_with("- ") || trimmed == "-";
            if is_item_start {
                if let Some(dep) = current_dep.take() {
                    dependencies.push(dep);
                }
                let mut dep = SystemDependency {
                    component: String::new(),
                    role: None,
                    path: None,
                    remote_repo: None,
                    contract: None,
                };
                if let Some(stripped) = trimmed.strip_prefix("- ") {
                    let rest = stripped.trim();
                    if let Some((k, v)) = rest.split_once(':') {
                        let k = k.trim();
                        let v = v.trim().trim_matches('"').trim_matches('\'');
                        match k {
                            "component" => dep.component = v.to_string(),
                            "role" => dep.role = Some(v.to_string()),
                            "path" => dep.path = Some(PathBuf::from(v)),
                            "remote_repo" => dep.remote_repo = Some(v.to_string()),
                            "contract" => dep.contract = Some(v.to_string()),
                            _ => {}
                        }
                    }
                }
                current_dep = Some(dep);
                continue;
            }

            // Sub-fields of current dependency
            let is_indented = line.starts_with("  ") || line.starts_with('\t');
            if is_indented {
                if let Some((k, v)) = trimmed.split_once(':') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'');
                    if let Some(ref mut dep) = current_dep {
                        match k {
                            "component" => dep.component = v.to_string(),
                            "role" => dep.role = Some(v.to_string()),
                            "path" => dep.path = Some(PathBuf::from(v)),
                            "remote_repo" => dep.remote_repo = Some(v.to_string()),
                            "contract" => dep.contract = Some(v.to_string()),
                            _ => {}
                        }
                    }
                }
                continue;
            } else {
                in_dependencies = false;
                if let Some(dep) = current_dep.take() {
                    dependencies.push(dep);
                }
            }
        }

        if !in_dependencies {
            if let Some((k, v)) = trimmed.split_once(':') {
                let k = k.trim();
                let v = v.trim().trim_matches('"').trim_matches('\'');
                match k {
                    "system" => system = Some(v.to_string()),
                    "component" => component = Some(v.to_string()),
                    "role" => role = Some(v.to_string()),
                    _ => {}
                }
            }
        }
    }

    if let Some(dep) = current_dep {
        dependencies.push(dep);
    }

    let system = system
        .ok_or_else(|| CeError::Usage("system.yaml missing required field 'system'".to_string()))?;
    let component = component.ok_or_else(|| {
        CeError::Usage("system.yaml missing required field 'component'".to_string())
    })?;

    for dep in &dependencies {
        if dep.component.is_empty() {
            return Err(CeError::Usage(
                "dependency entry in system.yaml missing required field 'component'".to_string(),
            ));
        }
        if let Some(ref url) = dep.remote_repo {
            validate_remote_repo_url(url)?;
        }
    }

    Ok(SystemManifest {
        system,
        component,
        role,
        dependencies,
    })
}

/// Loads and validates `docs/arch/system.yaml` if present in the repository root.
pub fn load_system_manifest(repo_root: &Path) -> Result<Option<SystemManifest>, CeError> {
    let path = system_manifest_path(repo_root);
    if !path.is_file() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)?;
    let manifest = parse_system_manifest(&content)?;
    Ok(Some(manifest))
}

/// Resolves dependency status across local filesystem and cached contracts in `docs/arch/contracts/`.
pub fn resolve_dependency_status(
    repo_root: &Path,
    dep: &SystemDependency,
) -> DependencyResolutionStatus {
    if let Some(ref rel_path) = dep.path {
        let abs_path = repo_root.join(rel_path);
        if abs_path.is_dir() {
            return DependencyResolutionStatus::Local(rel_path.clone());
        }
    }

    let contracts = contracts_dir(repo_root);
    if let Some(ref contract_rel) = dep.contract {
        let direct_contract = contracts.join(contract_rel);
        if direct_contract.is_file() {
            return DependencyResolutionStatus::RemoteCached(direct_contract);
        }
    }

    for ext in &["json", "yaml", "yml", "md"] {
        let candidate = contracts.join(format!("{}.{}", dep.component, ext));
        if candidate.is_file() {
            return DependencyResolutionStatus::RemoteCached(candidate);
        }
    }

    DependencyResolutionStatus::RemoteMissing
}

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
    if legacy.is_dir() || repo_root.join("openspec").is_dir() {
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
    if legacy.is_dir() || repo_root.join("openspec").is_dir() {
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

    #[test]
    fn test_parse_system_manifest_valid() {
        let yaml = r#"
system: payments-platform
component: billing-service
role: service
dependencies:
  - component: auth-service
    role: service
    path: ../auth-service
    remote_repo: https://github.com/org/auth-service
    contract: auth.json
  - component: notifications-service
    role: worker
    remote_repo: git@github.com:org/notifications.git
"#;
        let manifest = parse_system_manifest(yaml).expect("should parse valid manifest");
        assert_eq!(manifest.system, "payments-platform");
        assert_eq!(manifest.component, "billing-service");
        assert_eq!(manifest.role, Some("service".to_string()));
        assert_eq!(manifest.dependencies.len(), 2);

        let dep0 = &manifest.dependencies[0];
        assert_eq!(dep0.component, "auth-service");
        assert_eq!(dep0.role, Some("service".to_string()));
        assert_eq!(dep0.path, Some(PathBuf::from("../auth-service")));
        assert_eq!(
            dep0.remote_repo,
            Some("https://github.com/org/auth-service".to_string())
        );
        assert_eq!(dep0.contract, Some("auth.json".to_string()));

        let dep1 = &manifest.dependencies[1];
        assert_eq!(dep1.component, "notifications-service");
        assert_eq!(dep1.role, Some("worker".to_string()));
    }

    #[test]
    fn test_parse_system_manifest_rejects_embedded_credentials() {
        let yaml = r#"
system: payments-platform
component: billing-service
dependencies:
  - component: auth-service
    remote_repo: https://user:secrettoken123@github.com/org/auth-service
"#;
        let err = parse_system_manifest(yaml).unwrap_err();
        assert!(err
            .to_string()
            .contains("contains embedded credentials, which is forbidden"));
    }

    #[test]
    fn test_validate_remote_repo_url_variations() {
        assert!(validate_remote_repo_url("https://github.com/org/repo").is_ok());
        assert!(validate_remote_repo_url("git@github.com:org/repo.git").is_ok());
        assert!(validate_remote_repo_url("ssh://git@github.com/org/repo.git").is_ok());

        assert!(validate_remote_repo_url("https://token@github.com/org/repo").is_err());
        assert!(validate_remote_repo_url("https://user:pass@github.com/org/repo").is_err());
        assert!(validate_remote_repo_url("ssh://user:pass@github.com/org/repo").is_err());
    }

    #[test]
    fn test_resolve_dependency_status_local_and_cached() {
        let temp = tempdir().unwrap();
        let root = temp.path();

        // 1. Dependency with local directory existing
        let local_dep_dir = root.join("auth-service");
        fs::create_dir_all(&local_dep_dir).unwrap();
        let dep_local = SystemDependency {
            component: "auth-service".to_string(),
            role: None,
            path: Some(PathBuf::from("auth-service")),
            remote_repo: None,
            contract: None,
        };
        let status1 = resolve_dependency_status(root, &dep_local);
        assert_eq!(
            status1,
            DependencyResolutionStatus::Local(PathBuf::from("auth-service"))
        );

        // 2. Dependency without local dir, but with cached contract
        let contracts_dir = root.join("docs").join("arch").join("contracts");
        fs::create_dir_all(&contracts_dir).unwrap();
        fs::write(contracts_dir.join("notifications.json"), "{}").unwrap();
        let dep_cached = SystemDependency {
            component: "notifications".to_string(),
            role: None,
            path: Some(PathBuf::from("../notifications")),
            remote_repo: None,
            contract: None,
        };
        let status2 = resolve_dependency_status(root, &dep_cached);
        assert_eq!(
            status2,
            DependencyResolutionStatus::RemoteCached(contracts_dir.join("notifications.json"))
        );

        // 3. Dependency without local dir and without cached contract
        let dep_missing = SystemDependency {
            component: "analytics".to_string(),
            role: None,
            path: None,
            remote_repo: None,
            contract: None,
        };
        let status3 = resolve_dependency_status(root, &dep_missing);
        assert_eq!(status3, DependencyResolutionStatus::RemoteMissing);
    }
}
