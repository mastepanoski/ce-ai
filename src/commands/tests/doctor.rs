use super::*;
use tempfile::TempDir;

#[test]
fn test_doctor_strict_flag_default() {
    let args = Args::default();
    assert!(!args.strict);
}

#[test]
fn test_doctor_runs_on_clean_context() {
    let tmp = TempDir::new().unwrap();
    let ctx = Context {
        config_dir: tmp.path().to_path_buf(),
        opencode_config_dir: tmp.path().to_path_buf(),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    std::fs::write(
        ctx.config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();
    let args = Args::default();
    assert!(run(&ctx, &args).is_ok());
}

#[test]
fn test_doctor_detects_real_manifest_state_inconsistency() {
    let tmp = TempDir::new().unwrap();
    let ctx = Context {
        config_dir: tmp.path().join("config"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    std::fs::create_dir_all(&ctx.config_dir).unwrap();
    std::fs::create_dir_all(&ctx.opencode_config_dir).unwrap();
    std::fs::write(
        ctx.config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();

    let mut state = State::new();
    state.installed_harnesses.push(serde_json::json!({
        "name": "opencode",
        "version": "1.0.0",
        "scope": "global",
        "installed_at": "2026-08-22T00:00:00Z"
    }));
    state.save(&ctx.config_dir.join("state.json")).unwrap();

    // Manifest was deleted or never created under opencode_config_dir
    let args = Args::default();
    let res = run(&ctx, &args);
    assert!(res.is_err());
    if let Err(CeError::Runtime(err)) = res {
        assert!(err.contains("doctor found"));
    } else {
        panic!("expected Runtime error with doctor findings");
    }
}

#[test]
fn test_doctor_detects_workspace_manifest_inconsistency() {
    let tmp = TempDir::new().unwrap();
    let ws_dir = tmp.path().join("workspace");
    let ctx = Context {
        config_dir: tmp.path().join("config"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: Some(ws_dir.clone()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    std::fs::create_dir_all(&ctx.config_dir).unwrap();
    std::fs::create_dir_all(&ctx.opencode_config_dir).unwrap();
    std::fs::create_dir_all(&ws_dir).unwrap();
    std::fs::write(
        ctx.config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();

    let mut state = State::new();
    state.installed_harnesses.push(serde_json::json!({
        "name": "opencode",
        "version": "1.0.0",
        "scope": "workspace",
        "target_dir": ws_dir.display().to_string(),
        "installed_at": "2026-08-22T00:00:00Z"
    }));
    state.save(&ctx.config_dir.join("state.json")).unwrap();

    let args = Args::default();
    let res = run(&ctx, &args);
    assert!(res.is_err());
    if let Err(CeError::Runtime(err)) = res {
        assert!(err.contains("doctor found"));
    } else {
        panic!("expected Runtime error with doctor findings");
    }
}

#[test]
fn test_context_resolve_opencode_dir() {
    let tmp = TempDir::new().unwrap();
    let ws_dir = tmp.path().join("workspace");
    let ctx = Context {
        config_dir: tmp.path().join("config"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: Some(ws_dir.clone()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let mut state = State::new();
    // 1. Without workspace entry, defaults to opencode_config_dir
    assert_eq!(ctx.resolve_opencode_dir(&state), ctx.opencode_config_dir);

    // 2. With workspace entry matching target_dir, resolves to ws_dir
    state.installed_harnesses.push(serde_json::json!({
        "name": "opencode",
        "version": "1.0.0",
        "scope": "workspace",
        "target_dir": ws_dir.display().to_string(),
    }));
    assert_eq!(ctx.resolve_opencode_dir(&state), ws_dir);
}

mod branch_protection_tests {
    use super::github_slug_from_url;

    #[test]
    fn github_slug_parses_ssh_https_and_rejects_other_hosts() {
        assert_eq!(
            github_slug_from_url("git@github.com:mastepanoski/ce-ai.git").as_deref(),
            Some("mastepanoski/ce-ai")
        );
        assert_eq!(
            github_slug_from_url("https://github.com/mastepanoski/ce-ai/").as_deref(),
            Some("mastepanoski/ce-ai")
        );
        assert_eq!(
            github_slug_from_url("https://gitlab.com/group/proj.git"),
            None
        );
    }
}

#[test]
fn test_doctor_detects_missing_rtk_hook_in_strict_mode() {
    let tmp = TempDir::new().unwrap();
    let ctx = Context {
        config_dir: tmp.path().join("config"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    std::fs::create_dir_all(&ctx.config_dir).unwrap();
    std::fs::create_dir_all(&ctx.opencode_config_dir).unwrap();
    std::fs::write(
        ctx.config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();

    let mut state = State::new();
    state.installed_harnesses.push(serde_json::json!({
        "name": "claude",
        "version": "1.0.0",
        "scope": "global",
        "installed_at": "2026-08-22T00:00:00Z"
    }));
    state.save(&ctx.config_dir.join("state.json")).unwrap();

    // In strict mode, if RTK hook is missing for claude, doctor flags it
    let args = Args { strict: true };
    let res = run(&ctx, &args);
    // Since claude hook or manifest is missing, strict doctor returns finding
    assert!(res.is_err());
}
