use std::sync::Mutex;
use tempfile::tempdir;

use super::*;
use crate::harness::HarnessKind;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_is_rtk_supported_matches_support_matrix() {
    use HarnessKind::*;

    // Supported
    assert!(is_rtk_supported(Claude));
    assert!(is_rtk_supported(Cursor));
    assert!(is_rtk_supported(Copilot));
    assert!(is_rtk_supported(Codex));

    // Unsupported
    assert!(!is_rtk_supported(Opencode));
    assert!(!is_rtk_supported(Pi));
    assert!(!is_rtk_supported(Custom));
    assert!(!is_rtk_supported(Deepseek));
    assert!(!is_rtk_supported(Grok));
    assert!(!is_rtk_supported(Kimi));
    assert!(!is_rtk_supported(Agy));
    assert!(!is_rtk_supported(Fx));
}

#[test]
fn test_is_rtk_opted_out_cli_flags() {
    let _lock = ENV_MUTEX.lock().unwrap();
    std::env::remove_var("CE_AI_SKIP_RTK");
    std::env::remove_var("CE_AI_SKIP_COMPANIONS");

    assert!(is_rtk_opted_out(true, false));
    assert!(is_rtk_opted_out(false, true));
    assert!(is_rtk_opted_out(true, true));
    assert!(!is_rtk_opted_out(false, false));
}

#[test]
fn test_is_rtk_opted_out_env_vars() {
    let _lock = ENV_MUTEX.lock().unwrap();

    // CE_AI_SKIP_RTK values
    for val in ["1", "true", "yes", "TRUE", "Yes", " 1 "] {
        std::env::set_var("CE_AI_SKIP_RTK", val);
        std::env::remove_var("CE_AI_SKIP_COMPANIONS");
        assert!(is_rtk_opted_out(false, false), "expected true for {val}");
    }
    std::env::remove_var("CE_AI_SKIP_RTK");

    // CE_AI_SKIP_COMPANIONS values
    for val in ["1", "true", "yes", "TRUE", "Yes"] {
        std::env::remove_var("CE_AI_SKIP_RTK");
        std::env::set_var("CE_AI_SKIP_COMPANIONS", val);
        assert!(is_rtk_opted_out(false, false), "expected true for {val}");
    }
    std::env::remove_var("CE_AI_SKIP_COMPANIONS");

    // Negative env var values
    for val in ["0", "false", "no", "random"] {
        std::env::set_var("CE_AI_SKIP_RTK", val);
        std::env::set_var("CE_AI_SKIP_COMPANIONS", val);
        assert!(!is_rtk_opted_out(false, false), "expected false for {val}");
    }
    std::env::remove_var("CE_AI_SKIP_RTK");
    std::env::remove_var("CE_AI_SKIP_COMPANIONS");
}

#[test]
fn test_rtk_init_args_mapping() {
    assert_eq!(
        rtk_init_args(HarnessKind::Claude),
        Some(&["init", "-g", "--auto-patch", "--agent", "claude"][..])
    );
    assert_eq!(
        rtk_init_args(HarnessKind::Cursor),
        Some(&["init", "-g", "--auto-patch", "--agent", "cursor"][..])
    );
    assert_eq!(
        rtk_init_args(HarnessKind::Copilot),
        Some(&["init", "-g", "--copilot"][..])
    );
    assert_eq!(
        rtk_init_args(HarnessKind::Codex),
        Some(&["init", "-g", "--codex"][..])
    );

    // Unsupported return None
    assert!(rtk_init_args(HarnessKind::Opencode).is_none());
    assert!(rtk_init_args(HarnessKind::Pi).is_none());
    assert!(rtk_init_args(HarnessKind::Custom).is_none());
}

#[test]
fn test_rtk_uninstall_args_mapping() {
    assert_eq!(
        rtk_uninstall_args(HarnessKind::Claude),
        Some(&["init", "-g", "--uninstall", "--agent", "claude"][..])
    );
    assert_eq!(
        rtk_uninstall_args(HarnessKind::Cursor),
        Some(&["init", "-g", "--uninstall", "--agent", "cursor"][..])
    );
    assert_eq!(
        rtk_uninstall_args(HarnessKind::Copilot),
        Some(&["init", "-g", "--uninstall", "--copilot"][..])
    );
    assert_eq!(
        rtk_uninstall_args(HarnessKind::Codex),
        Some(&["init", "-g", "--uninstall", "--codex"][..])
    );

    assert!(rtk_uninstall_args(HarnessKind::Opencode).is_none());
    assert!(rtk_uninstall_args(HarnessKind::Pi).is_none());
}

#[test]
fn test_is_rtk_hook_configured_detection() {
    let temp = tempdir().unwrap();
    let home = temp.path();

    // Claude
    assert!(!is_rtk_hook_configured(home, HarnessKind::Claude));
    let claude_dir = home.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();
    fs::write(
        claude_dir.join("settings.json"),
        r#"{"hooks":{"PreToolUse":[{"matcher":"Bash","hooks":[{"command":"rtk hook claude"}]}]}}"#,
    )
    .unwrap();
    assert!(is_rtk_hook_configured(home, HarnessKind::Claude));

    // Cursor
    assert!(!is_rtk_hook_configured(home, HarnessKind::Cursor));
    let cursor_dir = home.join(".cursor");
    fs::create_dir_all(&cursor_dir).unwrap();
    fs::write(
        cursor_dir.join("hooks.json"),
        r#"{"hooks":{"preToolUse":[{"command":"rtk hook cursor"}]}}"#,
    )
    .unwrap();
    assert!(is_rtk_hook_configured(home, HarnessKind::Cursor));

    // Copilot
    assert!(!is_rtk_hook_configured(home, HarnessKind::Copilot));
    let copilot_hooks = home.join(".copilot").join("hooks");
    fs::create_dir_all(&copilot_hooks).unwrap();
    fs::write(copilot_hooks.join("rtk-rewrite.json"), "{}").unwrap();
    assert!(is_rtk_hook_configured(home, HarnessKind::Copilot));

    // Codex
    assert!(!is_rtk_hook_configured(home, HarnessKind::Codex));
    let codex_dir = home.join(".codex");
    fs::create_dir_all(&codex_dir).unwrap();
    fs::write(codex_dir.join("RTK.md"), "# RTK").unwrap();
    assert!(is_rtk_hook_configured(home, HarnessKind::Codex));

    // Unsupported harness always returns false
    assert!(!is_rtk_hook_configured(home, HarnessKind::Opencode));
    assert!(!is_rtk_hook_configured(home, HarnessKind::Pi));
}

#[test]
fn test_configure_rtk_hook_unsupported_harness_is_noop() {
    let temp = tempdir().unwrap();
    let home = temp.path();

    for unsupported in [
        HarnessKind::Opencode,
        HarnessKind::Pi,
        HarnessKind::Custom,
        HarnessKind::Deepseek,
        HarnessKind::Grok,
        HarnessKind::Kimi,
        HarnessKind::Agy,
        HarnessKind::Fx,
    ] {
        let res = configure_rtk_hook(home, unsupported, false, true);
        assert!(!res.unwrap());
    }
}

#[test]
fn test_configure_rtk_hook_dry_run() {
    let temp = tempdir().unwrap();
    let home = temp.path();

    let res = configure_rtk_hook(home, HarnessKind::Claude, true, true);
    assert!(res.unwrap());
    // In dry-run, no files or directories should be created
    assert!(!home.join(".claude").exists());
}

#[test]
fn test_unconfigure_rtk_hook_dry_run() {
    let temp = tempdir().unwrap();
    let home = temp.path();

    let res = unconfigure_rtk_hook(home, HarnessKind::Claude, true, true);
    assert!(res.unwrap());
}
