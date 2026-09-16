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

#[test]
fn test_doctor_kimi_marketplace_divergence_and_orphan_are_non_blocking() {
    // Shield against parallel adapter tests that mutate KIMI_CODE_HOME:
    // hold the harness env lock and force the default (home-derived) paths.
    let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
    let saved_kimi_home = std::env::var_os("KIMI_CODE_HOME");
    std::env::remove_var("KIMI_CODE_HOME");
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let kimi_dir = home.join(".kimi-code");

    // Native plugin manager: enabled compound-engineering at 3.14.3.
    let root = kimi_dir
        .join("plugins")
        .join("managed")
        .join("compound-engineering");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"compound-engineering","version":"3.14.3"}"#,
    )
    .unwrap();
    let plugins_dir = kimi_dir.join("plugins");
    std::fs::create_dir_all(&plugins_dir).unwrap();
    std::fs::write(
        plugins_dir.join("installed.json"),
        serde_json::json!({
            "version": 1,
            "plugins": [
                { "id": "compound-engineering", "root": root, "enabled": true }
            ]
        })
        .to_string(),
    )
    .unwrap();

    // Orphan ce-ai managed tree (install-manifest marker, unreferenced config.toml).
    let managed_dir = kimi_dir.join("compound-engineering");
    std::fs::create_dir_all(&managed_dir).unwrap();
    std::fs::write(managed_dir.join("install-manifest.json"), "{}").unwrap();
    std::fs::write(kimi_dir.join("config.toml"), "extra_skill_dirs = []\n").unwrap();

    std::env::set_var("KIMI_CODE_HOME", &kimi_dir);

    let ctx = Context {
        config_dir: home.join(".ce-ai"),
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
        "name": "kimi",
        "version": "compound-engineering-v3.24.0",
        "scope": "global",
        "installed_at": "2026-08-22T00:00:00Z"
    }));
    state.save(&ctx.config_dir.join("state.json")).unwrap();

    // Doctor must succeed (exit 0): divergence is doctor-info, orphan tree is doctor-warn.
    let res = run(&ctx, &Args::default());
    assert!(
        res.is_ok(),
        "doctor should pass with Ok(()), got: {:?}",
        res
    );

    if let Some(v) = saved_kimi_home {
        std::env::set_var("KIMI_CODE_HOME", v);
    }
}

#[test]
fn test_doctor_detects_claude_manifest_drift() {
    // Shield against parallel adapter tests that mutate CLAUDE_CONFIG_DIR:
    // hold the harness env lock and force the default (home-derived) paths.
    let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
    let saved_claude_dir = std::env::var_os("CLAUDE_CONFIG_DIR");
    std::env::remove_var("CLAUDE_CONFIG_DIR");
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let claude_dir = home.join(".claude");

    let managed = claude_dir.join("compound-engineering");
    std::fs::create_dir_all(managed.join("plugins")).unwrap();
    std::fs::write(managed.join("plugins/compound-engineering.js"), b"pristine").unwrap();
    InstallManifest {
        version: "compound-engineering-v3.24.0".into(),
        plugin_name: "compound-engineering".into(),
        installed_at: "2026-08-22T00:00:00Z".into(),
        source: serde_json::json!({"kind": "local"}),
        files: vec![crate::opencode::manifest::ManifestFile {
            path: "plugins/compound-engineering.js".into(),
            sha256: crate::state::diff::sha256_hex(b"pristine"),
        }],
        config_mutations: vec![],
    }
    .write(&claude_dir)
    .unwrap();

    // External drift: tamper with the managed file after manifest write.
    std::fs::write(managed.join("plugins/compound-engineering.js"), b"tampered").unwrap();

    let ctx = Context {
        config_dir: home.join(".ce-ai"),
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

    let res = run(&ctx, &Args::default());
    assert!(
        res.is_err(),
        "tampered claude managed file must produce a doctor finding"
    );
    let findings = non_opencode_diff_findings(&home);
    assert!(
        findings
            .iter()
            .any(|f| f == "diff: claude modified plugins/compound-engineering.js"),
        "expected claude drift finding, got: {findings:?}"
    );

    if let Some(v) = saved_claude_dir {
        std::env::set_var("CLAUDE_CONFIG_DIR", v);
    }
}

#[test]
fn test_doctor_ignores_absent_claude_manifest() {
    // Shield against parallel adapter tests that mutate CLAUDE_CONFIG_DIR:
    // hold the harness env lock and force the default (home-derived) paths.
    let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
    let saved_claude_dir = std::env::var_os("CLAUDE_CONFIG_DIR");
    std::env::remove_var("CLAUDE_CONFIG_DIR");
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let claude_dir = home.join(".claude");
    // claude dir exists (harness installed) but has NO managed manifest.
    std::fs::create_dir_all(&claude_dir).unwrap();

    let ctx = Context {
        config_dir: home.join(".ce-ai"),
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

    // Doctor may report unrelated findings; this test only pins that an
    // absent claude manifest never yields claude diff findings.
    let _ = run(&ctx, &Args::default());
    let findings = non_opencode_diff_findings(&home);
    assert!(
        findings.iter().all(|f| !f.contains("diff: claude")),
        "absent claude manifest must not produce findings, got: {findings:?}"
    );

    if let Some(v) = saved_claude_dir {
        std::env::set_var("CLAUDE_CONFIG_DIR", v);
    }
}

#[test]
fn test_probe_manifest_drift_count_includes_claude_and_kimi() {
    // Shield against parallel adapter tests that mutate CLAUDE_CONFIG_DIR /
    // KIMI_CODE_HOME: hold the harness env lock and force the default
    // (home-derived) paths.
    let _guard = crate::harness::tests::HARNESS_ENV_LOCK.lock().unwrap();
    let saved_claude_dir = std::env::var_os("CLAUDE_CONFIG_DIR");
    let saved_kimi_home = std::env::var_os("KIMI_CODE_HOME");
    std::env::remove_var("CLAUDE_CONFIG_DIR");
    std::env::remove_var("KIMI_CODE_HOME");
    let tmp = TempDir::new().unwrap();
    let home = tmp.path().join("home");
    let claude_dir = home.join(".claude");
    let kimi_dir = home.join(".kimi-code");

    // claude: one tampered managed file (real manifest entry).
    let claude_managed = claude_dir.join("compound-engineering");
    std::fs::create_dir_all(&claude_managed).unwrap();
    std::fs::write(claude_managed.join("SKILL.md"), b"tampered").unwrap();
    InstallManifest {
        version: "v1.0.0".into(),
        plugin_name: "compound-engineering".into(),
        installed_at: "2026-08-22T00:00:00Z".into(),
        source: serde_json::json!({"kind": "local"}),
        files: vec![crate::opencode::manifest::ManifestFile {
            path: "SKILL.md".into(),
            sha256: crate::state::diff::sha256_hex(b"pristine"),
        }],
        config_mutations: vec![],
    }
    .write(&claude_dir)
    .unwrap();

    // kimi: empty manifest -> contributes zero (graceful degradation).
    let kimi_managed = kimi_dir.join("compound-engineering");
    std::fs::create_dir_all(&kimi_managed).unwrap();
    InstallManifest {
        version: "v1.0.0".into(),
        plugin_name: "compound-engineering".into(),
        installed_at: "2026-08-22T00:00:00Z".into(),
        source: serde_json::json!({"kind": "local"}),
        files: vec![],
        config_mutations: vec![],
    }
    .write(&kimi_dir)
    .unwrap();

    let ctx = Context {
        config_dir: home.join(".ce-ai"),
        opencode_config_dir: home.join(".config").join("opencode"),
        workspace_root: None,
        dry_run: false,
        verbose: false,
        quiet: true,
    };

    let count = crate::commands::workflow::probe_manifest_drift_count(&ctx);
    assert_eq!(count, 1, "only the tampered claude file counts as drift");

    if let Some(v) = saved_claude_dir {
        std::env::set_var("CLAUDE_CONFIG_DIR", v);
    }
    if let Some(v) = saved_kimi_home {
        std::env::set_var("KIMI_CODE_HOME", v);
    }
}

#[test]
fn test_doctor_warns_on_mtime_fallback_and_uncommitted_spec() {
    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path().join("repo");
    let config_dir = tmp.path().join("config");
    let opencode_dir = tmp.path().join("opencode");
    std::fs::create_dir_all(&repo_root).unwrap();
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::create_dir_all(&opencode_dir).unwrap();

    std::fs::write(
        config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();

    let change_dir = repo_root.join("openspec").join("changes").join("feat-doc");
    std::fs::create_dir_all(&change_dir).unwrap();
    std::fs::write(change_dir.join("proposal.md"), "# Proposal").unwrap();
    std::fs::write(change_dir.join("spec.md"), "# Spec").unwrap();

    let ctx = Context {
        config_dir,
        opencode_config_dir: opencode_dir,
        workspace_root: Some(repo_root.clone()),
        dry_run: false,
        verbose: false,
        quiet: false,
    };

    let args = Args::default();
    let res = run(&ctx, &args);
    assert!(res.is_ok());
}

#[test]
fn test_doctor_reports_gate_check_telemetry() {
    use crate::commands::gate::{log_gate_event, GateDecision, GateEventRecord};

    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path().join("repo");
    let config_dir = tmp.path().join("config");
    let opencode_dir = tmp.path().join("opencode");
    std::fs::create_dir_all(&repo_root).unwrap();
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::create_dir_all(&opencode_dir).unwrap();

    std::fs::write(
        config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();

    let record = GateEventRecord {
        timestamp: "2026-09-09T12:00:00Z".to_string(),
        harness: "claude".to_string(),
        tool: "Write".to_string(),
        path: "src/main.rs".to_string(),
        workspace: repo_root.display().to_string(),
        branch: Some("feat/foo".to_string()),
        stage: Some(4),
        feature: Some("foo".to_string()),
        decision: GateDecision::WouldBlock,
        edge_case: None,
        reason: "missing proposal.md".to_string(),
    };
    log_gate_event(&config_dir, &record).unwrap();

    let ctx = Context {
        config_dir,
        opencode_config_dir: opencode_dir,
        workspace_root: Some(repo_root.clone()),
        dry_run: false,
        verbose: false,
        quiet: false,
    };

    let args = Args::default();
    let res = run(&ctx, &args);
    assert!(res.is_ok());
}

#[test]
fn test_doctor_reports_gate_check_telemetry_with_blocked() {
    use crate::commands::gate::{log_gate_event, GateDecision, GateEventRecord};

    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path().join("repo");
    let config_dir = tmp.path().join("config");
    let opencode_dir = tmp.path().join("opencode");
    std::fs::create_dir_all(&repo_root).unwrap();
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::create_dir_all(&opencode_dir).unwrap();

    std::fs::write(
        config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();

    let record = GateEventRecord {
        timestamp: "2026-09-09T12:00:00Z".to_string(),
        harness: "claude".to_string(),
        tool: "Write".to_string(),
        path: "src/main.rs".to_string(),
        workspace: repo_root.display().to_string(),
        branch: Some("feat/foo".to_string()),
        stage: Some(4),
        feature: Some("foo".to_string()),
        decision: GateDecision::Blocked,
        edge_case: None,
        reason: "missing proposal.md, spec.md".to_string(),
    };
    log_gate_event(&config_dir, &record).unwrap();

    let ctx = Context {
        config_dir,
        opencode_config_dir: opencode_dir,
        workspace_root: Some(repo_root.clone()),
        dry_run: false,
        verbose: false,
        quiet: false,
    };

    let args = Args::default();
    let res = run(&ctx, &args);
    assert!(res.is_ok());
}

#[test]
fn test_doctor_surfaces_blocked_gate_receipt() {
    use crate::state::state::{
        GateDecision, GateReceipt, WorkflowSource, WorkflowStage, WorkflowState,
    };

    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path().join("repo");
    let config_dir = tmp.path().join("config");
    let opencode_dir = tmp.path().join("opencode");
    std::fs::create_dir_all(&repo_root).unwrap();
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::create_dir_all(&opencode_dir).unwrap();

    std::fs::write(
        config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();

    let mut state = State::new();
    let wf = WorkflowState {
        stage: WorkflowStage::WorkTdd,
        task: "ce-work".to_string(),
        feature_name: Some("test-feat".to_string()),
        updated_at: "2026-09-12T00:00:00Z".to_string(),
        source: WorkflowSource::default(),
        resolution: None,
        new_cycle: false,
        execution_mode: None,
    };
    state
        .workflows
        .insert(State::workspace_branch_key(&repo_root, None), wf);
    state.gate_receipts.insert(
        "test-feat".to_string(),
        GateReceipt {
            timestamp: "2026-09-12T00:00:00Z".to_string(),
            feature: "test-feat".to_string(),
            target_path: "src/foo.rs".to_string(),
            decision: GateDecision::Blocked,
            stage: Some(4),
            entry_point: None,
            tier: "full".to_string(),
            missing_artifacts: vec!["proposal.md".to_string(), "spec.md".to_string()],
            reason: "blocked".to_string(),
        },
    );
    state.save(&config_dir.join("state.json")).unwrap();

    let ctx = Context {
        config_dir,
        opencode_config_dir: opencode_dir,
        workspace_root: Some(repo_root.clone()),
        dry_run: false,
        verbose: false,
        quiet: false,
    };

    let args = Args::default();
    let res = run(&ctx, &args);
    assert!(res.is_ok());
}

#[test]
fn test_doctor_stays_ok_with_ship_readiness_gaps() {
    // Issue #354: a workspace with commits ahead but no Stage 6 artifact and no
    // code-review receipt must emit non-fatal warnings, never fatal findings.
    let tmp = TempDir::new().unwrap();
    let repo_root = tmp.path().join("repo");
    let config_dir = tmp.path().join("config");
    let opencode_dir = tmp.path().join("opencode");
    std::fs::create_dir_all(&repo_root).unwrap();
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::create_dir_all(&opencode_dir).unwrap();

    std::fs::write(
        config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();

    let git = |args: &[&str]| {
        let mut cmd = std::process::Command::new("git");
        for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
            cmd.env_remove(var);
        }
        cmd.args(args).current_dir(&repo_root).output().unwrap();
    };
    git(&["init", "-b", "main"]);
    std::fs::write(repo_root.join("README.md"), "base\n").unwrap();
    git(&["add", "."]);
    git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.com",
        "commit",
        "-m",
        "base",
    ]);
    git(&["checkout", "-b", "feat/foo"]);
    std::fs::write(repo_root.join("src.rs"), "fn main() {}\n").unwrap();
    git(&["add", "."]);
    git(&[
        "-c",
        "user.name=Test",
        "-c",
        "user.email=test@example.com",
        "commit",
        "-m",
        "feat",
    ]);

    let ctx = Context {
        config_dir,
        opencode_config_dir: opencode_dir,
        workspace_root: Some(repo_root.clone()),
        dry_run: false,
        verbose: false,
        quiet: false,
    };

    let args = Args::default();
    let res = run(&ctx, &args);
    assert!(
        res.is_ok(),
        "ship-readiness gaps must remain non-fatal, got: {res:?}"
    );
}

#[test]
fn test_doctor_emits_doc_debt_warnings_non_fatal() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // 1. Create OpenSpec change with desync
    let feat_dir = root
        .join("openspec")
        .join("changes")
        .join("stranded-change");
    std::fs::create_dir_all(&feat_dir).unwrap();
    std::fs::write(
        feat_dir.join("tasks.md"),
        "- [x] 1. Parent complete\n  - [ ] 1.1 Open subtask\n",
    )
    .unwrap();

    // 2. Create solution with dead path and missing frontmatter
    let sol_dir = root.join("docs").join("solutions").join("architecture");
    std::fs::create_dir_all(&sol_dir).unwrap();
    let broken_sol = r#"---
title: "Solution with Dead Path"
category: "architecture"
problem_type: "design"
tags:
  - doc
---
References `src/ghost_module.rs`.
"#;
    std::fs::write(sol_dir.join("old.md"), broken_sol).unwrap();

    let ctx = Context {
        config_dir: root.join("config"),
        opencode_config_dir: root.join("opencode"),
        workspace_root: Some(root.to_path_buf()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    std::fs::create_dir_all(&ctx.config_dir).unwrap();
    std::fs::create_dir_all(&ctx.opencode_config_dir).unwrap();
    let state = State::new();
    state.save(&ctx.config_dir.join("state.json")).unwrap();

    std::fs::write(
        ctx.config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();

    let args = Args::default();
    let res = run(&ctx, &args);
    assert!(
        res.is_ok(),
        "doctor must exit 0 with doc debt warnings, got: {res:?}"
    );
}

#[test]
fn test_doctor_warns_on_archive_compaction_threshold() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create 3 archive packages with threshold = 2
    let archive_dir = root.join("openspec").join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir).unwrap();
    for i in 1..=3 {
        let pkg = archive_dir.join(format!("2026-08-0{i}-pkg-{i}"));
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::write(pkg.join("tasks.md"), "- [x] 1. Done\n").unwrap();
    }

    let ctx = Context {
        config_dir: root.join("config"),
        opencode_config_dir: root.join("opencode"),
        workspace_root: Some(root.to_path_buf()),
        dry_run: false,
        verbose: false,
        quiet: true,
    };
    std::fs::create_dir_all(&ctx.config_dir).unwrap();
    std::fs::create_dir_all(&ctx.opencode_config_dir).unwrap();

    let mut state = State::new();
    state.doc_hygiene = Some(crate::state::state::DocHygieneConfig {
        stale_spec_days: 21,
        check_solution_paths: true,
        require_solution_frontmatter: true,
        archive_compaction_threshold: 2,
    });
    state.save(&ctx.config_dir.join("state.json")).unwrap();

    std::fs::write(
        ctx.config_dir.join("skills-registry.json"),
        r#"{"version":"1.6.3","updated_at":"2026-08-22T00:00:00Z","skills":[]}"#,
    )
    .unwrap();

    let args = Args::default();
    let res = run(&ctx, &args);
    assert!(
        res.is_ok(),
        "doctor must exit 0 with compaction warning, got: {res:?}"
    );
}
