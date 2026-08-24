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
#[derive(Clone, Copy)]
pub(crate) struct RegistrationSpec {
    /// Vendor registrar; `None` for No-MCP harnesses such as pi.
    pub(crate) register_mcp: Option<McpRegistrar>,
    /// Managed-skills destination relative to the harness dir; `None` for
    /// kinds that consume no skills tree (e.g. cursor).
    pub(crate) skills_subpath: Option<&'static str>,
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
    let native = |reg: McpRegistrar, subpath: &'static str| RegistrationSpec {
        register_mcp: Some(reg),
        skills_subpath: Some(subpath),
    };
    Some(match kind {
        // Cursor reads MCP servers only — it has no skills-tree consumer,
        // so nothing is copied into its directory.
        HarnessKind::Cursor => RegistrationSpec {
            register_mcp: Some(crate::harness::cursor::register_cursor_mcp_server),
            skills_subpath: None,
        },
        HarnessKind::Claude => native(crate::harness::claude::register_claude_mcp_server, "skills"),
        HarnessKind::Codex => native(crate::harness::codex::register_codex_mcp_server, "skills"),
        HarnessKind::Copilot => native(
            crate::harness::copilot::register_copilot_mcp_server,
            "skills",
        ),
        HarnessKind::Grok => native(crate::harness::grok::register_grok_mcp_server, "skills"),
        HarnessKind::Kimi => native(crate::harness::kimi::register_kimi_mcp_server, "skills"),
        HarnessKind::Agy => native(
            crate::harness::agy::register_agy_mcp_server,
            "config/skills",
        ),
        HarnessKind::Fx => native(crate::harness::fx::register_fx_mcp_server, "skills"),
        // Pi is No-MCP by design (objective 8): skills tree only.
        HarnessKind::Pi => RegistrationSpec {
            register_mcp: None,
            skills_subpath: Some("skills"),
        },
        HarnessKind::Custom | HarnessKind::Opencode | HarnessKind::Deepseek => return None,
    })
}

/// Copies the managed skills subtree into a destination root, propagating
/// IO failures (invariant #5). No-op when the source tree is absent.
pub(crate) fn copy_managed_skills(managed_dir: &Path, dest: &Path) -> Result<(), CeError> {
    let src = managed_dir.join("skills");
    if !src.exists() {
        return Ok(());
    }
    crate::source::archive::copy_dir_all(&src, dest).map_err(|e| {
        CeError::Runtime(format!(
            "failed to copy managed skills to {}: {e}",
            dest.display()
        ))
    })
}
