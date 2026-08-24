//! Custom harness fallback mode adapter implementation (--harness custom).
//!
//! Single path contract: `<home>/.ce-ai/custom_harness.json` describes where
//! CE plugin assets, skill folders, and an optional managed rules block live.
//! No other hardcoded custom-mode path may exist in the codebase.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::commands::init_prj::{
    compute_sha256, render_block_content, BLOCK_BEGIN_MARKER, BLOCK_END_MARKER, BLOCK_VERSION,
};
use crate::error::CeError;
use crate::harness::{HarnessAdapter, HarnessKind};
use crate::state::state::AdoptionTier;

/// File name of the single-contract custom-mode configuration.
pub const CONFIG_FILE_NAME: &str = "custom_harness.json";

/// Configuration state for custom fallback harnesses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomHarnessConfig {
    pub plugins_dir: PathBuf,
    pub skills_dir: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules_file: Option<PathBuf>,
}

/// Explicit CLI flag overrides for custom-mode configuration resolution.
#[derive(Debug, Default, Clone)]
pub struct CustomConfigFlags {
    pub plugins_dir: Option<PathBuf>,
    pub skills_dir: Option<PathBuf>,
    pub rules_file: Option<PathBuf>,
}

impl CustomHarnessConfig {
    /// Absolute path of the persisted custom-mode config file.
    pub fn config_path(home: &Path) -> PathBuf {
        home.join(".ce-ai").join(CONFIG_FILE_NAME)
    }

    /// Loads the persisted custom-mode config; `Ok(None)` when absent.
    /// Malformed JSON is a hard `CeError::Runtime`, never silently ignored.
    pub fn load_from_home(home: &Path) -> Result<Option<Self>, CeError> {
        let path = Self::config_path(home);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&path)?;
        serde_json::from_slice(&bytes).map(Some).map_err(|e| {
            CeError::Runtime(format!(
                "invalid {CONFIG_FILE_NAME} at {}: {e}",
                path.display()
            ))
        })
    }

    /// Resolves the effective configuration with flag ▸ config-file
    /// precedence, `~` expansion, and relative-path anchoring. Fails fast
    /// with `CeError::Usage` when `plugins_dir`/`skills_dir` are unresolvable.
    pub fn resolve(home: &Path, flags: &CustomConfigFlags) -> Result<Self, CeError> {
        let from_file = Self::load_from_home(home)?;
        let plugins_dir = flags
            .plugins_dir
            .clone()
            .or_else(|| from_file.as_ref().map(|c| c.plugins_dir.clone()));
        let skills_dir = flags
            .skills_dir
            .clone()
            .or_else(|| from_file.as_ref().map(|c| c.skills_dir.clone()));
        let rules_file = flags
            .rules_file
            .clone()
            .or_else(|| from_file.as_ref().and_then(|c| c.rules_file.clone()));

        if plugins_dir.is_none() || skills_dir.is_none() {
            return Err(CeError::Usage(format!(
                "--harness custom requires target directories: pass --plugins-dir and \
                 --skills-dir, or persist them in {} as \
                 {{\"plugins_dir\": \"...\", \"skills_dir\": \"...\", \"rules_file\": \"...\"}}",
                Self::config_path(home).display()
            )));
        }

        Ok(Self {
            plugins_dir: absolutize(expand_tilde(plugins_dir.unwrap_or_default(), home)),
            skills_dir: absolutize(expand_tilde(skills_dir.unwrap_or_default(), home)),
            rules_file: rules_file.map(|p| absolutize(expand_tilde(p, home))),
        })
    }

    /// Restores a config previously embedded in a state entry's `custom` key.
    pub fn from_state_json(value: &serde_json::Value) -> Option<Self> {
        let dir = |key: &str| -> Option<PathBuf> {
            value[key]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
        };
        Some(Self {
            plugins_dir: dir("plugins_dir")?,
            skills_dir: dir("skills_dir")?,
            rules_file: dir("rules_file"),
        })
    }

    /// Serializes this config for embedding in a state entry's `custom` key.
    pub fn to_state_json(&self) -> serde_json::Value {
        serde_json::json!({
            "plugins_dir": self.plugins_dir.display().to_string(),
            "skills_dir": self.skills_dir.display().to_string(),
            "rules_file": self.rules_file.as_ref().map(|p| p.display().to_string()),
        })
    }
}

/// Maps a managed rel path onto its location under `plugins_dir`
/// (`plugins/x` → `Some("x")`).
pub fn plugin_rel(managed_rel: &str) -> Option<&str> {
    managed_rel.strip_prefix("plugins/")
}

/// Maps a managed rel path onto its location under `skills_dir`
/// (`skills/x` → `Some("x")`).
pub fn skill_rel(managed_rel: &str) -> Option<&str> {
    managed_rel.strip_prefix("skills/")
}

/// Expands a leading `~` against `home`.
fn expand_tilde(p: PathBuf, home: &Path) -> PathBuf {
    match p.to_str() {
        Some("~") => home.to_path_buf(),
        Some(s) if s.starts_with("~/") => home.join(&s[2..]),
        _ => p,
    }
}

/// Anchors a relative path against the process CWD so persisted snapshots
/// stay stable across invocations.
fn absolutize(p: PathBuf) -> PathBuf {
    if p.is_absolute() {
        p
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p)
    }
}

/// Renders the current-version managed CE block for markdown rules files.
fn render_full_block() -> String {
    let inner = render_block_content(AdoptionTier::Full);
    format!(
        "<!-- ce-ai:block begin v={} tier=full sha256={} -->\n{}\n{}",
        BLOCK_VERSION,
        compute_sha256(inner),
        inner,
        BLOCK_END_MARKER
    )
}

/// Ensures the markdown rules file contains exactly one current managed CE
/// block, preserving every non-managed byte. Idempotent. Returns `Ok(true)`
/// when the file content changed.
pub fn ensure_rules_block(path: &Path) -> Result<bool, CeError> {
    let existing = if path.exists() {
        std::fs::read_to_string(path)?
    } else {
        String::new()
    };
    let full_block = render_full_block();
    let updated = if let Some(start_idx) = existing.find(BLOCK_BEGIN_MARKER) {
        let Some(end_rel) = existing[start_idx..].find(BLOCK_END_MARKER) else {
            return Err(CeError::Runtime(format!(
                "malformed managed block in '{}': begin marker without end marker",
                path.display()
            )));
        };
        let end_idx = start_idx + end_rel + BLOCK_END_MARKER.len();
        format!(
            "{}{}{}",
            &existing[..start_idx],
            full_block,
            &existing[end_idx..]
        )
    } else if existing.is_empty() {
        format!("{full_block}\n")
    } else {
        let mut appended = existing.clone();
        if !appended.ends_with('\n') {
            appended.push('\n');
        }
        appended.push('\n');
        appended.push_str(&full_block);
        appended.push('\n');
        appended
    };

    if updated == existing {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    crate::state::write_atomic(path, updated.as_bytes())?;
    Ok(true)
}

/// Removes the managed CE block from a markdown rules file while preserving
/// every other byte. Returns `Ok(false)` when no block is present; a begin
/// marker without an end marker is a hard error.
pub fn strip_rules_block(path: &Path) -> Result<bool, CeError> {
    if !path.exists() {
        return Ok(false);
    }
    let text = std::fs::read_to_string(path)?;
    let Some(start_idx) = text.find(BLOCK_BEGIN_MARKER) else {
        return Ok(false);
    };
    let Some(end_rel) = text[start_idx..].find(BLOCK_END_MARKER) else {
        return Err(CeError::Runtime(format!(
            "malformed managed block in '{}': begin marker without end marker",
            path.display()
        )));
    };
    let end_idx = start_idx + end_rel + BLOCK_END_MARKER.len();

    let head = text[..start_idx].trim_end_matches(['\r', '\n']);
    let tail_content = text[end_idx..].trim_start_matches(['\r', '\n']);
    let mut out = String::with_capacity(text.len());
    if !head.is_empty() {
        out.push_str(head);
        out.push('\n');
    }
    out.push_str(tail_content);

    if out == text {
        return Ok(false);
    }
    crate::state::write_atomic(path, out.as_bytes())?;
    Ok(true)
}

/// Removes now-empty directories left behind after surgical uninstall,
/// stopping at (but never removing) the configured boundary roots.
pub fn prune_empty_dirs(start: &Path, boundaries: &[&Path]) {
    let mut current = start.to_path_buf();
    loop {
        if boundaries.iter().any(|b| current == *b) || current.parent().is_none() {
            break;
        }
        if std::fs::remove_dir(&current).is_err() {
            break;
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
    }
}

#[derive(Debug)]
pub struct CustomAdapter {
    config: Option<CustomHarnessConfig>,
}

impl CustomAdapter {
    pub fn new(config: Option<CustomHarnessConfig>) -> Self {
        Self { config }
    }

    /// The resolved configuration, when one was supplied at construction.
    pub fn config(&self) -> Option<&CustomHarnessConfig> {
        self.config.as_ref()
    }
}

impl HarnessAdapter for CustomAdapter {
    fn kind(&self) -> HarnessKind {
        HarnessKind::Custom
    }

    fn default_config_path(&self, home: &Path) -> PathBuf {
        CustomHarnessConfig::config_path(home)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn custom_adapter_default_paths_use_single_contract() {
        let home = PathBuf::from("/tmp/home");
        let adapter = CustomAdapter::new(None);
        assert_eq!(adapter.kind(), HarnessKind::Custom);
        assert_eq!(
            adapter.default_config_path(&home),
            home.join(".ce-ai").join(CONFIG_FILE_NAME)
        );
        assert!(adapter.config().is_none());
    }

    #[test]
    fn resolve_prefers_flags_over_config_file() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path();
        let cfg_path = CustomHarnessConfig::config_path(home);
        std::fs::create_dir_all(cfg_path.parent().unwrap()).unwrap();
        std::fs::write(
            &cfg_path,
            r#"{"plugins_dir": "~/file-plugins", "skills_dir": "/abs/file-skills"}"#,
        )
        .unwrap();

        let cfg = CustomHarnessConfig::resolve(
            home,
            &CustomConfigFlags {
                plugins_dir: Some(PathBuf::from("~/flag-plugins")),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(cfg.plugins_dir, home.join("flag-plugins"));
        assert_eq!(cfg.skills_dir, PathBuf::from("/abs/file-skills"));
        assert_eq!(cfg.rules_file, None);
    }

    #[test]
    fn resolve_falls_back_to_config_file_and_expands_tilde() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path();
        let cfg_path = CustomHarnessConfig::config_path(home);
        std::fs::create_dir_all(cfg_path.parent().unwrap()).unwrap();
        std::fs::write(
            &cfg_path,
            r#"{"plugins_dir": "~/p", "skills_dir": "rel-skills", "rules_file": "~/r.md"}"#,
        )
        .unwrap();

        let cfg = CustomHarnessConfig::resolve(home, &CustomConfigFlags::default()).unwrap();
        assert_eq!(cfg.plugins_dir, home.join("p"));
        assert_eq!(cfg.rules_file, Some(home.join("r.md")));
        assert!(cfg.skills_dir.is_absolute());
    }

    #[test]
    fn resolve_without_any_configuration_is_a_usage_error() {
        let tmp = TempDir::new().unwrap();
        let err =
            CustomHarnessConfig::resolve(tmp.path(), &CustomConfigFlags::default()).unwrap_err();
        assert!(matches!(err, CeError::Usage(_)));
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn load_from_home_rejects_malformed_json_as_runtime_error() {
        let tmp = TempDir::new().unwrap();
        let cfg_path = CustomHarnessConfig::config_path(tmp.path());
        std::fs::create_dir_all(cfg_path.parent().unwrap()).unwrap();
        std::fs::write(&cfg_path, "{not json").unwrap();

        let err = CustomHarnessConfig::load_from_home(tmp.path()).unwrap_err();
        assert!(matches!(err, CeError::Runtime(_)));
    }

    #[test]
    fn managed_rel_mappers_split_by_prefix() {
        assert_eq!(plugin_rel("plugins/loader.js"), Some("loader.js"));
        assert_eq!(
            skill_rel("skills/ce-work/SKILL.md"),
            Some("ce-work/SKILL.md")
        );
        assert_eq!(plugin_rel("skills/ce-work/SKILL.md"), None);
        assert_eq!(skill_rel("README.md"), None);
    }

    #[test]
    fn state_json_round_trips_resolved_config() {
        let cfg = CustomHarnessConfig {
            plugins_dir: PathBuf::from("/p"),
            skills_dir: PathBuf::from("/s"),
            rules_file: Some(PathBuf::from("/r.md")),
        };
        let parsed = CustomHarnessConfig::from_state_json(&cfg.to_state_json()).unwrap();
        assert_eq!(parsed, cfg);
    }

    #[test]
    fn from_state_json_requires_both_directories() {
        assert!(CustomHarnessConfig::from_state_json(&serde_json::json!({
            "plugins_dir": "/p"
        }))
        .is_none());
    }

    #[test]
    fn ensure_rules_block_creates_appends_and_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let rules = tmp.path().join("nested").join("rules.md");

        assert!(ensure_rules_block(&rules).unwrap());
        let first = std::fs::read_to_string(&rules).unwrap();
        assert!(first.contains(BLOCK_BEGIN_MARKER));
        assert!(first.contains(BLOCK_END_MARKER));

        assert!(!ensure_rules_block(&rules).unwrap());
        assert_eq!(std::fs::read_to_string(&rules).unwrap(), first);
    }

    #[test]
    fn ensure_rules_block_preserves_user_content_around_the_block() {
        let tmp = TempDir::new().unwrap();
        let rules = tmp.path().join("rules.md");
        std::fs::write(&rules, "# my rules\nbe excellent\n").unwrap();

        ensure_rules_block(&rules).unwrap();
        let with_block = std::fs::read_to_string(&rules).unwrap();
        assert!(with_block.starts_with("# my rules\nbe excellent\n"));

        // Re-running replaces the block in place, keeping surrounding bytes.
        ensure_rules_block(&rules).unwrap();
        let again = std::fs::read_to_string(&rules).unwrap();
        assert!(again.starts_with("# my rules\nbe excellent\n"));
    }

    #[test]
    fn strip_rules_block_removes_only_the_block() {
        let tmp = TempDir::new().unwrap();
        let rules = tmp.path().join("rules.md");
        std::fs::write(&rules, "# my rules\nbe excellent\n").unwrap();

        ensure_rules_block(&rules).unwrap();
        assert!(strip_rules_block(&rules).unwrap());
        assert_eq!(
            std::fs::read_to_string(&rules).unwrap(),
            "# my rules\nbe excellent\n"
        );
        assert!(!strip_rules_block(&rules).unwrap());

        let bare = tmp.path().join("bare.md");
        std::fs::write(&bare, "only block incoming\n").unwrap();
        ensure_rules_block(&bare).unwrap();
        assert!(strip_rules_block(&bare).unwrap());
        // User bytes survive verbatim; only the managed block disappears.
        assert_eq!(
            std::fs::read_to_string(&bare).unwrap(),
            "only block incoming\n"
        );
    }

    #[test]
    fn strip_rules_block_errors_on_malformed_block() {
        let tmp = TempDir::new().unwrap();
        let rules = tmp.path().join("broken.md");
        std::fs::write(&rules, "<!-- ce-ai:block begin v=1 tier=full -->\nno end").unwrap();

        let err = strip_rules_block(&rules).unwrap_err();
        assert!(matches!(err, CeError::Runtime(_)));
    }

    #[test]
    fn prune_empty_dirs_stops_at_boundaries() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("root");
        let deep = root.join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();

        prune_empty_dirs(&deep, &[&root]);
        assert!(root.exists());
        assert!(!root.join("a").exists());
    }
}
