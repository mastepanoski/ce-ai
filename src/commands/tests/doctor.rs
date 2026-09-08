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

#[test]
fn test_doctor_rtk_probe_resolves_home_from_ctx() {
    let tmp = TempDir::new().unwrap();
    let home_dir = tmp.path().join("home");
    let ctx = Context {
        config_dir: home_dir.join(".ce-ai"),
        opencode_config_dir: home_dir.join(".config").join("opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    std::fs::create_dir_all(&ctx.config_dir).unwrap();
    std::fs::create_dir_all(&ctx.opencode_config_dir).unwrap();
    std::fs::create_dir_all(&home_dir).unwrap();

    assert_eq!(crate::harness::home_dir_from_ctx(&ctx), home_dir);

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

    let manifest = InstallManifest {
        version: "1.0.0".into(),
        plugin_name: "compound-engineering".into(),
        installed_at: "2026-08-22T00:00:00Z".into(),
        source: serde_json::json!({"type": "local"}),
        files: vec![],
        config_mutations: vec![],
    };
    manifest.write(&ctx.opencode_config_dir).unwrap();

    // Pre-configure RTK hook inside the isolated home_dir
    let claude_dir = home_dir.join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(
        claude_dir.join("settings.json"),
        r#"{"hooks":{"PreToolUse":[{"matcher":"Bash","hooks":[{"command":"rtk hook claude"}]}]}}"#,
    )
    .unwrap();

    let args = Args { strict: true };
    let res = run(&ctx, &args);
    // In strict mode, doctor fails on missing optional companion tools, but it must NOT
    // report rtk-hook-missing because the hook was properly detected in home_dir_from_ctx.
    if crate::harness::rtk::is_rtk_available() {
        if let Err(CeError::Runtime(msg)) = res {
            assert!(
                !msg.contains("rtk-hook-missing"),
                "doctor should recognize hook configured in home_dir_from_ctx, got: {msg}"
            );
        }
    }
}

#[test]
fn test_doctor_code_review_mid_tier_note_matrix() {
    let mut state = State::new();
    let empty_config = serde_json::json!({});

    // 1. Neither configured -> None
    assert!(
        crate::commands::models::check_code_review_mid_tier_note(&state, &empty_config).is_none()
    );

    // 2. ce-code-review in config, mid-tier unset -> Some(note)
    let config_review_only = serde_json::json!({
        "agent": {
            "ce-code-review": { "model": "anthropic/claude-sonnet-4-5" }
        }
    });
    let note =
        crate::commands::models::check_code_review_mid_tier_note(&state, &config_review_only);
    assert!(note.is_some());
    let note_str = note.unwrap();
    assert!(note_str.contains("ce-code-review-mid-tier"));
    assert!(note_str.contains("persona-level cost tiering has no explicit target on opencode"));
    assert!(note_str
        .contains("ce-ai models set --harness opencode ce-code-review-mid-tier <provider/model>"));

    // 3. ce-code-review in state, mid-tier unset -> Some(note)
    state.set_model_assignment("ce-code-review", "anthropic", "claude-sonnet-4-5");
    let note_from_state =
        crate::commands::models::check_code_review_mid_tier_note(&state, &empty_config);
    assert!(note_from_state.is_some());

    // 4. Both configured in config -> None
    let config_both = serde_json::json!({
        "agent": {
            "ce-code-review": { "model": "anthropic/claude-sonnet-4-5" },
            "ce-code-review-mid-tier": { "model": "anthropic/claude-haiku-3-5" }
        }
    });
    let mut state_clean = State::new();
    assert!(
        crate::commands::models::check_code_review_mid_tier_note(&state_clean, &config_both)
            .is_none()
    );

    // 5. Both configured across state and config -> None
    state_clean.set_model_assignment("ce-code-review-mid-tier", "anthropic", "claude-haiku-3-5");
    assert!(crate::commands::models::check_code_review_mid_tier_note(
        &state_clean,
        &config_review_only
    )
    .is_none());

    // 6. Only mid-tier configured -> None
    let config_mid_tier_only = serde_json::json!({
        "agent": {
            "ce-code-review-mid-tier": { "model": "anthropic/claude-haiku-3-5" }
        }
    });
    let clean_state2 = State::new();
    assert!(crate::commands::models::check_code_review_mid_tier_note(
        &clean_state2,
        &config_mid_tier_only
    )
    .is_none());
}

#[test]
fn test_doctor_runs_cleanly_with_mid_tier_note() {
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
    state.set_model_assignment("ce-code-review", "anthropic", "claude-sonnet-4-5");
    state.save(&ctx.config_dir.join("state.json")).unwrap();

    let manifest = InstallManifest {
        version: "1.0.0".into(),
        plugin_name: "compound-engineering".into(),
        installed_at: "2026-08-22T00:00:00Z".into(),
        source: serde_json::json!({"type": "local"}),
        files: vec![],
        config_mutations: vec![],
    };
    manifest.write(&ctx.opencode_config_dir).unwrap();

    // SessionStart plugin properly configured
    crate::opencode::plugins::ensure_session_start_plugin(&ctx.opencode_config_dir).unwrap();

    // opencode.json matches state for ce-code-review so no drift finding occurs
    let mut config = read_config(&ctx.opencode_config_dir.join("opencode.json")).unwrap();
    config["agent"] = serde_json::json!({
        "ce-code-review": { "model": "anthropic/claude-sonnet-4-5" }
    });
    crate::state::write_atomic(
        &ctx.opencode_config_dir.join("opencode.json"),
        &serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();

    let args = Args::default();
    let res = run(&ctx, &args);
    // Doctor must succeed (exit 0) because mid-tier note is informational, not a blocking finding!
    assert!(
        res.is_ok(),
        "doctor should pass with Ok(()), got: {:?}",
        res
    );
}

#[test]
fn test_doctor_claude_marketplace_divergence_is_non_blocking() {
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let claude_dir = home.join(".claude");
    let plugins_dir = claude_dir.join("plugins");
    std::fs::create_dir_all(&plugins_dir).unwrap();
    let plugins_file = plugins_dir.join("installed_plugins.json");
    std::fs::write(
        &plugins_file,
        serde_json::json!({
            "version": 2,
            "plugins": {
                "compound-engineering@compound-engineering-plugin": [{
                    "scope": "user",
                    "version": "3.8.4"
                }]
            }
        })
        .to_string(),
    )
    .unwrap();

    let ctx = Context {
        config_dir: tmp.path().join("config"),
        opencode_config_dir: home.join(".config").join("opencode"),
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
        "version": "compound-engineering-v3.24.0",
        "scope": "global",
        "installed_at": "2026-08-22T00:00:00Z"
    }));
    state.save(&ctx.config_dir.join("state.json")).unwrap();

    // Doctor must succeed (exit 0) because marketplace divergence is informational, not a blocking finding!
    let res = run(&ctx, &Args::default());
    assert!(
        res.is_ok(),
        "doctor should pass with Ok(()), got: {:?}",
        res
    );
}
