//! Generic JSON harness adapter implementation for Custom.

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
        home.join(format!(".{}", self.kind)).join("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn generic_json_adapter_paths_and_kinds() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().to_path_buf();

        let custom = GenericJsonAdapter::new(HarnessKind::Custom);
        assert_eq!(custom.kind(), HarnessKind::Custom);
        assert_eq!(
            custom.default_config_path(&home),
            home.join(".custom/config.json")
        );
    }
}
