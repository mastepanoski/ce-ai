//! Harness abstraction module for multi-harness support.

pub mod agents;
pub mod agy;
pub mod claude;
pub mod codex;
pub mod copilot;
pub mod cursor;
pub mod custom;
pub mod fx;
pub mod grok;
pub mod kimi;
pub mod opencode;
pub mod pi;

pub use grok::{
    strip_managed_block as strip_managed_rule_block, update_grok_rule_md as update_managed_rule_md,
    CE_MANAGED_BEGIN,
};

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::CeError;

/// Supported AI coding harness identifiers.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HarnessKind {
    Opencode,
    Claude,
    Pi,
    Cursor,
    Copilot,
    Codex,
    Grok,
    Kimi,
    Agy,
    Deepseek,
    Fx,
    Custom,
}

#[allow(dead_code)]
impl HarnessKind {
    /// List all natively supported harness identifiers as string slices.
    pub fn all_str() -> &'static [&'static str] {
        &[
            "opencode", "claude", "pi", "cursor", "copilot", "codex", "grok", "kimi", "agy", "fx",
            "custom",
        ]
    }

    /// List all natively supported harness enum variants.
    pub fn all() -> Vec<HarnessKind> {
        vec![
            HarnessKind::Opencode,
            HarnessKind::Claude,
            HarnessKind::Pi,
            HarnessKind::Cursor,
            HarnessKind::Copilot,
            HarnessKind::Codex,
            HarnessKind::Grok,
            HarnessKind::Kimi,
            HarnessKind::Agy,
            HarnessKind::Fx,
            HarnessKind::Custom,
        ]
    }

    /// Check if harness directory or configuration exists on the host system.
    pub fn is_installed_on_host(&self, home_dir: &Path) -> bool {
        match self {
            HarnessKind::Opencode => {
                home_dir.join(".config").join("opencode").exists()
                    || home_dir.join(".opencode").exists()
            }
            HarnessKind::Claude => {
                let dir = self.harness_dir(home_dir);
                dir.exists()
                    || home_dir.join(".claude.json").exists()
                    || dir.join("settings.json").exists()
            }
            HarnessKind::Pi => {
                let dir = self.harness_dir(home_dir);
                dir.exists() || home_dir.join(".pi").exists()
            }
            HarnessKind::Cursor => {
                home_dir.join(".cursor").exists() || home_dir.join(".cursorrules").exists()
            }
            HarnessKind::Copilot => {
                let dir = self.harness_dir(home_dir);
                dir.exists() || dir.join("mcp-config.json").exists()
            }
            HarnessKind::Codex => {
                let dir = self.harness_dir(home_dir);
                dir.exists() || dir.join("config.toml").exists()
            }
            HarnessKind::Grok => {
                let dir = self.harness_dir(home_dir);
                dir.exists() || dir.join("config.toml").exists()
            }
            HarnessKind::Kimi => {
                let dir = self.harness_dir(home_dir);
                dir.exists() || dir.join("mcp.json").exists()
            }
            HarnessKind::Agy => {
                let dir = self.harness_dir(home_dir);
                dir.exists() || home_dir.join(".config").join("antigravity").exists()
            }
            HarnessKind::Deepseek => false,
            HarnessKind::Fx => {
                let dir = self.harness_dir(home_dir);
                dir.exists() || home_dir.join(".fx").exists()
            }
            HarnessKind::Custom => false,
        }
    }

    /// Auto-detect all harnesses present on the host system.
    pub fn detect_installed_harnesses(home_dir: &Path) -> Vec<HarnessKind> {
        let all = [
            HarnessKind::Opencode,
            HarnessKind::Claude,
            HarnessKind::Pi,
            HarnessKind::Cursor,
            HarnessKind::Copilot,
            HarnessKind::Codex,
            HarnessKind::Grok,
            HarnessKind::Kimi,
            HarnessKind::Agy,
            HarnessKind::Fx,
        ];
        all.into_iter()
            .filter(|h| h.is_installed_on_host(home_dir))
            .collect()
    }

    /// Check if compound-engineering assets are installed for this harness on the host system.
    pub fn is_ce_installed(&self, home_dir: &Path) -> bool {
        match self {
            HarnessKind::Opencode => {
                let opencode_dir = home_dir.join(".config").join("opencode");
                opencode_dir.join("opencode.json").exists()
                    || opencode_dir.join("compound-engineering").exists()
                    || home_dir.join(".opencode").join("plugins").exists()
            }
            HarnessKind::Claude => {
                let dir = self.harness_dir(home_dir);
                home_dir.join(".claude.json").exists()
                    || dir.join("skills").exists()
                    || dir.join("settings.json").exists()
            }
            HarnessKind::Pi => {
                let dir = self.harness_dir(home_dir);
                dir.join("skills").exists()
            }
            HarnessKind::Cursor => {
                home_dir.join(".cursorrules").exists()
                    || home_dir.join(".cursor").join("rules").exists()
            }
            HarnessKind::Copilot => {
                let dir = self.harness_dir(home_dir);
                dir.join("mcp-config.json").exists() || dir.join("skills").exists()
            }
            HarnessKind::Codex => {
                let dir = self.harness_dir(home_dir);
                dir.join("config.toml").exists() || dir.join("skills").exists()
            }
            HarnessKind::Grok => {
                let dir = self.harness_dir(home_dir);
                dir.join("config.toml").exists() || dir.join("skills").exists()
            }
            HarnessKind::Kimi => {
                let dir = self.harness_dir(home_dir);
                dir.join("mcp.json").exists() || dir.join("skills").exists()
            }
            HarnessKind::Agy => {
                let dir = self.harness_dir(home_dir);
                dir.join("config").join("mcp_config.json").exists()
                    || dir.join("config").join("skills").exists()
                    || home_dir
                        .join(".gemini")
                        .join("antigravity-cli")
                        .join("antigravity.json")
                        .exists()
                    || home_dir
                        .join(".gemini")
                        .join("antigravity-cli")
                        .join("plugins")
                        .exists()
                    || home_dir.join(".config").join("antigravity").exists()
            }
            HarnessKind::Deepseek => false,
            HarnessKind::Fx => {
                let dir = self.harness_dir(home_dir);
                dir.join("mcp.json").exists() || dir.join("skills").exists()
            }
            HarnessKind::Custom => false,
        }
    }

    /// Auto-detect all host harnesses that have compound-engineering installed.
    pub fn detect_ce_installed_harnesses(home_dir: &Path) -> Vec<HarnessKind> {
        let all = [
            HarnessKind::Opencode,
            HarnessKind::Claude,
            HarnessKind::Pi,
            HarnessKind::Cursor,
            HarnessKind::Copilot,
            HarnessKind::Codex,
            HarnessKind::Grok,
            HarnessKind::Kimi,
            HarnessKind::Agy,
            HarnessKind::Fx,
        ];
        all.into_iter()
            .filter(|h| h.is_ce_installed(home_dir))
            .collect()
    }

    /// Return string slice representation of the harness kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            HarnessKind::Opencode => "opencode",
            HarnessKind::Claude => "claude",
            HarnessKind::Pi => "pi",
            HarnessKind::Cursor => "cursor",
            HarnessKind::Copilot => "copilot",
            HarnessKind::Codex => "codex",
            HarnessKind::Grok => "grok",
            HarnessKind::Kimi => "kimi",
            HarnessKind::Agy => "agy",
            HarnessKind::Deepseek => "deepseek",
            HarnessKind::Fx => "fx",
            HarnessKind::Custom => "custom",
        }
    }

    /// Returns the native configuration directory root for this harness relative to `home_dir`.
    pub fn harness_dir(&self, home_dir: &Path) -> PathBuf {
        match self {
            HarnessKind::Opencode => home_dir.join(".config").join("opencode"),
            HarnessKind::Claude => std::env::var_os("CLAUDE_CONFIG_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir.join(".claude")),
            HarnessKind::Pi => std::env::var_os("PI_CODING_AGENT_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir.join(".pi").join("agent")),
            HarnessKind::Cursor => home_dir.join(".cursor"),
            HarnessKind::Copilot => std::env::var_os("COPILOT_CONFIG_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir.join(".copilot")),
            HarnessKind::Codex => std::env::var_os("CODEX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir.join(".codex")),
            HarnessKind::Grok => std::env::var_os("GROK_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir.join(".grok")),
            HarnessKind::Kimi => std::env::var_os("KIMI_CODE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir.join(".kimi-code")),
            HarnessKind::Agy => crate::harness::agy::AgyAdapter.harness_dir(home_dir),
            HarnessKind::Deepseek => home_dir.join(".config").join("deepseek"),
            HarnessKind::Fx => std::env::var_os("FX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir.join(".fx")),
            // Single custom-mode contract: the config file lives directly
            // under ~/.ce-ai (see harness::custom::CONFIG_FILE_NAME).
            HarnessKind::Custom => home_dir.join(".ce-ai"),
        }
    }

    /// Resolves target configuration file path for the harness given a base config directory.
    pub fn config_path(&self, base_dir: &Path) -> PathBuf {
        match self {
            HarnessKind::Opencode => base_dir.join("opencode.json"),
            HarnessKind::Claude => {
                crate::harness::claude::ClaudeAdapter.default_config_path(base_dir)
            }
            HarnessKind::Pi => crate::harness::pi::PiAdapter.default_config_path(base_dir),
            HarnessKind::Cursor => base_dir.join("mcp.json"),
            HarnessKind::Copilot => {
                crate::harness::copilot::CopilotAdapter.default_config_path(base_dir)
            }
            HarnessKind::Codex => crate::harness::codex::CodexAdapter.default_config_path(base_dir),
            HarnessKind::Grok => crate::harness::grok::GrokAdapter.default_config_path(base_dir),
            HarnessKind::Kimi => crate::harness::kimi::KimiAdapter.default_config_path(base_dir),
            HarnessKind::Agy => crate::harness::agy::AgyAdapter.default_config_path(base_dir),
            HarnessKind::Deepseek => base_dir.join("deepseek.json"),
            HarnessKind::Fx => crate::harness::fx::FxAdapter.default_config_path(base_dir),
            HarnessKind::Custom => base_dir.join(custom::CONFIG_FILE_NAME),
        }
    }
}

/// Resolves the home/base directory from context for native harness directories.
pub fn home_dir_from_ctx(ctx: &crate::commands::Context) -> PathBuf {
    if let Some(config_dir) = ctx.opencode_config_dir.parent() {
        if config_dir.file_name().and_then(|s| s.to_str()) == Some(".config") {
            if let Some(home) = config_dir.parent() {
                return home.to_path_buf();
            }
        }
        return config_dir.to_path_buf();
    }
    ctx.opencode_config_dir.clone()
}

impl FromStr for HarnessKind {
    type Err = CeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "opencode" => Ok(HarnessKind::Opencode),
            "claude" => Ok(HarnessKind::Claude),
            "pi" => Ok(HarnessKind::Pi),
            "cursor" => Ok(HarnessKind::Cursor),
            "copilot" => Ok(HarnessKind::Copilot),
            "codex" => Ok(HarnessKind::Codex),
            "grok" => Ok(HarnessKind::Grok),
            "kimi" => Ok(HarnessKind::Kimi),
            "agy" => Ok(HarnessKind::Agy),
            "deepseek" => Ok(HarnessKind::Deepseek),
            "fx" | "fx.sh" => Ok(HarnessKind::Fx),
            "custom" => Ok(HarnessKind::Custom),
            unknown => Err(CeError::Usage(format!(
                "unknown harness '{}'. Supported harnesses: {}",
                unknown,
                HarnessKind::all_str().join(", ")
            ))),
        }
    }
}

impl fmt::Display for HarnessKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            HarnessKind::Opencode => "opencode",
            HarnessKind::Claude => "claude",
            HarnessKind::Pi => "pi",
            HarnessKind::Cursor => "cursor",
            HarnessKind::Copilot => "copilot",
            HarnessKind::Codex => "codex",
            HarnessKind::Grok => "grok",
            HarnessKind::Kimi => "kimi",
            HarnessKind::Agy => "agy",
            HarnessKind::Deepseek => "deepseek",
            HarnessKind::Fx => "fx",
            HarnessKind::Custom => "custom",
        };
        write!(f, "{}", s)
    }
}

/// Abstract interface implemented by harness adapters.
#[allow(dead_code)]
pub trait HarnessAdapter {
    fn kind(&self) -> HarnessKind;
    fn default_config_path(&self, home: &Path) -> PathBuf;

    /// Primary instruction file name managed by this harness (default: AGENTS.md).
    fn canonical_instruction_file(&self) -> PathBuf {
        PathBuf::from("AGENTS.md")
    }

    /// Derived reference stub files (e.g. CLAUDE.md) associated with this harness.
    fn derived_stub_files(&self) -> Vec<PathBuf> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enum_parsing_and_resolution() {
        assert_eq!(
            "opencode".parse::<HarnessKind>().unwrap(),
            HarnessKind::Opencode
        );
        assert_eq!(
            "CLAUDE".parse::<HarnessKind>().unwrap(),
            HarnessKind::Claude
        );
        assert_eq!("pi".parse::<HarnessKind>().unwrap(), HarnessKind::Pi);
        assert_eq!(
            "cursor".parse::<HarnessKind>().unwrap(),
            HarnessKind::Cursor
        );
        assert_eq!(
            "copilot".parse::<HarnessKind>().unwrap(),
            HarnessKind::Copilot
        );
        assert_eq!("codex".parse::<HarnessKind>().unwrap(), HarnessKind::Codex);
        assert_eq!("grok".parse::<HarnessKind>().unwrap(), HarnessKind::Grok);
        assert_eq!("kimi".parse::<HarnessKind>().unwrap(), HarnessKind::Kimi);
        assert_eq!("agy".parse::<HarnessKind>().unwrap(), HarnessKind::Agy);
        assert_eq!(
            "deepseek".parse::<HarnessKind>().unwrap(),
            HarnessKind::Deepseek
        );
        assert_eq!("fx.sh".parse::<HarnessKind>().unwrap(), HarnessKind::Fx);
        assert_eq!(
            "custom".parse::<HarnessKind>().unwrap(),
            HarnessKind::Custom
        );

        assert!(matches!(
            "invalid_harness".parse::<HarnessKind>(),
            Err(CeError::Usage(_))
        ));
    }

    #[test]
    fn auto_detects_installed_harnesses() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let home = tmp.path();

        std::fs::create_dir_all(home.join(".config/opencode")).unwrap();
        std::fs::write(home.join(".config/opencode/opencode.json"), "{}").unwrap();
        std::fs::write(home.join(".claude.json"), "{}").unwrap();

        let detected = HarnessKind::detect_installed_harnesses(home);
        assert_eq!(detected.len(), 2);
        assert!(detected.contains(&HarnessKind::Opencode));
        assert!(detected.contains(&HarnessKind::Claude));
    }

    #[test]
    fn detects_ce_installed_harnesses() {
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let home = tmp.path();

        std::fs::create_dir_all(home.join(".config/opencode")).unwrap();
        std::fs::write(home.join(".config/opencode/opencode.json"), "{}").unwrap();
        std::fs::write(home.join(".claude.json"), "{}").unwrap();

        let ce_harnesses = HarnessKind::detect_ce_installed_harnesses(home);
        assert_eq!(ce_harnesses.len(), 2);
        assert!(ce_harnesses.contains(&HarnessKind::Opencode));
        assert!(ce_harnesses.contains(&HarnessKind::Claude));
    }

    pub(crate) static HARNESS_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn harness_dir_resolves_native_paths_for_all_kinds() {
        let _guard = HARNESS_ENV_LOCK.lock().unwrap();
        std::env::remove_var("GROK_HOME");
        std::env::remove_var("CODEX_HOME");
        std::env::remove_var("COPILOT_CONFIG_DIR");
        std::env::remove_var("CLAUDE_CONFIG_DIR");
        std::env::remove_var("KIMI_CODE_HOME");
        std::env::remove_var("ANTIGRAVITY_CONFIG_DIR");
        std::env::remove_var("GEMINI_HOME");
        std::env::remove_var("PI_CODING_AGENT_DIR");
        std::env::remove_var("FX_HOME");
        let home = Path::new("/tmp/home");
        assert_eq!(
            HarnessKind::Opencode.harness_dir(home),
            home.join(".config/opencode")
        );
        assert_eq!(HarnessKind::Claude.harness_dir(home), home.join(".claude"));
        assert_eq!(HarnessKind::Cursor.harness_dir(home), home.join(".cursor"));
        assert_eq!(
            HarnessKind::Pi.harness_dir(home),
            home.join(".pi").join("agent")
        );
        assert_eq!(
            HarnessKind::Copilot.harness_dir(home),
            home.join(".copilot")
        );
        assert_eq!(HarnessKind::Grok.harness_dir(home), home.join(".grok"));
        assert_eq!(HarnessKind::Kimi.harness_dir(home), home.join(".kimi-code"));
        assert_eq!(HarnessKind::Agy.harness_dir(home), home.join(".gemini"));
        assert_eq!(HarnessKind::Fx.harness_dir(home), home.join(".fx"));
    }
}
