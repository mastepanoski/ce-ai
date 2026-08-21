//! Claude Code harness adapter implementation.

use std::path::{Path, PathBuf};

use crate::harness::{HarnessAdapter, HarnessKind};

#[derive(Debug, Default)]
pub struct ClaudeAdapter;

impl HarnessAdapter for ClaudeAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Claude
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        home.join(".claude.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_adapter_default_paths() {
        let adapter = ClaudeAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Claude);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.claude.json")
        );
    }
}
