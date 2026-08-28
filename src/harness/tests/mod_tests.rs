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
