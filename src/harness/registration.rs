//! Exhaustive per-harness registration specification shared by `install`
//! and `sync` (Strategy-via-data): a single table whose match lists every
//! [`HarnessKind`] variant, so adding one is a compile error until it is
//! classified — the forgotten-arm fictional-write bug class becomes
//! structurally impossible on this surface.

use std::collections::BTreeMap;
use std::path::Path;

use crate::error::CeError;
use crate::harness::HarnessKind;

/// Vendor MCP registrar signature shared by every native adapter.
pub(crate) type McpRegistrar =
    fn(&Path, &str, &str, &[&str], &BTreeMap<String, String>) -> Result<(), CeError>;

/// Strategy-table entry describing how a kind registers Compound Engineering.
/// Skills-root conventions live in `sync_skills_root` (commands::sync) —
/// adoption and verification derive roots from it, not from this table.
#[derive(Clone, Copy)]
pub(crate) struct RegistrationSpec {
    /// Vendor registrar; `None` for No-MCP harnesses such as pi.
    pub(crate) register_mcp: Option<McpRegistrar>,
}

impl RegistrationSpec {
    /// Registers the companion MCP servers (`codegraph`, `engram`) when the
    /// vendor supports MCP definitions; silent no-op otherwise.
    pub(crate) fn register_companions(&self, target_config: &Path) -> Result<(), CeError> {
        let Some(register) = self.register_mcp else {
            return Ok(());
        };
        let env = BTreeMap::new();
        register(target_config, "codegraph", "codegraph", &["mcp"], &env)?;
        register(target_config, "engram", "engram", &["serve"], &env)?;
        Ok(())
    }
}

/// Exhaustive re-registration table. Dedicated call-site arms remain for
/// `Custom` (state-snapshot-driven layout), `Opencode` (plugin/skills JSON
/// writer) and `Deepseek` (de-scoped).
pub(crate) fn registration_spec(kind: HarnessKind) -> Option<RegistrationSpec> {
    let native = |reg: McpRegistrar| RegistrationSpec {
        register_mcp: Some(reg),
    };
    Some(match kind {
        // Cursor reads MCP servers only — it has no skills-tree consumer,
        // so nothing is copied into its directory.
        HarnessKind::Cursor => RegistrationSpec {
            register_mcp: Some(crate::harness::cursor::register_cursor_mcp_server),
        },
        HarnessKind::Claude => native(crate::harness::claude::register_claude_mcp_server),
        HarnessKind::Codex => native(crate::harness::codex::register_codex_mcp_server),
        HarnessKind::Copilot => native(crate::harness::copilot::register_copilot_mcp_server),
        HarnessKind::Grok => native(crate::harness::grok::register_grok_mcp_server),
        HarnessKind::Kimi => native(crate::harness::kimi::register_kimi_mcp_server),
        HarnessKind::Agy => native(crate::harness::agy::register_agy_mcp_server),
        HarnessKind::Fx => native(crate::harness::fx::register_fx_mcp_server),
        // Pi is No-MCP by design (Objective 8): skills tree only (~/.pi/agent/skills/).
        // Companion integration (codegraph, engram) is fulfilled via CLI binaries
        // available on PATH; Pi exposes no JSON/YAML configuration file for MCP registration.
        HarnessKind::Pi => RegistrationSpec { register_mcp: None },
        // Custom has a snapshot-driven layout and optional `--mcp-file` (handled via dedicated arm
        // in install/sync/uninstall).
        // Opencode uses its dedicated config writer (`crate::opencode::config::register_companions`).
        // Deepseek is de-scoped during developer preview: `dsh` uses YAML patch layers under
        // `~/.dsh` and `install --harness deepseek` returns CeError::Usage.
        HarnessKind::Custom | HarnessKind::Opencode | HarnessKind::Deepseek => return None,
    })
}

#[cfg(test)]
#[path = "tests/registration.rs"]
mod tests;
