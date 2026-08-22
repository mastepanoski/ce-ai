//! Harness abstraction module for multi-harness support.

pub mod agents;
pub mod claude;
pub mod copilot;
pub mod cursor;
pub mod custom;
pub mod generic_json;
pub mod opencode;
pub mod pi;

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
            "opencode", "claude", "pi", "cursor", "copilot", "codex", "grok", "kimi", "agy",
            "deepseek", "fx", "custom",
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
                home_dir.join(".claude").exists()
                    || home_dir.join(".claude.json").exists()
                    || home_dir.join(".config").join("claude").exists()
            }
            HarnessKind::Pi => home_dir.join(".pi").exists() || home_dir.join(".pi-lens").exists(),
            HarnessKind::Cursor => {
                home_dir.join(".cursor").exists() || home_dir.join(".cursorrules").exists()
            }
            HarnessKind::Copilot => {
                home_dir.join(".copilot").exists()
                    || home_dir
                        .join(".github")
                        .join("copilot-instructions.md")
                        .exists()
            }
            HarnessKind::Codex => {
                home_dir.join(".codex").exists() || home_dir.join(".config").join("codex").exists()
            }
            HarnessKind::Grok => {
                home_dir.join(".grok").exists() || home_dir.join(".config").join("grok").exists()
            }
            HarnessKind::Kimi => {
                home_dir.join(".kimi").exists()
                    || home_dir.join(".kimi-code").exists()
                    || home_dir.join(".config").join("kimi").exists()
            }
            HarnessKind::Agy => {
                home_dir.join(".gemini").join("antigravity-cli").exists()
                    || home_dir.join(".gemini").exists()
                    || home_dir.join(".config").join("antigravity").exists()
            }
            HarnessKind::Deepseek => {
                home_dir.join(".deepseek").exists()
                    || home_dir.join(".config").join("deepseek").exists()
            }
            HarnessKind::Fx => {
                home_dir.join(".fx").exists() || home_dir.join(".config").join("fx").exists()
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
            HarnessKind::Deepseek,
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
                home_dir.join(".claude.json").exists()
                    || home_dir.join(".claude").join("plugins").exists()
                    || home_dir
                        .join(".config")
                        .join("claude")
                        .join("claude.json")
                        .exists()
            }
            HarnessKind::Pi => {
                home_dir.join(".pi").join("config.json").exists()
                    || home_dir.join(".pi").join("plugins").exists()
            }
            HarnessKind::Cursor => {
                home_dir.join(".cursorrules").exists()
                    || home_dir.join(".cursor").join("rules").exists()
            }
            HarnessKind::Copilot => {
                home_dir
                    .join(".github")
                    .join("copilot-instructions.md")
                    .exists()
                    || home_dir.join(".copilot").exists()
            }
            HarnessKind::Codex => {
                home_dir.join(".codex").join("codex.json").exists()
                    || home_dir.join(".config").join("codex").exists()
            }
            HarnessKind::Grok => {
                home_dir.join(".grok").join("grok.json").exists()
                    || home_dir.join(".config").join("grok").exists()
            }
            HarnessKind::Kimi => {
                home_dir.join(".kimi").join("kimi.json").exists()
                    || home_dir.join(".kimi-code").exists()
                    || home_dir.join(".config").join("kimi").exists()
            }
            HarnessKind::Agy => {
                home_dir
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
            HarnessKind::Deepseek => {
                home_dir.join(".deepseek").join("deepseek.json").exists()
                    || home_dir.join(".config").join("deepseek").exists()
            }
            HarnessKind::Fx => {
                home_dir.join(".fx").join("fx.json").exists()
                    || home_dir.join(".config").join("fx").exists()
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
            HarnessKind::Deepseek,
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

    /// Resolves target configuration file path for the harness given a base config directory.
    pub fn config_path(&self, base_dir: &Path) -> PathBuf {
        match self {
            HarnessKind::Opencode => base_dir.join("opencode.json"),
            HarnessKind::Claude => base_dir.join("claude.json"),
            HarnessKind::Pi => base_dir.join("config.json"),
            HarnessKind::Cursor => base_dir.join(".cursorrules"),
            HarnessKind::Copilot => base_dir.join("copilot-instructions.md"),
            HarnessKind::Codex => base_dir.join("codex.json"),
            HarnessKind::Grok => base_dir.join("grok.json"),
            HarnessKind::Kimi => base_dir.join("kimi.json"),
            HarnessKind::Agy => base_dir.join("antigravity.json"),
            HarnessKind::Deepseek => base_dir.join("deepseek.json"),
            HarnessKind::Fx => base_dir.join("fx.json"),
            HarnessKind::Custom => base_dir.join("custom.json"),
        }
    }
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
}
