//! Concrete harness driver implementations for OpenCode, Claude Code, Pi, Codex, and Cursor.

use crate::commands::Context;
use crate::compat::release::CeRelease;
use crate::error::CeError;
use crate::fleet::driver::{FleetAction, FleetHarnessDriver};
use crate::harness::HarnessKind;
use std::fs;

/// Compares two version strings for semantic equality, normalizing tag formats
/// (e.g. "v3.30.4", "3.30.4", and "compound-engineering-v3.30.4" all match).
pub fn is_version_match(v1: &str, v2: &str) -> bool {
    if v1 == v2 {
        return true;
    }
    let norm1 = CeRelease::parse_tag(v1)
        .map(|r| r.version)
        .unwrap_or_else(|| v1.trim_start_matches('v').to_string());
    let norm2 = CeRelease::parse_tag(v2)
        .map(|r| r.version)
        .unwrap_or_else(|| v2.trim_start_matches('v').to_string());
    norm1 == norm2
}

fn detect_claude_installed_plugin(ctx: &Context) -> Option<(String, String)> {
    let installed_plugins_path = ctx
        .home_dir()
        .join(".claude")
        .join("plugins")
        .join("installed_plugins.json");
    if installed_plugins_path.exists() {
        if let Ok(content) = fs::read_to_string(&installed_plugins_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(plugins) = val.get("plugins").and_then(|p| p.as_object()) {
                    for (key, installations) in plugins {
                        let plugin_name = key.split('@').next().unwrap_or(key);
                        if plugin_name == "compound-engineering" {
                            if let Some(arr) = installations.as_array() {
                                let ver = arr
                                    .iter()
                                    .find(|inst| {
                                        inst.get("scope").and_then(|s| s.as_str()) == Some("user")
                                    })
                                    .or_else(|| arr.first())
                                    .and_then(|inst| inst.get("version"))
                                    .and_then(|v| v.as_str());

                                if let Some(v) = ver {
                                    return Some((key.clone(), v.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn detect_version_from_state(ctx: &Context, harness_name: &str) -> Option<String> {
    let state_file = ctx.state_path();
    if !state_file.exists() {
        return None;
    }
    if let Ok(state) = crate::state::state::State::load(&state_file) {
        for h in &state.installed_harnesses {
            if h["name"].as_str() == Some(harness_name) {
                if let Some(v) = h["version"].as_str() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

/// Native OpenCode driver.
pub struct OpenCodeDriver;

impl FleetHarnessDriver for OpenCodeDriver {
    fn harness_kind(&self) -> HarnessKind {
        HarnessKind::Opencode
    }

    fn is_installed(&self, ctx: &Context) -> bool {
        ctx.opencode_config_dir.exists()
            || HarnessKind::Opencode.is_installed_on_host(&ctx.home_dir())
    }

    fn detect_version(&self, ctx: &Context) -> Result<Option<String>, CeError> {
        let config_path = ctx.opencode_config_path();
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(plugins) = val.get("plugins").and_then(|p| p.as_array()) {
                        for p in plugins {
                            if let Some(s) = p.as_str() {
                                if s.contains("compound-engineering") {
                                    if let Some((_, ver)) = s.rsplit_once('@') {
                                        if !ver.is_empty() && ver != "compound-engineering" {
                                            return Ok(Some(ver.to_string()));
                                        }
                                    }
                                    return Ok(Some("latest".to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(v) = detect_version_from_state(ctx, "opencode") {
            return Ok(Some(v));
        }

        let home = ctx.home_dir();
        if HarnessKind::Opencode.is_ce_installed(&home) {
            return Ok(Some("host-detected".to_string()));
        }

        Ok(None)
    }

    fn plan_sync(&self, ctx: &Context, target_version: &str) -> Result<FleetAction, CeError> {
        if let Some(installed) = self.detect_version(ctx)? {
            if is_version_match(&installed, target_version) {
                return Ok(FleetAction::UpToDate);
            }
        }

        Ok(FleetAction::UpdateConfig {
            file_path: ctx.opencode_config_path(),
            key: "plugins".to_string(),
            value: serde_json::json!([format!(
                "@everyinc/compound-engineering@{}",
                target_version
            )]),
            description: format!(
                "Update opencode.json plugins to pin @everyinc/compound-engineering@{}",
                target_version
            ),
        })
    }
}

/// Native Claude Code driver.
pub struct ClaudeDriver;

impl FleetHarnessDriver for ClaudeDriver {
    fn harness_kind(&self) -> HarnessKind {
        HarnessKind::Claude
    }

    fn is_installed(&self, ctx: &Context) -> bool {
        HarnessKind::Claude.is_installed_on_host(&ctx.home_dir())
    }

    fn detect_version(&self, ctx: &Context) -> Result<Option<String>, CeError> {
        if let Some((_, v)) = detect_claude_installed_plugin(ctx) {
            return Ok(Some(v));
        }

        if let Some(v) = detect_version_from_state(ctx, "claude") {
            return Ok(Some(v));
        }

        let home = ctx.home_dir();
        if HarnessKind::Claude.is_ce_installed(&home) {
            return Ok(Some("host-detected".to_string()));
        }

        Ok(None)
    }

    fn plan_sync(&self, ctx: &Context, target_version: &str) -> Result<FleetAction, CeError> {
        let plugin_info = detect_claude_installed_plugin(ctx);
        if let Some((_, ref installed_ver)) = plugin_info {
            if is_version_match(installed_ver, target_version) {
                return Ok(FleetAction::UpToDate);
            }
        } else if let Some(installed) = self.detect_version(ctx)? {
            if is_version_match(&installed, target_version) {
                return Ok(FleetAction::UpToDate);
            }
        }

        let (subcmd, target) = match plugin_info {
            Some((key, _)) => ("update".to_string(), key),
            None => (
                "install".to_string(),
                "compound-engineering@compound-engineering-plugin".to_string(),
            ),
        };

        Ok(FleetAction::RunCommand {
            program: "claude".to_string(),
            args: vec![
                "plugin".to_string(),
                subcmd.clone(),
                target.clone(),
                "-y".to_string(),
            ],
            description: format!(
                "{} Claude Code plugin {} to {}",
                if subcmd == "update" {
                    "Update"
                } else {
                    "Install"
                },
                target,
                target_version
            ),
        })
    }
}

/// Native Pi driver.
pub struct PiDriver;

impl FleetHarnessDriver for PiDriver {
    fn harness_kind(&self) -> HarnessKind {
        HarnessKind::Pi
    }

    fn is_installed(&self, ctx: &Context) -> bool {
        HarnessKind::Pi.is_installed_on_host(&ctx.home_dir())
    }

    fn detect_version(&self, ctx: &Context) -> Result<Option<String>, CeError> {
        if let Some(v) = detect_version_from_state(ctx, "pi") {
            return Ok(Some(v));
        }

        let home = ctx.home_dir();
        if HarnessKind::Pi.is_ce_installed(&home) {
            return Ok(Some("host-detected".to_string()));
        }

        Ok(None)
    }

    fn plan_sync(&self, ctx: &Context, target_version: &str) -> Result<FleetAction, CeError> {
        if let Some(installed) = self.detect_version(ctx)? {
            if is_version_match(&installed, target_version) {
                return Ok(FleetAction::UpToDate);
            }
        }

        Ok(FleetAction::RunCommand {
            program: "npm".to_string(),
            args: vec![
                "install".to_string(),
                format!("@everyinc/compound-engineering@{}", target_version),
            ],
            description: format!(
                "Install/update Pi extension @everyinc/compound-engineering@{}",
                target_version
            ),
        })
    }
}

/// Native Codex driver.
pub struct CodexDriver;

impl FleetHarnessDriver for CodexDriver {
    fn harness_kind(&self) -> HarnessKind {
        HarnessKind::Codex
    }

    fn is_installed(&self, ctx: &Context) -> bool {
        HarnessKind::Codex.is_installed_on_host(&ctx.home_dir())
    }

    fn detect_version(&self, ctx: &Context) -> Result<Option<String>, CeError> {
        if let Some(v) = detect_version_from_state(ctx, "codex") {
            return Ok(Some(v));
        }

        let home = ctx.home_dir();
        if HarnessKind::Codex.is_ce_installed(&home) {
            return Ok(Some("host-detected".to_string()));
        }

        Ok(None)
    }

    fn plan_sync(&self, ctx: &Context, target_version: &str) -> Result<FleetAction, CeError> {
        if let Some(installed) = self.detect_version(ctx)? {
            if is_version_match(&installed, target_version) {
                return Ok(FleetAction::UpToDate);
            }
        }

        Ok(FleetAction::UpdateConfig {
            file_path: ctx.home_dir().join(".codex").join("config.json"),
            key: "compound_engineering_version".to_string(),
            value: serde_json::json!(target_version),
            description: format!(
                "Pin Codex compound-engineering version to {}",
                target_version
            ),
        })
    }
}

/// Native Cursor driver.
pub struct CursorDriver;

impl FleetHarnessDriver for CursorDriver {
    fn harness_kind(&self) -> HarnessKind {
        HarnessKind::Cursor
    }

    fn is_installed(&self, ctx: &Context) -> bool {
        HarnessKind::Cursor.is_installed_on_host(&ctx.home_dir())
    }

    fn detect_version(&self, ctx: &Context) -> Result<Option<String>, CeError> {
        if let Some(v) = detect_version_from_state(ctx, "cursor") {
            return Ok(Some(v));
        }

        let home = ctx.home_dir();
        if HarnessKind::Cursor.is_ce_installed(&home) {
            return Ok(Some("host-detected".to_string()));
        }

        Ok(None)
    }

    fn plan_sync(&self, ctx: &Context, target_version: &str) -> Result<FleetAction, CeError> {
        if let Some(installed) = self.detect_version(ctx)? {
            if is_version_match(&installed, target_version) {
                return Ok(FleetAction::UpToDate);
            }
        }

        Ok(FleetAction::UpdateConfig {
            file_path: ctx.home_dir().join(".cursor").join("settings.json"),
            key: "compound_engineering_version".to_string(),
            value: serde_json::json!(target_version),
            description: format!(
                "Pin Cursor compound-engineering version to {}",
                target_version
            ),
        })
    }
}

/// Returns all natively supported fleet harness drivers.
pub fn all_drivers() -> Vec<Box<dyn FleetHarnessDriver>> {
    vec![
        Box::new(OpenCodeDriver),
        Box::new(ClaudeDriver),
        Box::new(PiDriver),
        Box::new(CodexDriver),
        Box::new(CursorDriver),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_opencode_driver_detect_and_plan() {
        let temp = tempdir().unwrap();
        let config_dir = temp.path().join(".ce-ai");
        let opencode_dir = temp.path().join(".config/opencode");
        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&opencode_dir).unwrap();

        let ctx = Context {
            config_dir,
            opencode_config_dir: opencode_dir.clone(),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        let driver = OpenCodeDriver;
        assert!(driver.is_installed(&ctx));

        // When opencode.json has pinned version
        fs::write(
            ctx.opencode_config_path(),
            r#"{"plugins": ["@everyinc/compound-engineering@v1.2.3"]}"#,
        )
        .unwrap();

        let ver = driver.detect_version(&ctx).unwrap();
        assert_eq!(ver, Some("v1.2.3".to_string()));

        let plan = driver.plan_sync(&ctx, "v1.2.3").unwrap();
        assert_eq!(plan, FleetAction::UpToDate);

        let plan_upgrade = driver.plan_sync(&ctx, "v1.3.0").unwrap();
        match plan_upgrade {
            FleetAction::UpdateConfig { key, value, .. } => {
                assert_eq!(key, "plugins");
                assert_eq!(
                    value,
                    serde_json::json!(["@everyinc/compound-engineering@v1.3.0"])
                );
            }
            _ => panic!("Expected UpdateConfig action"),
        }
    }

    #[test]
    fn test_claude_driver_plan_sync() {
        let temp = tempdir().unwrap();
        let ctx = Context {
            config_dir: temp.path().join(".ce-ai"),
            opencode_config_dir: temp.path().join(".config/opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        let driver = ClaudeDriver;
        let plan = driver.plan_sync(&ctx, "v2.0.0").unwrap();
        match plan {
            FleetAction::RunCommand { program, args, .. } => {
                assert_eq!(program, "claude");
                assert_eq!(
                    args,
                    vec![
                        "plugin".to_string(),
                        "install".to_string(),
                        "compound-engineering@compound-engineering-plugin".to_string(),
                        "-y".to_string(),
                    ]
                );
            }
            _ => panic!("Expected RunCommand action"),
        }
    }

    #[test]
    fn test_claude_driver_plan_sync_when_installed() {
        let temp = tempdir().unwrap();
        let plugins_dir = temp.path().join(".claude").join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();
        fs::write(
            plugins_dir.join("installed_plugins.json"),
            serde_json::json!({
                "plugins": {
                    "compound-engineering@compound-engineering-plugin": [
                        { "scope": "user", "version": "3.30.4" }
                    ]
                }
            })
            .to_string(),
        )
        .unwrap();

        let ctx = Context {
            config_dir: temp.path().join(".ce-ai"),
            opencode_config_dir: temp.path().join(".config/opencode"),
            workspace_root: None,
            dry_run: false,
            verbose: false,
            quiet: false,
        };

        let driver = ClaudeDriver;
        // When target matches normalized version
        let plan = driver.plan_sync(&ctx, "v3.30.4").unwrap();
        assert_eq!(plan, FleetAction::UpToDate);

        // When target differs
        let plan_upgrade = driver.plan_sync(&ctx, "v3.31.0").unwrap();
        match plan_upgrade {
            FleetAction::RunCommand { program, args, .. } => {
                assert_eq!(program, "claude");
                assert_eq!(
                    args,
                    vec![
                        "plugin".to_string(),
                        "update".to_string(),
                        "compound-engineering@compound-engineering-plugin".to_string(),
                        "-y".to_string(),
                    ]
                );
            }
            _ => panic!("Expected RunCommand action"),
        }
    }

    #[test]
    fn test_is_version_match() {
        assert!(is_version_match("v3.30.4", "3.30.4"));
        assert!(is_version_match("compound-engineering-v3.30.4", "v3.30.4"));
        assert!(is_version_match("compound-engineering-v3.30.4", "3.30.4"));
        assert!(is_version_match("3.30.4", "3.30.4"));
        assert!(!is_version_match("v3.30.4", "v3.30.5"));
        assert!(!is_version_match("host-detected", "v3.30.4"));
    }
}
