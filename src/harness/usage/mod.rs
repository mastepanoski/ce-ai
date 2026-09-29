//! Usage source adapters per harness.

use std::path::Path;

use crate::capture::ledger::UsageRecord;
use crate::error::CeError;

pub mod claude;

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
    vec![Box::new(claude::ClaudeUsageAdapter)]
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
    fn test_all_adapters_contains_claude() {
        let adapters = all_adapters();
        assert!(!adapters.is_empty());
        assert_eq!(adapters[0].harness_name(), "claude");
    }

    #[test]
    fn test_get_adapter_resolves_case_insensitively() {
        assert!(get_adapter("claude").is_some());
        assert!(get_adapter("Claude").is_some());
        assert!(get_adapter("CLAUDE").is_some());
        assert!(get_adapter("nonexistent").is_none());
    }
}
