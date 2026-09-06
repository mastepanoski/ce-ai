//! RTK (Token Reduction Engine) harness hook integration.
//!
//! RTK intercepts agent tool execution via hooks (such as `PreToolUse` or
//! command rewrites) to condense output before it enters LLM context.
//!
//! Only a subset of harnesses officially support RTK hook injection
//! (`Claude`, `Cursor`, `Copilot`, `Codex`). Unsupported harnesses are
//! treated as an explicit, safe no-op.

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::error::CeError;
use crate::harness::HarnessKind;

/// Checks if a harness officially supports RTK hook injection.
pub fn is_rtk_supported(kind: HarnessKind) -> bool {
    matches!(
        kind,
        HarnessKind::Claude | HarnessKind::Cursor | HarnessKind::Copilot | HarnessKind::Codex
    )
}

/// Evaluates whether RTK integration is opted out via CLI flags or environment variables.
pub fn is_rtk_opted_out(skip_rtk_flag: bool, skip_companions_flag: bool) -> bool {
    if skip_rtk_flag || skip_companions_flag {
        return true;
    }
    if let Ok(val) = std::env::var("CE_AI_SKIP_RTK") {
        let v = val.trim().to_lowercase();
        if v == "1" || v == "true" || v == "yes" {
            return true;
        }
    }
    if let Ok(val) = std::env::var("CE_AI_SKIP_COMPANIONS") {
        let v = val.trim().to_lowercase();
        if v == "1" || v == "true" || v == "yes" {
            return true;
        }
    }
    false
}

/// Returns true if the `rtk` executable is available on the system PATH.
pub fn is_rtk_available() -> bool {
    Command::new("rtk")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Returns the `rtk init -g` command arguments for configuring hooks on a supported harness.
pub fn rtk_init_args(harness: HarnessKind) -> Option<&'static [&'static str]> {
    match harness {
        HarnessKind::Claude => Some(&["init", "-g", "--auto-patch", "--agent", "claude"]),
        HarnessKind::Cursor => Some(&["init", "-g", "--auto-patch", "--agent", "cursor"]),
        HarnessKind::Copilot => Some(&["init", "-g", "--copilot"]),
        HarnessKind::Codex => Some(&["init", "-g", "--codex"]),
        _ => None,
    }
}

/// Returns the `rtk init -g --uninstall` arguments for removing hooks on a supported harness.
pub fn rtk_uninstall_args(harness: HarnessKind) -> Option<&'static [&'static str]> {
    match harness {
        HarnessKind::Claude => Some(&["init", "-g", "--uninstall", "--agent", "claude"]),
        HarnessKind::Cursor => Some(&["init", "-g", "--uninstall", "--agent", "cursor"]),
        HarnessKind::Copilot => Some(&["init", "-g", "--uninstall", "--copilot"]),
        HarnessKind::Codex => Some(&["init", "-g", "--uninstall", "--codex"]),
        _ => None,
    }
}

/// Checks whether an RTK hook is already configured on disk for a given harness.
pub fn is_rtk_hook_configured(home: &Path, harness: HarnessKind) -> bool {
    match harness {
        HarnessKind::Claude => {
            let settings = home.join(".claude").join("settings.json");
            fs::read_to_string(&settings)
                .map(|content| content.contains("rtk hook claude"))
                .unwrap_or(false)
        }
        HarnessKind::Cursor => {
            let hooks = home.join(".cursor").join("hooks.json");
            fs::read_to_string(&hooks)
                .map(|content| content.contains("rtk hook cursor"))
                .unwrap_or(false)
        }
        HarnessKind::Copilot => home
            .join(".copilot")
            .join("hooks")
            .join("rtk-rewrite.json")
            .exists(),
        HarnessKind::Codex => home.join(".codex").join("RTK.md").exists(),
        _ => false,
    }
}

/// Configures the RTK hook for a supported harness under the target home directory.
///
/// Returns `Ok(true)` if configured, `Ok(false)` if skipped or unavailable, or `Err` on I/O error.
pub fn configure_rtk_hook(
    home: &Path,
    harness: HarnessKind,
    dry_run: bool,
    quiet: bool,
) -> Result<bool, CeError> {
    if !is_rtk_supported(harness) {
        if !quiet {
            println!("rtk: hook injection not supported for {harness}, skipping");
        }
        return Ok(false);
    }

    if dry_run {
        if !quiet {
            println!("[dry-run] would configure rtk hook for {harness}");
        }
        return Ok(true);
    }

    if !is_rtk_available() {
        if !quiet {
            eprintln!(
                "warning: rtk executable not found on PATH; skipping hook injection (install via 'ce-ai tools install rtk')"
            );
        }
        return Ok(false);
    }

    let Some(args) = rtk_init_args(harness) else {
        return Ok(false);
    };

    // Pre-condition: rtk init -g upstream expects target directories and
    // ~/.claude to exist before writing files.
    let harness_dir = match harness {
        HarnessKind::Claude => home.join(".claude"),
        HarnessKind::Cursor => home.join(".cursor"),
        HarnessKind::Copilot => home.join(".copilot"),
        HarnessKind::Codex => home.join(".codex"),
        _ => unreachable!(),
    };
    fs::create_dir_all(&harness_dir)?;
    fs::create_dir_all(home.join(".claude"))?;

    let mut cmd = Command::new("rtk");
    cmd.args(args);
    cmd.env("HOME", home);
    #[cfg(windows)]
    cmd.env("USERPROFILE", home);

    match cmd.output() {
        Ok(out) if out.status.success() => {
            if !quiet {
                println!("✓ rtk: configured hook for {harness}");
            }
            Ok(true)
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            if !quiet {
                eprintln!(
                    "warning: failed to configure rtk hook for {harness}: {}",
                    stderr.trim()
                );
            }
            Ok(false)
        }
        Err(e) => {
            if !quiet {
                eprintln!("warning: failed to execute rtk for {harness}: {e}");
            }
            Ok(false)
        }
    }
}

/// Unconfigures the RTK hook for a supported harness under the target home directory.
pub fn unconfigure_rtk_hook(
    home: &Path,
    harness: HarnessKind,
    dry_run: bool,
    quiet: bool,
) -> Result<bool, CeError> {
    if !is_rtk_supported(harness) {
        return Ok(false);
    }

    if dry_run {
        if !quiet {
            println!("[dry-run] would unconfigure rtk hook for {harness}");
        }
        return Ok(true);
    }

    if !is_rtk_available() {
        return Ok(false);
    }

    let Some(args) = rtk_uninstall_args(harness) else {
        return Ok(false);
    };

    let mut cmd = Command::new("rtk");
    cmd.args(args);
    cmd.env("HOME", home);
    #[cfg(windows)]
    cmd.env("USERPROFILE", home);

    match cmd.output() {
        Ok(out) if out.status.success() => {
            if !quiet {
                println!("✓ rtk: uninstalled hook for {harness}");
            }
            Ok(true)
        }
        Ok(_) | Err(_) => Ok(false),
    }
}

#[cfg(test)]
#[path = "tests/rtk.rs"]
mod tests;
