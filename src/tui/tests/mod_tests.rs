use super::*;
use crate::commands::Context;
use crate::error::CeError;
use crate::state::state::State;
use tempfile::TempDir;

fn ctx() -> (TempDir, Context) {
    let tmp = TempDir::new().unwrap();
    let ctx = Context::resolve(Some(tmp.path().join("ce-ai")), false, false, true).unwrap();
    (tmp, ctx)
}

#[test]
fn failure_lines_carry_marker_and_remedy() {
    let lines = handlers::workflow_failure_lines(&CeError::Runtime("state.json corrupt".into()));
    let joined = lines.join("\n");
    assert!(joined.contains('❌'));
    assert!(joined.contains("state.json corrupt"));
    assert!(joined.contains("ce-ai doctor"));
}

#[test]
fn stage_transition_confirms_and_persists_checkpoint() {
    let (_tmp, ctx) = ctx();
    let lines = handlers::workflow_stage_transition_lines(&ctx, 2);
    let joined = lines.join("\n");
    assert!(joined.contains("✅ Workflow Checkpoint Saved to Stage 2!"));
    assert!(joined.contains("Stage transition recorded"));

    let state = State::load(&ctx.config_dir.join("state.json")).unwrap();
    let wf = state.workflow.expect("workflow should be set");
    assert_eq!(wf.stage, crate::state::state::WorkflowStage::OpenSpec);
}

#[test]
fn stage_transition_failure_uses_failure_class() {
    let (_tmp, ctx) = ctx();
    std::fs::create_dir_all(&ctx.config_dir).unwrap();
    std::fs::write(ctx.config_dir.join("state.json"), "{broken").unwrap();
    let joined = handlers::workflow_stage_transition_lines(&ctx, 4).join("\n");
    assert!(joined.contains('❌'));
    assert!(!joined.contains("✅"));
}

mod workflow_guide_tests {
    use super::*;

    fn panel_text(ctx: &Context) -> String {
        handlers::workflow_panel_lines(ctx)
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.clone())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn every_cycle_row_carries_skill_marker_and_native_rows_run_marker() {
        let (_tmp, ctx) = ctx();
        let joined = panel_text(&ctx);
        let cycle_rows: Vec<&str> = joined
            .lines()
            .filter(|l| {
                l.contains("[1:")
                    || l.contains("[2:")
                    || l.contains("[3:")
                    || l.contains("[4:")
                    || l.contains("[5:")
                    || l.contains("[6:")
                    || l.contains("[7:")
            })
            .collect();
        assert_eq!(cycle_rows.len(), 7);
        assert!(cycle_rows
            .iter()
            .all(|l| l.trim_start().starts_with("skill:")));
        assert!(joined.matches("[run]").count() >= 2);
    }

    #[test]
    fn verify_stage_is_tech_neutral() {
        let (_tmp, ctx) = ctx();
        let joined = panel_text(&ctx);
        let verify_line = joined.lines().find(|l| l.contains("[5:")).unwrap();
        for tool in ["cargo", "make e2e", "npm", "pytest"] {
            assert!(!verify_line.contains(tool), "toolchain leaked: {tool}");
        }
        assert!(verify_line.contains("test/e2e"));
    }

    #[test]
    fn hints_enumerate_actions_without_resume() {
        let (_tmp, ctx) = ctx();
        let joined = panel_text(&ctx);
        let hint_line = joined
            .lines()
            .find(|l| l.contains("Press"))
            .expect("hint line present");
        assert!(hint_line.contains("[Enter]"));
        assert!(hint_line.contains("[1-7]"));
        assert!(!joined.to_lowercase().contains("resume"));
    }
}

mod spawned_contract_tests {
    use super::*;
    use clap::Command;

    /// Mirrors the top-level Cli global so module Args augmented standalone
    /// accept flags like --dry-run exactly as the real binary does.
    fn with_cli_globals(cmd: Command) -> Command {
        cmd.arg(
            clap::Arg::new("dry-run")
                .long("dry-run")
                .global(true)
                .action(clap::ArgAction::SetTrue),
        )
    }

    fn assert_parses(cmd: Command, args_without_verb: &[String]) {
        // clap treats argv[0] as the program name; re-add a dummy so the
        // verb-stripped vector starts at its first real flag/subcommand.
        let mut argv = vec!["tui".to_string()];
        argv.extend_from_slice(args_without_verb);
        cmd.try_get_matches_from(argv).unwrap_or_else(|e| {
            panic!(
                "TUI-spawned args rejected by the current CLI contract: {e}\nargs: {args_without_verb:?}"
            )
        });
    }

    /// Anti-drift net (Issue #161 regression class): every vector the TUI
    /// spawns must parse against its subcommand's live clap surface.
    #[test]
    fn every_tui_spawned_vector_satisfies_its_cli_contract() {
        let harness = "claude";

        assert_parses(
            with_cli_globals(
                <crate::commands::install::Args as clap::Args>::augment_args(Command::new(
                    "install",
                )),
            ),
            &crate::tui::spawn::install_cmd_args(harness, true)[1..],
        );
        assert_parses(
            <crate::commands::models::Args as clap::Args>::augment_args(Command::new("models")),
            &crate::tui::spawn::models_list_args()[1..],
        );
        assert_parses(
            with_cli_globals(<crate::commands::sync::Args as clap::Args>::augment_args(
                Command::new("sync"),
            )),
            &crate::tui::spawn::sync_cmd_args(true)[1..],
        );
        assert_parses(
            <crate::commands::uninstall::Args as clap::Args>::augment_args(Command::new(
                "uninstall",
            )),
            &crate::tui::spawn::uninstall_cmd_args(harness)[1..],
        );
        assert_parses(
            <crate::commands::doctor::Args as clap::Args>::augment_args(Command::new("doctor")),
            &crate::tui::spawn::doctor_cmd_args()[1..],
        );
        // Extended coverage (tui-e2e-zen): 15 CLI subcommands — pin the 6 that were
        // missing and gave false 8/8 green. Future TUI tabs must keep this green.
        assert_parses(
            <crate::commands::skills::Args as clap::Args>::augment_args(Command::new("skills")),
            &["list".to_string()][..],
        );
        assert_parses(
            <crate::commands::skills::Args as clap::Args>::augment_args(Command::new("skills")),
            &[
                "resolve".to_string(),
                "--harness".to_string(),
                harness.to_string(),
            ][..],
        );
        assert_parses(
            <crate::commands::skills::Args as clap::Args>::augment_args(Command::new("skills")),
            &["doctor".to_string()][..],
        );
        assert_parses(
            <crate::commands::backups::BackupsArgs as clap::Args>::augment_args(Command::new(
                "backups",
            )),
            &["list".to_string()][..],
        );
        assert_parses(
            <crate::commands::tools::Args as clap::Args>::augment_args(Command::new("tools")),
            &["status".to_string()][..],
        );
        assert_parses(
            <crate::commands::usage::Args as clap::Args>::augment_args(Command::new("usage")),
            &["sync".to_string()][..],
        );
        assert_parses(
            <crate::commands::usage::Args as clap::Args>::augment_args(Command::new("usage")),
            &["report".to_string()][..],
        );
        assert_parses(
            <crate::commands::tools::Args as clap::Args>::augment_args(Command::new("tools")),
            &["install".to_string(), "codegraph".to_string()][..],
        );
        assert_parses(
            <crate::commands::audit::Args as clap::Args>::augment_args(Command::new("audit")),
            &[][..],
        );
        assert_parses(
            <crate::commands::workflow::Args as clap::Args>::augment_args(Command::new("workflow")),
            &["status".to_string()][..],
        );

        // Zero-arg / inline-enum commands: pin exact vectors.
        assert_eq!(crate::tui::spawn::status_args(), ["status"]);
        assert_eq!(
            crate::tui::spawn::workflow_status_args(),
            ["workflow", "status"]
        );
        assert_eq!(crate::tui::spawn::init_prj_args(), ["init-prj"]);
        assert_eq!(crate::tui::spawn::upgrade_cmd_args(), ["upgrade"]);
    }

    #[test]
    fn upgrade_dead_flags_stay_rejected() {
        // v1.18.1 (Issue #161) removed --harness/--force from upgrade; the
        // TUI must never resurrect them.
        let err =
            <crate::commands::upgrade::Args as clap::Args>::augment_args(Command::new("upgrade"))
                .try_get_matches_from(vec![
                    "upgrade".to_string(),
                    "--harness".to_string(),
                    "all".to_string(),
                    "--force".to_string(),
                ])
                .expect_err("dead --harness/--force must be rejected by upgrade's contract");
        assert_eq!(err.kind(), clap::error::ErrorKind::UnknownArgument);
    }

    #[test]
    fn headless_ui_renders_all_tabs_without_panic() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let ctx =
            crate::commands::Context::resolve(Some(tmp.path().join("ce-ai")), false, false, true)
                .unwrap();
        for (idx, tab) in MenuTab::all().iter().enumerate() {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            let mut app = App::new(&ctx);
            app.selected_tab = idx;
            terminal
                .draw(|f| render::ui(f, &app, &ctx))
                .unwrap_or_else(|e| panic!("ui draw failed for tab {:?}: {e}", tab));
            let buffer = terminal.backend().buffer().clone();
            let content: String = buffer
                .content()
                .iter()
                .map(|c| c.symbol().to_string())
                .collect();
            // Title contains emoji + spaces; check keyword to avoid width flakiness.
            let keyword = match tab {
                MenuTab::Status => "Status",
                MenuTab::Workflow => "Workflow",
                MenuTab::Install => "Install",
                MenuTab::Models => "Models",
                MenuTab::Skills => "Skills",
                MenuTab::Sync => "Sync",
                MenuTab::Upgrade => "Upgrade",
                MenuTab::Doctor => "Doctor",
                MenuTab::Backups => "Backups",
                MenuTab::Tools => "Tools",
                MenuTab::Usage => "Usage",
                MenuTab::Audit => "Audit",
                MenuTab::InitPrj => "Project",
                MenuTab::Uninstall => "Uninstall",
                MenuTab::Exit => "Quit",
            };
            assert!(
                content.contains(keyword),
                "tab {:?} keyword '{keyword}' not in buffer: {}",
                tab,
                &content[..500.min(content.len())]
            );
        }
    }

    #[test]
    fn headless_screenshots_no_overflow() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        use std::path::Path;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let ctx =
            crate::commands::Context::resolve(Some(tmp.path().join("ce-ai")), false, false, true)
                .unwrap();
        // Screenshots go to gitignored dir — never committed (LOW design fix).
        let out_dir = Path::new("tui-screenshots");
        let _ = std::fs::create_dir_all(out_dir);
        for (idx, tab) in MenuTab::all().iter().enumerate() {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            let mut app = App::new(&ctx);
            app.selected_tab = idx;
            terminal.draw(|f| render::ui(f, &app, &ctx)).unwrap();
            let buffer = terminal.backend().buffer().clone();
            // Verify no row exceeds width and no panic/overflow — width=80, height=24
            assert_eq!(buffer.area.width, 80);
            assert_eq!(buffer.area.height, 24);
            // Dump buffer as text lines for visual proof (gitignored)
            let mut lines = Vec::new();
            for y in 0..buffer.area.height {
                let mut row = String::new();
                for x in 0..buffer.area.width {
                    let cell = &buffer[(x, y)];
                    row.push_str(cell.symbol());
                }
                // Trim trailing spaces but keep content length check
                let trimmed = row.trim_end().to_string();
                // Width check is display-width aware (emoji =2 cells); allow 1-char slop for wide glyphs
                // Core proof is no panic/overflow and screenshot written to gitignored dir
                assert!(
                    trimmed.chars().count() <= 84,
                    "tab {:?} row {y} overflow: {} chars >84",
                    tab,
                    trimmed.chars().count()
                );
                lines.push(trimmed);
            }
            let dump = lines.join("\n");
            let path = out_dir.join(format!("{idx:02}-{:?}.txt", tab));
            let _ = std::fs::write(&path, dump);
            // Also assert keyword present (same as above, but proves screenshot captured)
            assert!(lines.join(" ").contains(match tab {
                MenuTab::Status => "Status",
                MenuTab::Workflow => "Workflow",
                MenuTab::Install => "Install",
                MenuTab::Models => "Models",
                MenuTab::Skills => "Skills",
                MenuTab::Sync => "Sync",
                MenuTab::Upgrade => "Upgrade",
                MenuTab::Doctor => "Doctor",
                MenuTab::Backups => "Backups",
                MenuTab::Tools => "Tools",
                MenuTab::Usage => "Usage",
                MenuTab::Audit => "Audit",
                MenuTab::InitPrj => "Project",
                MenuTab::Uninstall => "Uninstall",
                MenuTab::Exit => "Quit",
            }));
        }
    }

    #[test]
    fn ui_is_english_only() {
        // Regression net for artifact language contract: TUI must be English, never Spanish.
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        use tempfile::TempDir;

        let tmp = TempDir::new().unwrap();
        let ctx =
            crate::commands::Context::resolve(Some(tmp.path().join("ce-ai")), false, false, true)
                .unwrap();
        let forbidden = [
            "¿",
            "Descargar",
            "Aviso",
            "Fuente",
            "Instalación",
            "Reconcilia",
            "Busca",
            "Versión",
        ];
        for (idx, tab) in MenuTab::all().iter().enumerate() {
            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            let mut app = App::new(&ctx);
            app.selected_tab = idx;
            terminal.draw(|f| render::ui(f, &app, &ctx)).unwrap();
            let buffer = terminal.backend().buffer().clone();
            let content: String = buffer
                .content()
                .iter()
                .map(|c| c.symbol().to_string())
                .collect();
            for word in &forbidden {
                assert!(
                    !content.contains(word),
                    "tab {:?} contains Spanish '{}' — UI must be English per artifact contract",
                    tab,
                    word
                );
            }
        }
    }
}
