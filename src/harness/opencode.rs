//! OpenCode harness adapter implementation.

use std::path::{Path, PathBuf};

use crate::harness::{HarnessAdapter, HarnessKind};

#[derive(Debug, Default)]
pub struct OpencodeAdapter;

impl HarnessAdapter for OpencodeAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Opencode
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if let Ok(path) = std::env::var("CE_AI_OPENCODE_CONFIG") {
            return PathBuf::from(path).join("opencode.json");
        }
        home.join(".config").join("opencode").join("opencode.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opencode_adapter_default_paths() {
        let adapter = OpencodeAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Opencode);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.config/opencode/opencode.json")
        );
    }
}
