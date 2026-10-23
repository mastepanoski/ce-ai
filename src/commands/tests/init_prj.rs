use super::*;
use tempfile::TempDir;

#[test]
fn test_check_adoption_block_status_file_missing() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("AGENTS.md");
    assert_eq!(
        check_adoption_block_status(&path, AdoptionTier::Full),
        AdoptionBlockStatus::FileMissing
    );
}

#[test]
fn test_check_adoption_block_status_block_missing() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("AGENTS.md");
    fs::write(&path, "# Hello World\n").unwrap();
    assert_eq!(
        check_adoption_block_status(&path, AdoptionTier::Full),
        AdoptionBlockStatus::BlockMissing
    );
}

#[test]
fn test_check_adoption_block_status_stale_version() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("AGENTS.md");
    let content = format!(
        "{} v=1\nOld Content\n{}\n",
        BLOCK_BEGIN_MARKER, BLOCK_END_MARKER
    );
    fs::write(&path, content).unwrap();
    assert_eq!(
        check_adoption_block_status(&path, AdoptionTier::Full),
        AdoptionBlockStatus::StaleVersion { version: 1 }
    );
}

#[test]
fn test_reconcile_project_harness_hooks_upgrades_stale_hooks() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path();

    // 1. Setup Claude with only SessionStart
    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();
    let claude_settings = claude_dir.join("settings.json");
    let old_claude = serde_json::json!({
        "hooks": {
            "SessionStart": [
                { "matcher": ".*", "commands": [{ "type": "command", "command": "ce-ai workflow resume" }] }
            ]
        }
    });
    fs::write(
        &claude_settings,
        serde_json::to_string(&old_claude).unwrap(),
    )
    .unwrap();

    // 2. Setup Cursor with only sessionStart
    let cursor_dir = project_dir.join(".cursor");
    fs::create_dir_all(&cursor_dir).unwrap();
    let cursor_hooks = cursor_dir.join("hooks.json");
    let old_cursor = serde_json::json!({
        "version": 1,
        "hooks": {
            "sessionStart": [
                { "command": "ce-ai workflow resume" }
            ]
        }
    });
    fs::write(&cursor_hooks, serde_json::to_string(&old_cursor).unwrap()).unwrap();

    // 3. Setup Pi with v=1 extension
    let pi_dir = project_dir.join(".pi").join("extensions");
    fs::create_dir_all(&pi_dir).unwrap();
    let pi_ext = pi_dir.join("compound-engineering.ts");
    let old_pi = "// ce-ai:hook v=1\nexport default function() {}";
    fs::write(&pi_ext, old_pi).unwrap();

    // Run reconciliation
    reconcile_project_harness_hooks(project_dir, "Test block").unwrap();

    // Verify Claude was upgraded to have Stop and PreCompact
    let claude_val: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&claude_settings).unwrap()).unwrap();
    assert!(claude_val["hooks"]["Stop"].is_array());
    assert!(claude_val["hooks"]["PreCompact"].is_array());

    // Verify Cursor was upgraded to have stop
    let cursor_val: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&cursor_hooks).unwrap()).unwrap();
    assert!(cursor_val["hooks"]["stop"].is_array());

    // Verify Pi was upgraded to v=2 in ce-ai-companion.ts and legacy compound-engineering.ts was cleaned up
    let companion_ext = pi_dir.join(crate::harness::pi::PI_EXTENSION_FILENAME);
    let pi_content = fs::read_to_string(&companion_ext).unwrap();
    assert!(pi_content.contains("// ce-ai:hook v=2"));
    assert!(pi_content.contains("agent_end"));
    assert!(pi_content.contains("session_before_compact"));
    assert!(!pi_ext.exists());
}

#[test]
fn test_reconcile_rtk_hooks_if_supported_in_dry_run() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path();

    // Create .claude and .cursor in project
    fs::create_dir_all(project_dir.join(".claude")).unwrap();
    fs::create_dir_all(project_dir.join(".cursor")).unwrap();

    let state = State::default();
    let ctx = Context {
        config_dir: tmp.path().join("config"),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        quiet: true,
        dry_run: true,
        verbose: false,
    };

    let res = reconcile_rtk_hooks_if_supported(project_dir, &state, &ctx, false);
    assert!(res.is_ok());
    // Dry-run must not write hook artifacts, even for detected harnesses.
    assert!(!project_dir.join(".claude").join("settings.json").exists());
    assert!(!project_dir.join(".cursor").join("hooks.json").exists());
}

#[test]
fn test_reconcile_rtk_hooks_if_supported_resolves_home_from_ctx() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("project");
    let home_dir = tmp.path().join("home");
    fs::create_dir_all(&project_dir).unwrap();
    fs::create_dir_all(&home_dir).unwrap();

    // Create .claude in project so reconcile triggers for Claude
    fs::create_dir_all(project_dir.join(".claude")).unwrap();

    let state = State::default();
    let ctx = Context {
        config_dir: home_dir.join(".ce-ai"),
        opencode_config_dir: home_dir.join(".config").join("opencode"),
        workspace_root: None,
        quiet: true,
        dry_run: false,
        verbose: false,
    };

    assert_eq!(crate::harness::home_dir_from_ctx(&ctx), home_dir);

    let res = reconcile_rtk_hooks_if_supported(&project_dir, &state, &ctx, false);
    assert!(res.is_ok());

    if crate::harness::rtk::is_rtk_available() {
        assert!(home_dir.join(".claude").exists());
    }
}

#[test]
fn test_reconcile_project_harness_hooks_skips_claude_when_delegating_to_agents() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path();

    fs::write(project_dir.join("AGENTS.md"), "# AGENTS\n").unwrap();
    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();
    let claude_settings = claude_dir.join("settings.json");
    fs::write(&claude_settings, "{}").unwrap();

    let claude_md = project_dir.join("CLAUDE.md");
    fs::write(&claude_md, "@AGENTS.md\n").unwrap();

    reconcile_project_harness_hooks(project_dir, "## Managed Block").unwrap();

    let claude_content = fs::read_to_string(&claude_md).unwrap();
    assert_eq!(claude_content.trim(), "@AGENTS.md");
    assert!(!claude_content.contains("Managed Block"));
}

#[test]
fn test_reconcile_project_harness_hooks_strips_duplicate_claude_block_when_delegating() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path();

    fs::write(project_dir.join("AGENTS.md"), "# AGENTS\n").unwrap();
    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    let claude_md = project_dir.join("CLAUDE.md");
    let duplicate_content = format!(
        "@AGENTS.md\n\n# User Guidelines\nBe careful.\n\n{}\n## Managed Block\n{}\n",
        crate::harness::claude::CE_MANAGED_BEGIN,
        crate::harness::claude::CE_MANAGED_END
    );
    fs::write(&claude_md, duplicate_content).unwrap();

    reconcile_project_harness_hooks(project_dir, "## Managed Block").unwrap();

    let claude_content = fs::read_to_string(&claude_md).unwrap();
    assert!(claude_content.contains("@AGENTS.md"));
    assert!(claude_content.contains("# User Guidelines\nBe careful."));
    assert!(!claude_content.contains(crate::harness::claude::CE_MANAGED_BEGIN));
    assert!(!claude_content.contains("## Managed Block"));
}

#[test]
fn test_reconcile_project_harness_hooks_injects_claude_when_not_delegating() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path();

    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    let claude_md = project_dir.join("CLAUDE.md");
    fs::write(&claude_md, "# Custom Rules\n").unwrap();

    reconcile_project_harness_hooks(project_dir, "## Managed Block").unwrap();

    let claude_content = fs::read_to_string(&claude_md).unwrap();
    assert!(claude_content.contains("# Custom Rules"));
    assert!(claude_content.contains(crate::harness::claude::CE_MANAGED_BEGIN));
    assert!(claude_content.contains("## Managed Block"));
}

#[test]
fn test_render_block_content_includes_self_explaining_pr_directives() {
    let full = render_block_content(AdoptionTier::Full);
    assert!(full.contains("### 🚀 Self-Explaining Pull Requests & Review Readiness"));
    assert!(full.contains("What It Ruled Out"));
    assert!(full.contains("Which Rule Decided It"));
    assert!(full.contains("Upfront Evidence"));
    assert!(full.contains("Pre-Review Automated Checks"));
    assert!(full.contains("Reviewer Routing"));

    let minimal = render_block_content(AdoptionTier::Minimal);
    assert!(minimal.contains("Ensure PRs explain what was ruled out, which rule decided it, and attach verification evidence upfront in collapsible blocks."));

    let orchestrator = render_block_content(AdoptionTier::Orchestrator);
    assert!(orchestrator.contains("Enforce self-explaining PRs: mandate that subagents document rejected alternatives, governing rules, and attach empirical verification evidence in collapsible blocks before requesting review."));
}

#[test]
fn test_render_block_content_turn_0_and_progressive_openspec() {
    let full = render_block_content(AdoptionTier::Full);

    // Turn-0: Hook-aware conditional instructions
    assert!(full.contains("### ⚡ Turn-0 Session Directives (Zero-Step Drift Recovery)"));
    assert!(full.contains("`ce-ai` auto-installs a SessionStart hook on supported harnesses that runs `ce-ai workflow resume --json`"));
    assert!(full.contains("If that state is already present at session start, treat it as current — do NOT re-run the command."));
    assert!(full
        .contains("Only run `ce-ai workflow resume` yourself when no such context was injected"));

    // Stage 2: Living Architecture & Deltas authoring
    assert!(full.contains("### Stage 2 Living Architecture & Deltas (Optional)"));
    assert!(
        full.contains("CE-AI enforces zero semantic authority: architecture deltas are decoupled")
    );
    assert!(full.contains("The execution plan and task checklist reside in `docs/plans/` (`- [x]` checkboxes) as the single source of truth for progress."));

    // Verify tasks.md is NOT listed under the Stage 2 contract bullet points
    let stage2_section = full
        .split("### Stage 2 Living Architecture & Deltas (Optional)")
        .nth(1)
        .unwrap()
        .split("### Single Source of Truth Rule")
        .next()
        .unwrap();
    assert!(!stage2_section.contains("- `tasks.md`:"));
}

#[test]
fn test_init_prj_all_updates_registered_projects() {
    let tmp = TempDir::new().unwrap();
    let prj_a = tmp.path().join("prj_a");
    let prj_b = tmp.path().join("prj_b");
    let config_dir = tmp.path().join("config");
    fs::create_dir_all(&prj_a).unwrap();
    fs::create_dir_all(&prj_b).unwrap();
    fs::create_dir_all(&config_dir).unwrap();

    fs::write(
        prj_a.join("AGENTS.md"),
        "<!-- ce-ai:block begin v=6 tier=full sha256=stale -->\nstale\n<!-- ce-ai:block end -->\n",
    )
    .unwrap();
    fs::write(
        prj_b.join("AGENTS.md"),
        "<!-- ce-ai:block begin v=4 tier=minimal sha256=stale -->\nstale\n<!-- ce-ai:block end -->\n",
    )
    .unwrap();

    let mut state = State::new();
    state.projects.push(ProjectAdoptionEntry {
        path: prj_a.clone(),
        file: "AGENTS.md".into(),
        tier: AdoptionTier::Full,
        block_version: 6,
        block_sha256: "stale".into(),
        created_file: false,
        adopted_at: "2026-09-01T00:00:00Z".into(),
    });
    state.projects.push(ProjectAdoptionEntry {
        path: prj_b.clone(),
        file: "AGENTS.md".into(),
        tier: AdoptionTier::Minimal,
        block_version: 4,
        block_sha256: "stale".into(),
        created_file: false,
        adopted_at: "2026-09-01T00:00:00Z".into(),
    });
    state.save(&config_dir.join("state.json")).unwrap();

    let ctx = Context {
        config_dir: config_dir.clone(),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        quiet: true,
        dry_run: false,
        verbose: false,
    };

    let res = run(
        &ctx,
        None,
        InitPrjOptions {
            all: true,
            force: true,
            ..Default::default()
        },
    );
    assert!(res.is_ok());

    let content_a = fs::read_to_string(prj_a.join("AGENTS.md")).unwrap();
    assert!(content_a.contains(&format!("v={BLOCK_VERSION}")));
    assert!(content_a.contains("tier=full"));

    let content_b = fs::read_to_string(prj_b.join("AGENTS.md")).unwrap();
    assert!(content_b.contains(&format!("v={BLOCK_VERSION}")));
    assert!(content_b.contains("tier=minimal"));

    let updated_state = State::load(&config_dir.join("state.json")).unwrap();
    assert_eq!(updated_state.projects.len(), 2);
    assert_eq!(updated_state.projects[0].block_version, BLOCK_VERSION);
    assert_eq!(updated_state.projects[0].tier, AdoptionTier::Full);
    assert_eq!(updated_state.projects[1].block_version, BLOCK_VERSION);
    assert_eq!(updated_state.projects[1].tier, AdoptionTier::Minimal);
}

#[test]
fn test_scan_legacy_directives_identifies_all_patterns() {
    let legacy_text = r#"# AGENTS.md
6. **OpenSpec Required**: NO code changes without formal spec in `openspec/changes/<feature_name>/`.
5. **Mandatory OpenSpec Before Code Changes**:
   - NO code changes or feature implementations shall be made without creating or updating a formal spec under `openspec/changes/<feature_name>/` containing `proposal.md`, `exploration.md`, `design.md`, `spec.md`, and `tasks.md`.
[Stage 2: OpenSpec Definition (MANDATORY)]
### Stage 2 OpenSpec Enforcement Requirements
├── models.rs      # ce-ai models set/list/profile implementation
├── decisions.rs   # ce-ai decisions route model advisor
so the repository FSM is left at ✓ Ready (100%) with zero unarchived change warnings before concluding
"#;

    let findings = scan_legacy_directives(legacy_text);
    assert_eq!(findings.len(), 8);
    assert!(findings.iter().any(|f| f.pattern_id == "PAT-INV-6"));
    assert!(findings
        .iter()
        .any(|f| f.pattern_id == "PAT-CONSTRAINT-OPENSPEC"));
    assert!(findings.iter().any(|f| f.pattern_id == "PAT-FSM-7"));
    assert!(findings.iter().any(|f| f.pattern_id == "PAT-STAGE-2"));
    assert!(findings.iter().any(|f| f.pattern_id == "PAT-CLI-MODELS"));
    assert!(findings.iter().any(|f| f.pattern_id == "PAT-CLI-DECISIONS"));
    assert!(findings.iter().any(|f| f.pattern_id == "PAT-FSM-READY"));
}

#[test]
fn test_migrate_legacy_directives_preserves_custom_domain_rules() {
    let text = r#"# Custom Project
This is our unique custom architecture.

## Custom Invariants
1. Do not break database migrations.
6. **OpenSpec Required**: NO code changes without formal spec in `openspec/changes/<feature_name>/`.

## Development Cycle
[Stage 1: Ideation (ce-brainstorm)] ➔ [Stage 2: OpenSpec Definition (MANDATORY)] ➔ [Stage 3: Execution Plan (ce-plan)]

### Stage 2 OpenSpec Enforcement Requirements
Before creating any PR or writing feature code, agents MUST verify that `openspec/changes/<feature_name>/` contains all five files:

### Single Source of Truth Rule
Ideation artifacts are disposable inputs.

### Custom Project Linters
Run `npm test` before committing.
"#;

    let (migrated, findings) = migrate_legacy_directives(text);
    assert!(!findings.is_empty());
    // Custom domain content preserved
    assert!(migrated.contains("This is our unique custom architecture."));
    assert!(migrated.contains("1. Do not break database migrations."));
    assert!(migrated.contains("### Custom Project Linters"));
    assert!(migrated.contains("Run `npm test` before committing."));

    // Legacy directives replaced
    assert!(!migrated.contains("OpenSpec Required: NO code changes"));
    assert!(migrated.contains("No Semantic Authority & Optional OpenSpec"));
    assert!(!migrated.contains("[Stage 2: OpenSpec Definition (MANDATORY)]"));
    assert!(migrated.contains("[Stage 2: OpenSpec Definition (OPTIONAL)]"));
    assert!(!migrated.contains("Stage 2 OpenSpec Enforcement Requirements"));
    assert!(migrated.contains("Stage 2 OpenSpec Integration (Optional)"));
    assert!(migrated.contains("Proactive OpenSpec Triage & Guidance for Developers"));
}

#[test]
fn test_adopt_project_with_migrate_creates_backup_and_upgrades() {
    let tmp = TempDir::new().unwrap();
    let prj_dir = tmp.path().join("legacy-project");
    let config_dir = tmp.path().join("config");
    fs::create_dir_all(&prj_dir).unwrap();
    fs::create_dir_all(&config_dir).unwrap();

    let legacy_content = "# Project Notes\n\n6. **OpenSpec Required**: NO code changes without formal spec in `openspec/changes/<feature_name>/`.\n";
    fs::write(prj_dir.join("AGENTS.md"), legacy_content).unwrap();

    let ctx = Context {
        config_dir: config_dir.clone(),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        quiet: true,
        dry_run: false,
        verbose: false,
    };

    let res = adopt_project(
        &ctx,
        &prj_dir,
        AdoptionTier::Full,
        false,
        true, // migrate = true
        false,
        false,
    );
    assert!(res.is_ok());

    // Backup created
    let backup_path = prj_dir.join("AGENTS.md.bak");
    assert!(backup_path.exists());
    assert_eq!(fs::read_to_string(&backup_path).unwrap(), legacy_content);

    // File modernized and managed block injected
    let modernized = fs::read_to_string(prj_dir.join("AGENTS.md")).unwrap();
    assert!(modernized.contains("No Semantic Authority & Optional OpenSpec"));
    assert!(modernized.contains(&format!("v={BLOCK_VERSION}")));
}

#[test]
fn test_adopt_project_with_migrate_dry_run_leaves_disk_untouched() {
    let tmp = TempDir::new().unwrap();
    let prj_dir = tmp.path().join("dry-run-legacy");
    let config_dir = tmp.path().join("config");
    fs::create_dir_all(&prj_dir).unwrap();
    fs::create_dir_all(&config_dir).unwrap();

    let legacy_content = "# Project Notes\n\n6. **OpenSpec Required**: NO code changes without formal spec in `openspec/changes/<feature_name>/`.\n";
    fs::write(prj_dir.join("AGENTS.md"), legacy_content).unwrap();

    let ctx = Context {
        config_dir: config_dir.clone(),
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        quiet: true,
        dry_run: true, // dry_run = true
        verbose: false,
    };

    let res = adopt_project(
        &ctx,
        &prj_dir,
        AdoptionTier::Full,
        false,
        true, // migrate = true
        false,
        false,
    );
    assert!(res.is_ok());

    // Backup NOT created on dry-run
    assert!(!prj_dir.join("AGENTS.md.bak").exists());

    // File unchanged on disk
    assert_eq!(
        fs::read_to_string(prj_dir.join("AGENTS.md")).unwrap(),
        legacy_content
    );
}

#[test]
fn test_migrate_arch_dry_run() {
    let tmp = TempDir::new().unwrap();
    let prj_dir = tmp.path().join("dry-run-arch");
    let config_dir = tmp.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let openspec_dir = prj_dir.join("openspec");
    fs::create_dir_all(openspec_dir.join("specs")).unwrap();
    fs::create_dir_all(
        openspec_dir
            .join("changes")
            .join("archive")
            .join("old-feat"),
    )
    .unwrap();
    fs::write(openspec_dir.join("specs").join("core.md"), "# Core Spec").unwrap();
    fs::write(
        openspec_dir
            .join("changes")
            .join("archive")
            .join("old-feat")
            .join("tasks.md"),
        "- [x] Done\n",
    )
    .unwrap();

    let ctx = Context {
        config_dir,
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        quiet: true,
        dry_run: true,
        verbose: false,
    };

    let options = InitPrjOptions {
        migrate_arch: true,
        force: true,
        ..Default::default()
    };

    let res = run(&ctx, Some(prj_dir.clone()), options);
    assert!(res.is_ok());

    // Legacy openspec still exists, modern docs/arch NOT created
    assert!(openspec_dir.join("specs").join("core.md").exists());
    assert!(!prj_dir.join("docs").join("arch").exists());
}

#[test]
fn test_migrate_arch_execution() {
    let tmp = TempDir::new().unwrap();
    let prj_dir = tmp.path().join("exec-arch");
    let config_dir = tmp.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let openspec_dir = prj_dir.join("openspec");
    fs::create_dir_all(openspec_dir.join("specs")).unwrap();
    fs::create_dir_all(
        openspec_dir
            .join("changes")
            .join("archive")
            .join("old-feat"),
    )
    .unwrap();
    fs::create_dir_all(openspec_dir.join("changes").join("active-feat")).unwrap();
    fs::write(openspec_dir.join("specs").join("core.md"), "# Core Spec").unwrap();
    fs::write(
        openspec_dir
            .join("changes")
            .join("archive")
            .join("old-feat")
            .join("tasks.md"),
        "- [x] Done\n",
    )
    .unwrap();
    fs::write(
        openspec_dir
            .join("changes")
            .join("active-feat")
            .join("proposal.md"),
        "# Active Proposal\n",
    )
    .unwrap();

    let ctx = Context {
        config_dir,
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        quiet: true,
        dry_run: false,
        verbose: false,
    };

    let options = InitPrjOptions {
        migrate_arch: true,
        force: true,
        ..Default::default()
    };

    let res = run(&ctx, Some(prj_dir.clone()), options);
    assert!(res.is_ok());

    // Destination docs/arch structure populated
    let arch_dir = prj_dir.join("docs").join("arch");
    assert!(arch_dir.join("system").join("core.md").exists());
    assert_eq!(
        fs::read_to_string(arch_dir.join("system").join("core.md")).unwrap(),
        "# Core Spec"
    );
    assert!(arch_dir
        .join("archive")
        .join("old-feat")
        .join("tasks.md")
        .exists());
    assert!(arch_dir
        .join("deltas")
        .join("active-feat")
        .join("proposal.md")
        .exists());

    // Legacy openspec root removed
    assert!(!openspec_dir.exists());
}

#[test]
fn test_migrate_arch_updates_agents_block() {
    let tmp = TempDir::new().unwrap();
    let prj_dir = tmp.path().join("block-arch");
    let config_dir = tmp.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let openspec_dir = prj_dir.join("openspec");
    fs::create_dir_all(&openspec_dir).unwrap();

    let ctx = Context {
        config_dir,
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        quiet: true,
        dry_run: false,
        verbose: false,
    };

    let options = InitPrjOptions {
        migrate_arch: true,
        force: true,
        tier: Some("full".to_string()),
        ..Default::default()
    };

    let res = run(&ctx, Some(prj_dir.clone()), options);
    assert!(res.is_ok());

    let agents_content = fs::read_to_string(prj_dir.join("AGENTS.md")).unwrap();
    assert!(agents_content.contains(&format!("v={BLOCK_VERSION}")));
    assert!(agents_content.contains("Stage 2 Living Architecture & Deltas (Optional)"));
    assert!(agents_content.contains("docs/arch/system/"));
}

#[test]
fn test_migrate_arch_aborts_on_dirty_working_tree() {
    let tmp = TempDir::new().unwrap();
    let prj_dir = tmp.path().join("dirty-arch");
    let config_dir = tmp.path().join("config");
    fs::create_dir_all(&prj_dir).unwrap();
    fs::create_dir_all(&config_dir).unwrap();

    // Initialize real git repo
    let run_git = |args: &[&str]| {
        let mut cmd = std::process::Command::new("git");
        for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
            cmd.env_remove(var);
        }
        cmd.args(args).current_dir(&prj_dir).output().unwrap()
    };
    run_git(&["init"]);
    run_git(&["config", "user.email", "test@example.com"]);
    run_git(&["config", "user.name", "Test"]);

    let openspec_dir = prj_dir.join("openspec");
    fs::create_dir_all(openspec_dir.join("specs")).unwrap();
    fs::write(openspec_dir.join("specs").join("core.md"), "# Core").unwrap();

    run_git(&["add", "."]);
    run_git(&["commit", "-m", "Initial commit"]);

    // Create an uncommitted dirty file
    fs::write(prj_dir.join("dirty.txt"), "uncommitted changes").unwrap();

    let ctx = Context {
        config_dir,
        opencode_config_dir: tmp.path().join("opencode"),
        workspace_root: None,
        quiet: true,
        dry_run: false,
        verbose: false,
    };

    // Without --force: must fail with Usage error
    let options = InitPrjOptions {
        migrate_arch: true,
        force: false,
        ..Default::default()
    };
    let res = run(&ctx, Some(prj_dir.clone()), options);
    assert!(res.is_err());
    match res {
        Err(CeError::Usage(msg)) => assert!(msg.contains("dirty")),
        other => panic!(
            "expected CeError::Usage with dirty tree message, got {:?}",
            other
        ),
    }

    // With --force: must succeed
    let options_force = InitPrjOptions {
        migrate_arch: true,
        force: true,
        ..Default::default()
    };
    let res_force = run(&ctx, Some(prj_dir.clone()), options_force);
    assert!(res_force.is_ok());
    assert!(!openspec_dir.exists());
    assert!(prj_dir
        .join("docs")
        .join("arch")
        .join("system")
        .join("core.md")
        .exists());
}
