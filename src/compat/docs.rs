use std::path::{Path, PathBuf};

/// Strongly-typed documentation configuration resolving dynamic documentation roots
/// defined in `.compound-engineering/config.yaml` or `.compound-engineering/config.local.yaml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CeDocsConfig {
    pub docs_root: PathBuf,
}

impl Default for CeDocsConfig {
    fn default() -> Self {
        Self {
            docs_root: PathBuf::from("docs"),
        }
    }
}

impl CeDocsConfig {
    /// Discovers the configured `docs_root` from the repository root.
    /// Checks `.compound-engineering/config.local.yaml` first, then `.compound-engineering/config.yaml`,
    /// defaulting to `docs` if absent or unspecified.
    pub fn discover(repo_root: &Path) -> Self {
        let local_config = repo_root.join(".compound-engineering/config.local.yaml");
        if local_config.exists() {
            if let Some(root) = Self::parse_docs_root(&local_config) {
                return Self { docs_root: root };
            }
        }

        let main_config = repo_root.join(".compound-engineering/config.yaml");
        if main_config.exists() {
            if let Some(root) = Self::parse_docs_root(&main_config) {
                return Self { docs_root: root };
            }
        }

        Self::default()
    }

    /// Returns the absolute path to the plans directory under the configured docs root.
    pub fn plans_dir(&self, repo_root: &Path) -> PathBuf {
        repo_root.join(&self.docs_root).join("plans")
    }

    /// Returns the absolute path to the solutions directory under the configured docs root.
    pub fn solutions_dir(&self, repo_root: &Path) -> PathBuf {
        repo_root.join(&self.docs_root).join("solutions")
    }

    /// Parses `docs_root:` from a given YAML configuration file.
    pub fn parse_docs_root(config_file: &Path) -> Option<PathBuf> {
        let content = std::fs::read_to_string(config_file).ok()?;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = trimmed.split_once(':') {
                if k.trim() == "docs_root" {
                    let val = v.trim().trim_matches('"').trim_matches('\'');
                    if !val.is_empty() {
                        return Some(PathBuf::from(val));
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_docs_config() {
        let config = CeDocsConfig::default();
        assert_eq!(config.docs_root, PathBuf::from("docs"));
        let root = Path::new("/tmp/test");
        assert_eq!(
            config.plans_dir(root),
            PathBuf::from("/tmp/test/docs/plans")
        );
        assert_eq!(
            config.solutions_dir(root),
            PathBuf::from("/tmp/test/docs/solutions")
        );
    }

    #[test]
    fn test_discover_custom_docs_root() {
        let temp = tempdir().unwrap();
        let ce_dir = temp.path().join(".compound-engineering");
        std::fs::create_dir_all(&ce_dir).unwrap();
        std::fs::write(ce_dir.join("config.yaml"), "docs_root: 'documentation'\n").unwrap();

        let config = CeDocsConfig::discover(temp.path());
        assert_eq!(config.docs_root, PathBuf::from("documentation"));
        assert_eq!(
            config.plans_dir(temp.path()),
            temp.path().join("documentation/plans")
        );
        assert_eq!(
            config.solutions_dir(temp.path()),
            temp.path().join("documentation/solutions")
        );
    }

    #[test]
    fn test_local_config_precedence() {
        let temp = tempdir().unwrap();
        let ce_dir = temp.path().join(".compound-engineering");
        std::fs::create_dir_all(&ce_dir).unwrap();
        std::fs::write(ce_dir.join("config.yaml"), "docs_root: 'shared_docs'\n").unwrap();
        std::fs::write(
            ce_dir.join("config.local.yaml"),
            "docs_root: 'local_docs'\n",
        )
        .unwrap();

        let config = CeDocsConfig::discover(temp.path());
        assert_eq!(config.docs_root, PathBuf::from("local_docs"));
    }
}
