//! Pi harness adapter implementation.

use std::path::{Path, PathBuf};

use crate::harness::{HarnessAdapter, HarnessKind};

#[derive(Debug, Default)]
pub struct PiAdapter;

impl HarnessAdapter for PiAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Pi
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        home.join(".pi").join("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pi_adapter_default_paths() {
        let adapter = PiAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Pi);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.pi/config.json")
        );
    }
}
