//! Usage source adapters per harness.

use std::path::Path;

use crate::capture::ledger::UsageRecord;
use crate::error::CeError;

pub mod claude;
pub mod codex;
pub mod opencode;
pub mod pi;

/// Common interface for ingesting local usage metrics from AI coding harnesses.
pub trait UsageAdapter: Send + Sync {
    /// Canonical harness name matching `HarnessKind` (e.g. "claude", "opencode", "codex", "pi").
    fn harness_name(&self) -> &'static str;

    /// Checks whether local transcripts for this harness exist on disk.
    fn is_available(&self, home: &Path) -> bool;

    /// Discovers and parses all session transcripts, extracting normalized `UsageRecord`s.
    fn read_usage(
        &self,
        home: &Path,
        author: &str,
        since: Option<&str>,
        cwd_filter: Option<&str>,
    ) -> Result<Vec<UsageRecord>, CeError>;
}

/// Returns all registered usage adapters.
pub fn all_adapters() -> Vec<Box<dyn UsageAdapter>> {
    vec![
        Box::new(claude::ClaudeUsageAdapter),
        Box::new(opencode::OpenCodeUsageAdapter),
        Box::new(codex::CodexUsageAdapter),
        Box::new(pi::PiUsageAdapter),
    ]
}

/// Resolves an adapter by its harness name.
pub fn get_adapter(name: &str) -> Option<Box<dyn UsageAdapter>> {
    let lower = name.to_lowercase();
    all_adapters()
        .into_iter()
        .find(|a| a.harness_name() == lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_adapters_contains_registered_adapters() {
        let adapters = all_adapters();
        assert_eq!(adapters.len(), 4);
        assert_eq!(adapters[0].harness_name(), "claude");
        assert_eq!(adapters[1].harness_name(), "opencode");
        assert_eq!(adapters[2].harness_name(), "codex");
        assert_eq!(adapters[3].harness_name(), "pi");
    }

    #[test]
    fn test_get_adapter_resolves_case_insensitively() {
        assert!(get_adapter("claude").is_some());
        assert!(get_adapter("Claude").is_some());
        assert!(get_adapter("CLAUDE").is_some());
        assert!(get_adapter("opencode").is_some());
        assert!(get_adapter("OpenCode").is_some());
        assert!(get_adapter("codex").is_some());
        assert!(get_adapter("CODEX").is_some());
        assert!(get_adapter("pi").is_some());
        assert!(get_adapter("PI").is_some());
        assert!(get_adapter("nonexistent").is_none());
    }
}
