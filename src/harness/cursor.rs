//! Cursor harness Markdown block adapter implementation.

use std::path::{Path, PathBuf};

use crate::harness::{HarnessAdapter, HarnessKind};

pub const CE_MANAGED_BEGIN: &str = "<!-- CE-AI MANAGED BLOCK BEGIN -->";
pub const CE_MANAGED_END: &str = "<!-- CE-AI MANAGED BLOCK END -->";

#[derive(Debug, Default)]
pub struct CursorAdapter;

impl HarnessAdapter for CursorAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Cursor
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        home.join(".cursorrules")
    }
}

/// Inject or replace demarcated managed comment block in markdown instruction file.
#[allow(dead_code)]
pub fn update_managed_block(content: &str, managed_text: &str) -> String {
    let block = format!(
        "{}\n{}\n{}",
        CE_MANAGED_BEGIN,
        managed_text.trim(),
        CE_MANAGED_END
    );

    if let (Some(start), Some(end)) = (content.find(CE_MANAGED_BEGIN), content.find(CE_MANAGED_END))
    {
        let before = &content[..start];
        let after = &content[end + CE_MANAGED_END.len()..];
        format!("{}{}{}", before.trim_end(), block, after)
    } else if content.trim().is_empty() {
        block
    } else {
        format!("{}\n\n{}", content.trim_end(), block)
    }
}

/// Strip demarcated managed comment block on uninstallation.
#[allow(dead_code)]
pub fn strip_managed_block(content: &str) -> String {
    if let (Some(start), Some(end)) = (content.find(CE_MANAGED_BEGIN), content.find(CE_MANAGED_END))
    {
        let before = &content[..start];
        let after = &content[end + CE_MANAGED_END.len()..];
        format!("{}{}", before.trim_end(), after.trim_start())
            .trim()
            .to_string()
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_adapter_default_paths() {
        let adapter = CursorAdapter;
        assert_eq!(adapter.kind(), HarnessKind::Cursor);
        let home = PathBuf::from("/tmp/home");
        assert_eq!(
            adapter.default_config_path(&home),
            PathBuf::from("/tmp/home/.cursorrules")
        );
    }

    #[test]
    fn managed_block_injection_and_stripping() {
        let original = "# User Rules\nDo not mutate env.";
        let managed = "# Compound Engineering Rules\nFollow OpenSpec 7-stage workflow.";
        let updated = update_managed_block(original, managed);

        assert!(updated.contains(CE_MANAGED_BEGIN));
        assert!(updated.contains(CE_MANAGED_END));
        assert!(updated.contains("Follow OpenSpec 7-stage workflow."));
        assert!(updated.contains("# User Rules"));

        let stripped = strip_managed_block(&updated);
        assert_eq!(stripped, original);
    }
}
