//! Pi native harness adapter implementation for Mario Zechner's pi coding agent.

use std::path::{Path, PathBuf};

use crate::harness::{HarnessAdapter, HarnessKind};

/// Harness adapter implementation for the `pi` coding agent (`~/.pi/agent`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PiAdapter;

impl HarnessAdapter for PiAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Pi
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        if home.file_name().and_then(|n| n.to_str()) == Some("skills") {
            return home.to_path_buf();
        }
        if home.file_name().and_then(|n| n.to_str()) == Some("agent") {
            return home.join("skills");
        }
        if home.file_name().and_then(|n| n.to_str()) == Some(".pi") {
            return home.join("agent").join("skills");
        }
        self.kind().harness_dir(home).join("skills")
    }
}

#[cfg(test)]
#[path = "tests/pi.rs"]
mod tests;
