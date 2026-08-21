//! Custom harness fallback mode adapter implementation (--harness custom).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::harness::{HarnessAdapter, HarnessKind};

/// Configuration state for custom fallback harnesses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomHarnessConfig {
    pub plugins_dir: PathBuf,
    pub skills_dir: PathBuf,
    pub rules_file: Option<PathBuf>,
}

#[derive(Debug)]
pub struct CustomAdapter {
    config: Option<CustomHarnessConfig>,
}

impl CustomAdapter {
    #[allow(dead_code)]
    pub fn new(config: Option<CustomHarnessConfig>) -> Self {
        Self { config }
    }
}

impl HarnessAdapter for CustomAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Custom
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if let Some(ref cfg) = self.config {
            if let Some(ref rules) = cfg.rules_file {
                return rules.clone();
            }
            return cfg.plugins_dir.clone();
        }
        home.join(".ce-ai").join("custom_harness.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_adapter_default_paths() {
        let home = PathBuf::from("/tmp/home");
        let adapter = CustomAdapter::new(None);
        assert_eq!(adapter.kind(), HarnessKind::Custom);
        assert_eq!(
            adapter.default_config_path(&home),
            home.join(".ce-ai/custom_harness.json")
        );

        let configured = CustomAdapter::new(Some(CustomHarnessConfig {
            plugins_dir: PathBuf::from("/custom/plugins"),
            skills_dir: PathBuf::from("/custom/skills"),
            rules_file: Some(PathBuf::from("/custom/rules.md")),
        }));
        assert_eq!(
            configured.default_config_path(&home),
            PathBuf::from("/custom/rules.md")
        );
    }
}
