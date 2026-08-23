//! Generic JSON harness adapter implementation for Grok, Kimi, AGY, DeepSeek, FX.

use std::path::{Path, PathBuf};

use crate::harness::{HarnessAdapter, HarnessKind};

#[derive(Debug)]
pub struct GenericJsonAdapter {
    kind: HarnessKind,
}

#[allow(dead_code)]
impl GenericJsonAdapter {
    pub fn new(kind: HarnessKind) -> Self {
        Self { kind }
    }
}

impl HarnessAdapter for GenericJsonAdapter {
    fn kind(&self) -> HarnessKind {
        self.kind
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        match self.kind {
            HarnessKind::Grok => home.join(".grok").join("config.json"),
            HarnessKind::Kimi => home.join(".kimi").join("config.json"),
            HarnessKind::Agy => home
                .join(".gemini")
                .join("antigravity-cli")
                .join("config.json"),
            HarnessKind::Deepseek => home.join(".deepseek").join("config.json"),
            HarnessKind::Fx => home.join(".fx").join("config.json"),
            _ => home.join(format!(".{}", self.kind)).join("config.json"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_json_adapter_paths() {
        let home = PathBuf::from("/tmp/home");

        let agy = GenericJsonAdapter::new(HarnessKind::Agy);
        assert_eq!(
            agy.default_config_path(&home),
            home.join(".gemini/antigravity-cli/config.json")
        );

        let deepseek = GenericJsonAdapter::new(HarnessKind::Deepseek);
        assert_eq!(
            deepseek.default_config_path(&home),
            home.join(".deepseek/config.json")
        );
    }
}
