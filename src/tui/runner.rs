//! TUI runner — event loop and raw-mode guard (KTD3, R5).

use std::io::{stdout, IsTerminal};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use super::app::App;
use super::tabs::MenuTab;
use crate::commands::Context;
use crate::error::CeError;

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
            .draw(|f| super::render::ui(f, app, ctx))
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
                        let lines = super::handlers::run_init_prj_cmd(ctx);
                        execute_action(app, "Adopt Project (init-prj)", move || lines);
                    }
                    KeyCode::Enter | KeyCode::Char('r') => {
                        let dry_run = app.dry_run;
                        match app.current_tab() {
                            MenuTab::Status => {
                                execute_action(app, "Harness Status", || {
                                    super::handlers::run_status_cmd(ctx)
                                });
                            }
                            MenuTab::Workflow => {
                                execute_action(app, "Workflow FSM Status", || {
                                    super::handlers::run_workflow_cmd(ctx)
                                });
                            }
                            MenuTab::Install => {
                                let lines = super::handlers::run_install_cmd(ctx, app, dry_run);
                                execute_action(app, "Install Plugin", move || lines);
                            }
                            MenuTab::Models => {
                                execute_action(app, "Model Assignments", || {
                                    super::handlers::run_models_cmd(ctx)
                                });
                            }
                            MenuTab::Skills => {
                                execute_action(app, "Skills Registry", || {
                                    super::handlers::run_skills_cmd(ctx)
                                });
                            }
                            MenuTab::Sync => {
                                execute_action(app, "Sync Drift", move || {
                                    super::handlers::run_sync_cmd(ctx, dry_run)
                                });
                            }
                            MenuTab::Upgrade => {
                                let lines = super::handlers::run_upgrade_cmd(ctx, app);
                                execute_action(app, "Upgrade Release", move || lines);
                            }
                            MenuTab::Doctor => {
                                execute_action(app, "Doctor Diagnostics", || {
                                    super::handlers::run_doctor_cmd(ctx)
                                });
                            }
                            MenuTab::Backups => {
                                let lines = super::handlers::run_restore_backup_cmd(ctx, app);
                                execute_action(app, "Restore Backup", move || lines);
                            }
                            MenuTab::Tools => {
                                execute_action(app, "Tools Status", || {
                                    super::handlers::run_tools_cmd(ctx)
                                });
                            }
                            MenuTab::Usage => {
                                execute_action(app, "Usage Report", || {
                                    super::handlers::run_usage_cmd(ctx)
                                });
                            }
                            MenuTab::Audit => {
                                execute_action(app, "Audit", || {
                                    super::handlers::run_audit_cmd(ctx)
                                });
                            }
                            MenuTab::InitPrj => {
                                let lines = super::handlers::run_init_prj_cmd(ctx);
                                execute_action(app, "Project Adopt", move || lines);
                            }
                            MenuTab::Uninstall => {
                                let lines = super::handlers::run_uninstall_cmd(ctx, app);
                                execute_action(app, "Uninstall Plugin", move || lines);
                            }
                            MenuTab::Exit => break,
                        }
                    }
                    KeyCode::Char(c @ '1'..='7') if app.current_tab() == MenuTab::Workflow => {
                        if let Some(stage_num) = c.to_digit(10) {
                            let lines =
                                super::handlers::workflow_stage_transition_lines(ctx, stage_num);
                            execute_action(app, "Workflow Stage Transition", move || lines);
                        }
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
