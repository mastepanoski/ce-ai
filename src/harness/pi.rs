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

    fn canonical_instruction_file(&self) -> PathBuf {
        PathBuf::from("AGENTS.md")
    }

    fn derived_stub_files(&self) -> Vec<PathBuf> {
        vec![PathBuf::from(".pi").join("AGENTS.md")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::harness::tests::HARNESS_ENV_LOCK;

    #[test]
    fn pi_adapter_default_paths() {
        let _guard = HARNESS_ENV_LOCK.lock().unwrap();
        std::env::remove_var("PI_CODING_AGENT_DIR");

        let adapter = PiAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Pi);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.kind().harness_dir(&home),
            PathBuf::from("/tmp/home/.pi/agent")
        );
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.pi/agent/skills")
        );
        assert_eq!(
            adapter.canonical_instruction_file(),
            PathBuf::from("AGENTS.md")
        );
        assert_eq!(
            adapter.derived_stub_files(),
            vec![PathBuf::from(".pi/AGENTS.md")]
        );
    }

    #[test]
    fn pi_adapter_respects_pi_coding_agent_dir_env() {
        let _guard = HARNESS_ENV_LOCK.lock().unwrap();
        std::env::set_var("PI_CODING_AGENT_DIR", "/custom/pi/dir");

        let adapter = PiAdapter;
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.kind().harness_dir(&home),
            PathBuf::from("/custom/pi/dir")
        );
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/custom/pi/dir/skills")
        );

        std::env::remove_var("PI_CODING_AGENT_DIR");
    }
}
