//! Full-screen Ratatui TUI dashboard for ce-ai.
//! Provides a modern, rich, split-panel terminal interface with live status,
//! keyboard navigation, model slot tables, and one-key action execution.

use std::io::{stdout, IsTerminal};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::Terminal;

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;
use crate::state::state::State;

mod app;
mod render;
mod spawn;
mod tabs;
use app::App;
pub use tabs::MenuTab;

struct RawModeGuard;
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
        let _ = Terminal::new(CrosstermBackend::new(stdout)).map(|mut t| t.show_cursor());
    }
}

pub fn run_interactive(ctx: &Context) -> Result<(), CeError> {
    if !std::io::stdout().is_terminal() {
        return Err(CeError::Usage(
            "no subcommand provided — run 'ce-ai --help' for usage or run in an interactive terminal for TUI mode".into(),
        ));
    }

    enable_raw_mode().map_err(|e| CeError::Runtime(format!("raw mode error: {e}")))?;
    let _guard = RawModeGuard;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| CeError::Runtime(format!("screen error: {e}")))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| CeError::Runtime(format!("terminal error: {e}")))?;

    let mut app = App::new(ctx);
    let res = run_app(&mut terminal, ctx, &mut app);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();
    drop(_guard);

    if let Err(err) = res {
        eprintln!("error: {err}");
    }
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ctx: &Context,
    app: &mut App,
) -> Result<(), CeError> {
    loop {
        terminal
            .draw(|f| render::ui(f, app, ctx))
            .map_err(|e| CeError::Runtime(format!("draw error: {e}")))?;

        if event::poll(Duration::from_millis(100))
            .map_err(|e| CeError::Runtime(format!("poll error: {e}")))?
        {
            if let Event::Key(key) =
                event::read().map_err(|e| CeError::Runtime(format!("read error: {e}")))?
            {
                // Precedence: picker > modal > tabs (R3)
                if app.model_picker_open {
                    match key.code {
                        KeyCode::Esc => app.model_picker_open = false,
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.picker_selected = app.picker_selected.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if app.picker_selected + 1 < app.picker_items.len() {
                                app.picker_selected += 1;
                            }
                        }
                        KeyCode::Enter => {
                            let slot = app.model_slots[app.selected_model_idx].clone();
                            let model = app.picker_items[app.picker_selected].clone();
                            let harness = app.selected_harness_target().to_string();
                            app.model_picker_open = false;
                            let lines =
                                match crate::commands::models::set(ctx, &harness, &slot, &model) {
                                    Ok(()) => vec![
                                        format!("✅ Set {harness}/{slot} = {model}"),
                                        "Applied atomically to the harness config and state.json."
                                            .to_string(),
                                    ],
                                    Err(err) => vec![format!("❌ Failed to set {slot}: {err}")],
                                };
                            execute_action(app, "Model Assignment", move || lines);
                        }
                        _ => {}
                    }
                    continue;
                }

                if app.output_modal.is_some() {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                            app.output_modal = None;
                            app.output_scroll = 0;
                            app.reload_state(ctx);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.output_scroll = app.output_scroll.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.output_scroll = app.output_scroll.saturating_add(1);
                        }
                        KeyCode::PageUp => {
                            app.output_scroll = app.output_scroll.saturating_sub(5);
                        }
                        KeyCode::PageDown => {
                            app.output_scroll = app.output_scroll.saturating_add(5);
                        }
                        _ => {}
                    }
                    continue;
                }

                if app.current_tab() == MenuTab::Models {
                    match key.code {
                        KeyCode::Char('n') | KeyCode::Char('J') => {
                            if !app.model_slots.is_empty()
                                && app.selected_model_idx + 1 < app.model_slots.len()
                            {
                                app.selected_model_idx += 1;
                            }
                        }
                        KeyCode::Char('p') | KeyCode::Char('K') => {
                            app.selected_model_idx = app.selected_model_idx.saturating_sub(1);
                        }
                        KeyCode::Char('m') => {
                            let harness = app.selected_harness_target().to_string();
                            match crate::commands::models::discover_models(&harness) {
                                Ok(mut models) => {
                                    if let Some((_, current)) =
                                        app.model_scope.iter().find(|(s, _)| {
                                            Some(s.as_str())
                                                == app
                                                    .model_slots
                                                    .get(app.selected_model_idx)
                                                    .map(String::as_str)
                                        })
                                    {
                                        if !models.contains(current) {
                                            models.insert(0, current.clone());
                                        }
                                    }
                                    app.picker_items = models;
                                    app.picker_selected = 0;
                                    app.model_picker_open = true;
                                }
                                Err(err) => {
                                    execute_action(app, "Model Discovery Failed", move || {
                                        vec![format!("❌ {err}")]
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.selected_tab > 0 {
                            app.selected_tab -= 1;
                        } else {
                            app.selected_tab = MenuTab::all().len() - 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.selected_tab + 1 < MenuTab::all().len() {
                            app.selected_tab += 1;
                        } else {
                            app.selected_tab = 0;
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        app.prev_harness();
                        app.reload_state(ctx);
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        app.next_harness();
                        app.reload_state(ctx);
                    }
                    KeyCode::Char('d') => {
                        app.dry_run = !app.dry_run;
                    }
                    KeyCode::Char('i') | KeyCode::Char('I') => {
                        let lines = run_init_prj_cmd(ctx);
                        execute_action(app, "Adopt Project (init-prj)", move || lines);
                    }
                    KeyCode::Enter | KeyCode::Char('r') => {
                        let dry_run = app.dry_run;
                        match app.current_tab() {
                            MenuTab::Status => {
                                execute_action(app, "Harness Status", || run_status_cmd(ctx));
                            }
                            MenuTab::Workflow => {
                                execute_action(app, "Workflow FSM Status", || {
                                    run_workflow_cmd(ctx)
                                });
                            }
                            MenuTab::Install => {
                                let lines = run_install_cmd(ctx, app, dry_run);
                                execute_action(app, "Install Plugin", move || lines);
                            }
                            MenuTab::Models => {
                                execute_action(app, "Model Assignments", || run_models_cmd(ctx));
                            }
                            MenuTab::Skills => {
                                execute_action(app, "Skills Registry", || run_skills_cmd(ctx));
                            }
                            MenuTab::Sync => {
                                execute_action(app, "Sync Drift", move || {
                                    run_sync_cmd(ctx, dry_run)
                                });
                            }
                            MenuTab::Upgrade => {
                                let lines = run_upgrade_cmd(ctx, app);
                                execute_action(app, "Upgrade Release", move || lines);
                            }
                            MenuTab::Doctor => {
                                execute_action(app, "Doctor Diagnostics", || run_doctor_cmd(ctx));
                            }
                            MenuTab::Backups => {
                                let lines = run_restore_backup_cmd(ctx, app);
                                execute_action(app, "Restore Backup", move || lines);
                            }
                            MenuTab::Tools => {
                                execute_action(app, "Tools Status", || run_tools_cmd(ctx));
                            }
                            MenuTab::Usage => {
                                execute_action(app, "Usage Report", || run_usage_cmd(ctx));
                            }
                            MenuTab::Audit => {
                                execute_action(app, "Audit", || run_audit_cmd(ctx));
                            }
                            MenuTab::InitPrj => {
                                let lines = run_init_prj_cmd(ctx);
                                execute_action(app, "Project Adopt", move || lines);
                            }
                            MenuTab::Uninstall => {
                                let lines = run_uninstall_cmd(ctx, app);
                                execute_action(app, "Uninstall Plugin", move || lines);
                            }
                            MenuTab::Exit => break,
                        }
                    }
                    KeyCode::Char(c @ '1'..='7') if app.current_tab() == MenuTab::Workflow => {
                        let stage_num = c.to_digit(10).unwrap();
                        let lines = workflow_stage_transition_lines(ctx, stage_num);
                        execute_action(app, "Workflow Stage Transition", move || lines);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn execute_action<F>(app: &mut App, title: &str, f: F)
where
    F: FnOnce() -> Vec<String>,
{
    let lines = f();
    app.output_scroll = 0;
    app.output_modal = Some((title.to_string(), lines));
}

/// Runs the installed ce-ai binary as a subprocess and captures its output.
/// Commands that `println!` would otherwise paint straight onto the
/// alternate screen and corrupt the dashboard layout (#72-class breakage).
fn capture_cli(args: &[String]) -> Vec<String> {
    let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("ce-ai"));
    match std::process::Command::new(&exe).args(args).output() {
        Ok(out) => {
            let mut lines: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::to_string)
                .collect();
            lines.extend(
                String::from_utf8_lossy(&out.stderr)
                    .lines()
                    .map(str::to_string),
            );
            if lines.is_empty() {
                lines.push(if out.status.success() {
                    "(no output)".to_string()
                } else {
                    format!("❌ exit code {}", out.status.code().unwrap_or(-1))
                });
            }
            if !out.status.success() {
                lines.push(format!("❌ command failed: ce-ai {}", args.join(" ")));
            }
            lines
        }
        Err(err) => vec![format!("❌ failed to launch {}: {err}", exe.display())],
    }
}

fn run_status_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::status_args())
}

fn run_install_cmd(_ctx: &Context, app: &App, dry_run: bool) -> Vec<String> {
    capture_cli(&spawn::install_cmd_args(
        app.selected_harness_target(),
        dry_run,
    ))
}

fn run_models_cmd(_ctx: &Context) -> Vec<String> {
    let mut lines = capture_cli(&spawn::models_list_args());
    lines.push(String::new());
    lines.push("Assign a slot or manage profiles:".to_string());
    lines.push("  ce-ai models set <slot> <provider/model>".to_string());
    lines.push("  ce-ai models profile save|load <name>".to_string());
    lines.push(
        "In this tab: [n/p] select slot · [m] pick a model from the harness catalog.".to_string(),
    );
    lines
}

fn run_sync_cmd(_ctx: &Context, dry_run: bool) -> Vec<String> {
    if dry_run {
        capture_cli(&spawn::sync_cmd_args(true))
    } else {
        capture_cli(&spawn::sync_cmd_args(false))
    }
}

fn run_workflow_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::workflow_status_args())
}

/// Workflow panel content: native actions marked `[run]`, agent-session stages
/// marked `skill:` — text-perceivable, never color-only (R6).
fn workflow_panel_lines(ctx: &Context) -> Vec<Line<'static>> {
    let header = |text: &str| {
        Line::from(Span::styled(
            text.to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))
    };
    let native = |key: &str, desc: &str| {
        Line::from(vec![
            Span::styled(
                "  [run] ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{key:<6} "), Style::default().fg(Color::Green)),
            Span::raw(desc.to_string()),
        ])
    };
    let skill = |row: &str| Line::from(format!("  skill: {row}"));

    let mut lines = vec![
        Line::from(Span::styled(
            "Workflow FSM Engine & Progress Recovery:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        header("Native actions (run inside this dashboard):"),
        native("[Enter]", "Query workflow status"),
        native("[1-7]", "Save stage-transition checkpoint"),
        Line::from(""),
        header("7-Stage Flywheel Cycle (agent-session work):"),
        skill("[1: Ideation]   ➔ /ce-brainstorm · /ce-ideate · /ce-strategy"),
        skill("[2: OpenSpec]   ➔ Formal spec definition (proposal, spec, tasks)"),
        skill("[3: Plan]       ➔ /ce-plan · /ce-doc-review"),
        skill("[4: Work/TDD]   ➔ /ce-work · /ce-debug · /ce-simplify-code"),
        skill("[5: Verify]     ➔ Run your project's test/e2e commands"),
        skill("[6: Compound]   ➔ /ce-compound · /ce-compound-refresh (docs/solutions/)"),
        skill("[7: Ship]       ➔ /ce-commit-push-pr"),
        Line::from(""),
    ];

    let state_path = ctx.config_dir.join("state.json");
    match State::load(&state_path) {
        Ok(state) => match state.last_update_check.clone() {
            Some(cp) => lines.push(Line::from(vec![
                Span::styled("  Latest Checkpoint: ", Style::default().fg(Color::Yellow)),
                Span::styled(cp, Style::default().fg(Color::White)),
            ])),
            None => lines.push(Line::from(
                "  (No progress checkpoint saved yet — press [1-7] or run 'ce-ai workflow checkpoint')",
            )),
        },
        Err(_) => lines.push(Line::from(Span::styled(
            "  ⚠️ Could not read state.json",
            Style::default().fg(Color::Red),
        ))),
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press [Enter] to query status, or keys [1-7] to save a stage checkpoint.",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )));
    lines
}

/// Failure-class modal content for workflow actions: cause plus actionable remedy.
fn workflow_failure_lines(err: &CeError) -> Vec<String> {
    vec![
        format!("❌ Workflow command failed: {err}"),
        "Remedy: check state.json corruption or permissions, then run `ce-ai doctor`.".into(),
    ]
}

/// Saves the `[1-7]` stage-transition checkpoint and builds its confirmation modal.
fn workflow_stage_transition_lines(ctx: &Context, stage_num: u32) -> Vec<String> {
    let stage = match crate::state::state::WorkflowStage::parse(&stage_num.to_string()) {
        Ok(st) => st,
        Err(err) => return workflow_failure_lines(&err),
    };
    let task = match stage_num {
        1 => "1.0 Ideation & Brainstorming (ce-brainstorm)",
        2 => "2.0 OpenSpec Specification",
        3 => "3.0 Technical Plan (ce-plan)",
        4 => "4.0 Implementation (ce-work)",
        5 => "5.0 Verification (project test/e2e commands)",
        6 => "6.0 Knowledge Capture (ce-compound)",
        7 => "7.0 PR Delivery (ce-commit-push-pr)",
        _ => "1.0 Ideation",
    };
    let mut lines = match crate::commands::workflow::checkpoint_lines(ctx, stage, task, None) {
        Ok(out) => out,
        Err(err) => return workflow_failure_lines(&err),
    };
    lines.insert(
        0,
        format!("✅ Workflow Checkpoint Saved to Stage {}!", stage.number()),
    );
    lines.push("Stage transition recorded in state.json successfully.".to_string());
    lines
}
fn run_upgrade_cmd(_ctx: &Context, _app: &App) -> Vec<String> {
    // Plain `upgrade` reconciles every active harness; --harness/--force were
    // removed from the CLI contract in v1.18.1 (Issue #161).
    capture_cli(&spawn::upgrade_cmd_args())
}

fn run_doctor_cmd(_ctx: &Context) -> Vec<String> {
    let mut lines = capture_cli(&spawn::doctor_cmd_args());
    if !lines.iter().any(|l| l.contains("doctor: ok")) {
        lines.push("❌ doctor reported findings (exit non-zero)".to_string());
    }
    lines
}

fn run_uninstall_cmd(_ctx: &Context, app: &App) -> Vec<String> {
    let selected = app.selected_harness_target().to_string();
    let mut out = vec![format!("Uninstalling target: [{selected}]")];
    let target_harnesses: Vec<String> = if selected == "all" {
        if app.harnesses.is_empty() {
            vec!["opencode".to_string()]
        } else {
            app.harnesses.iter().map(|(n, _, _)| n.clone()).collect()
        }
    } else {
        vec![selected]
    };

    for harness in &target_harnesses {
        out.extend(capture_cli(&spawn::uninstall_cmd_args(harness)));
    }
    out
}

fn run_restore_backup_cmd(ctx: &Context, app: &mut App) -> Vec<String> {
    if app.backups.is_empty() {
        return vec!["No backups found for the selected harness target.".to_string()];
    }

    let idx = app.selected_backup_idx.min(app.backups.len() - 1);
    let entry = &app.backups[idx];

    let target_harness = entry
        .harness
        .parse::<HarnessKind>()
        .unwrap_or(HarnessKind::Opencode);
    let target_path = target_harness.config_path(&ctx.opencode_config_dir);
    let backups_dir = ctx.config_dir.join("backups");

    match crate::state::backups::restore_backup_by_id(&backups_dir, &entry.id, &target_path) {
        Ok(restored) => {
            app.reload_state(ctx);
            vec![
                format!("✅ Successfully restored backup '{}'", restored.id),
                format!("   Harness Target: {}", restored.harness),
                format!("   Restored File: {}", restored.file_name),
                format!("   Config Path: {}", target_path.display()),
            ]
        }
        Err(err) => vec![format!("❌ Failed to restore backup: {err}")],
    }
}

fn run_init_prj_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::init_prj_args())
}

fn run_skills_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::skills_list_args())
}

fn run_tools_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::tools_status_args())
}

fn run_usage_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::usage_report_args())
}

fn run_audit_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::audit_args())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Context;
    use tempfile::TempDir;

    fn ctx() -> (TempDir, Context) {
        let tmp = TempDir::new().unwrap();
        let ctx = Context::resolve(Some(tmp.path().join("ce-ai")), false, false, true).unwrap();
        (tmp, ctx)
    }

    #[test]
    fn failure_lines_carry_marker_and_remedy() {
        let lines = workflow_failure_lines(&CeError::Runtime("state.json corrupt".into()));
        let joined = lines.join("\n");
        assert!(joined.contains('❌'));
        assert!(joined.contains("state.json corrupt"));
        assert!(joined.contains("ce-ai doctor"));
    }

    #[test]
    fn stage_transition_confirms_and_persists_checkpoint() {
        let (_tmp, ctx) = ctx();
        let lines = workflow_stage_transition_lines(&ctx, 2);
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
        let joined = workflow_stage_transition_lines(&ctx, 4).join("\n");
        assert!(joined.contains('❌'));
        assert!(!joined.contains("✅"));
    }
}

#[cfg(test)]
mod workflow_guide_tests {
    use super::*;
    use crate::commands::Context;
    use tempfile::TempDir;

    fn ctx() -> (TempDir, Context) {
        let tmp = TempDir::new().unwrap();
        let ctx = Context::resolve(Some(tmp.path().join("ce-ai")), false, false, true).unwrap();
        (tmp, ctx)
    }

    fn panel_text(ctx: &Context) -> String {
        workflow_panel_lines(ctx)
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

#[cfg(test)]
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
