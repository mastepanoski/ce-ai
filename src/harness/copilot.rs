//! GitHub Copilot harness Markdown block adapter implementation.

use std::path::{Path, PathBuf};

use crate::harness::{HarnessAdapter, HarnessKind};

#[derive(Debug, Default)]
pub struct CopilotAdapter;

impl HarnessAdapter for CopilotAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Copilot
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        home.join(".github").join("copilot-instructions.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copilot_adapter_default_paths() {
        let adapter = CopilotAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Copilot);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.github/copilot-instructions.md")
        );
    }
}
