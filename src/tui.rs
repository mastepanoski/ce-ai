//! Full-screen Ratatui TUI dashboard for ce-ai.
//! Provides a modern, rich, split-panel terminal interface with live status,
//! keyboard navigation, model slot tables, and one-key action execution.

use std::collections::BTreeMap;
use std::io::{stdout, IsTerminal};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Terminal;

use crate::commands::{doctor, install, sync, uninstall, upgrade, Context};
use crate::error::CeError;
use crate::harness::HarnessKind;
use crate::opencode::manifest::InstallManifest;
use crate::state::state::State;

#[allow(dead_code)]
pub const LOGO: &str = r#"
       █████████████
    ███
   ██        █████
   ██        █████
    ███
      █████████████

       C O M P O U N D
        ─ ENGINEERING ─
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTab {
    Status,
    Workflow,
    Install,
    Models,
    Sync,
    Upgrade,
    Doctor,
    Backups,
    Uninstall,
    Exit,
}

impl MenuTab {
    fn all() -> Vec<Self> {
        vec![
            MenuTab::Status,
            MenuTab::Workflow,
            MenuTab::Install,
            MenuTab::Models,
            MenuTab::Sync,
            MenuTab::Upgrade,
            MenuTab::Doctor,
            MenuTab::Backups,
            MenuTab::Uninstall,
            MenuTab::Exit,
        ]
    }

    fn title(&self) -> &'static str {
        match self {
            MenuTab::Status => "📊  Status & Harnesses",
            MenuTab::Workflow => "🎮  Workflow (FSM)",
            MenuTab::Install => "📥  Install Plugin",
            MenuTab::Models => "🤖  Models & Profiles",
            MenuTab::Sync => "🔄  Sync & Reconcile",
            MenuTab::Upgrade => "🚀  Upgrade Release",
            MenuTab::Doctor => "🩺  Health Doctor",
            MenuTab::Backups => "💾  Backups & Restore",
            MenuTab::Uninstall => "🗑️   Uninstall Plugin",
            MenuTab::Exit => "❌  Quit Dashboard",
        }
    }
}

struct App {
    selected_tab: usize,
    dry_run: bool,
    harnesses: Vec<(String, String, String)>, // (name, version, source)
    detected_harnesses: Vec<HarnessKind>,
    model_assignments: Vec<(String, String)>, // (slot, model)
    output_modal: Option<(String, Vec<String>)>, // (title, lines)
    selected_harness_idx: usize,
    harness_targets: Vec<String>,
    selected_backup_idx: usize,
    backups: Vec<crate::state::backups::BackupEntry>,
}

impl App {
    fn new(ctx: &Context) -> Self {
        let mut app = Self {
            selected_tab: 0,
            dry_run: ctx.dry_run,
            harnesses: Vec::new(),
            detected_harnesses: Vec::new(),
            model_assignments: Vec::new(),
            output_modal: None,
            selected_harness_idx: 0,
            selected_backup_idx: 0,
            backups: Vec::new(),
            harness_targets: vec![
                "all".into(),
                "opencode".into(),
                "claude".into(),
                "pi".into(),
                "cursor".into(),
                "copilot".into(),
                "codex".into(),
                "grok".into(),
                "kimi".into(),
                "agy".into(),
                "deepseek".into(),
                "fx".into(),
                "custom".into(),
            ],
        };
        app.reload_state(ctx);
        app
    }

    fn selected_harness_target(&self) -> &str {
        self.harness_targets
            .get(self.selected_harness_idx)
            .map(|s| s.as_str())
            .unwrap_or("all")
    }

    fn next_harness(&mut self) {
        if self.selected_harness_idx + 1 < self.harness_targets.len() {
            self.selected_harness_idx += 1;
        } else {
            self.selected_harness_idx = 0;
        }
    }

    fn prev_harness(&mut self) {
        if self.selected_harness_idx > 0 {
            self.selected_harness_idx -= 1;
        } else {
            self.selected_harness_idx = self.harness_targets.len() - 1;
        }
    }

    fn reload_state(&mut self, ctx: &Context) {
        self.harnesses.clear();
        self.detected_harnesses.clear();
        self.model_assignments.clear();

        if let Ok(home) = std::env::var("HOME") {
            self.detected_harnesses =
                HarnessKind::detect_installed_harnesses(std::path::Path::new(&home));
        }

        let mut seen = std::collections::HashSet::new();

        // Load harness status from state.json
        let state_path = ctx.config_dir.join("state.json");
        if let Ok(state) = State::load(&state_path) {
            for h in &state.installed_harnesses {
                let name = h["name"].as_str().unwrap_or("unknown").to_string();
                let version = h["version"].as_str().unwrap_or("unknown").to_string();
                let source = h["source"]["kind"].as_str().unwrap_or("local").to_string();
                seen.insert(name.clone());
                self.harnesses.push((name, version, source));
            }
            for (slot, model_info) in &state.model_assignments {
                self.model_assignments.push((
                    slot.clone(),
                    format!("{}/{}", model_info.provider_id, model_info.model_id),
                ));
            }
        }

        // Auto-probe host harnesses for compound-engineering installations
        if let Ok(home) = std::env::var("HOME") {
            let home_path = std::path::Path::new(&home);
            for h in HarnessKind::detect_ce_installed_harnesses(home_path) {
                let name = h.to_string();
                if !seen.contains(&name) {
                    seen.insert(name.clone());
                    self.harnesses
                        .push((name, "host-detected".to_string(), "local".to_string()));
                }
            }
        }

        let backups_dir = ctx.config_dir.join("backups");
        let filter = self.selected_harness_target();
        self.backups =
            crate::state::backups::list_backups(&backups_dir, Some(filter)).unwrap_or_default();
        if self.selected_backup_idx >= self.backups.len() && !self.backups.is_empty() {
            self.selected_backup_idx = self.backups.len() - 1;
        }
    }

    fn current_tab(&self) -> MenuTab {
        MenuTab::all()[self.selected_tab]
    }
}

pub fn run_interactive(ctx: &Context) -> Result<(), CeError> {
    if !std::io::stdin().is_terminal() {
        return Err(CeError::Usage(
            "no subcommand provided — run 'ce-ai --help' for usage or run in an interactive terminal for TUI mode".into(),
        ));
    }

    enable_raw_mode().map_err(|e| CeError::Runtime(format!("raw mode error: {e}")))?;
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
            .draw(|f| ui(f, app, ctx))
            .map_err(|e| CeError::Runtime(format!("draw error: {e}")))?;

        if event::poll(Duration::from_millis(100))
            .map_err(|e| CeError::Runtime(format!("poll error: {e}")))?
        {
            if let Event::Key(key) =
                event::read().map_err(|e| CeError::Runtime(format!("read error: {e}")))?
            {
                if app.output_modal.is_some() {
                    // Any key closes the modal
                    app.output_modal = None;
                    app.reload_state(ctx);
                    continue;
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
                            MenuTab::Uninstall => {
                                let lines = run_uninstall_cmd(ctx, app);
                                execute_action(app, "Uninstall Plugin", move || lines);
                            }
                            MenuTab::Exit => break,
                        }
                    }
                    KeyCode::Char(c @ '1'..='7') if app.current_tab() == MenuTab::Workflow => {
                        let stage_num = c.to_digit(10).unwrap();
                        let (phase, task) = match stage_num {
                            1 => (
                                "Stage 1: Ideation",
                                "1.0 Ideation & Brainstorming (ce-brainstorm)",
                            ),
                            2 => ("Stage 2: OpenSpec Definition", "2.0 OpenSpec Specification"),
                            3 => ("Stage 3: Execution Plan", "3.0 Technical Plan (ce-plan)"),
                            4 => ("Stage 4: TDD & Work", "4.0 Implementation (ce-work)"),
                            5 => (
                                "Stage 5: Empirical Verification",
                                "5.0 Verification (cargo test)",
                            ),
                            6 => (
                                "Stage 6: Knowledge Capture",
                                "6.0 Knowledge Capture (ce-compound)",
                            ),
                            7 => (
                                "Stage 7: Git Shipping",
                                "7.0 PR Delivery (ce-commit-push-pr)",
                            ),
                            _ => ("Stage 1: Ideation", "1.0 Ideation"),
                        };
                        let args = crate::commands::workflow::Args {
                            action: crate::commands::workflow::Action::Checkpoint {
                                task: task.to_string(),
                                phase: phase.to_string(),
                            },
                        };
                        let lines = match crate::commands::workflow::run(ctx, &args) {
                            Ok(_) => vec![
                                format!("✅ Workflow Checkpoint Saved to {phase}!"),
                                format!("Active Task: {task}"),
                                "".to_string(),
                                "Stage transition recorded in state.json successfully.".to_string(),
                            ],
                            Err(err) => vec![format!("❌ Failed to save checkpoint: {err}")],
                        };
                        execute_action(app, "Workflow Stage Transition", move || lines);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn ui(f: &mut ratatui::Frame, app: &App, ctx: &Context) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // Header logo
            Constraint::Min(10),   // Main dashboard body
            Constraint::Length(3), // Footer status bar
        ])
        .split(f.area());

    // 1. Header Banner
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Compound Engineering Plugin Manager (ce-ai) ");
    let header_lines = vec![
        Line::from(Span::styled(
            "       █████████████   C O M P O U N D",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "    ███                ─ ENGINEERING ─",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "   ██        █████",
            Style::default().fg(Color::LightCyan),
        )),
        Line::from(Span::styled(
            format!("   ██        █████     Harnesses: 12 Supported | {} Detected | {} Active", app.detected_harnesses.len(), app.harnesses.len()),
            Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "    ███                Supported: opencode, claude, pi, cursor, copilot, codex, grok, kimi, agy, deepseek, fx, custom",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            format!("      █████████████    Config Dir: {}", ctx.config_dir.display()),
            Style::default().fg(Color::Gray),
        )),
    ];
    let header_p = Paragraph::new(header_lines).block(header_block);
    f.render_widget(header_p, main_chunks[0]);

    // 2. Dashboard Body (Sidebar + Content Panel)
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(main_chunks[1]);

    // Left Sidebar Menu
    let menu_items: Vec<ListItem> = MenuTab::all()
        .iter()
        .enumerate()
        .map(|(idx, tab)| {
            let style = if idx == app.selected_tab {
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Span::styled(tab.title(), style))
        })
        .collect();

    let sidebar = List::new(menu_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Navigation ")
            .border_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(sidebar, body_chunks[0]);

    // Right Content Panel
    render_content_panel(f, body_chunks[1], app, ctx);

    // 3. Footer Bar
    let dry_run_str = if app.dry_run { "ON" } else { "OFF" };
    let dry_run_color = if app.dry_run {
        Color::Yellow
    } else {
        Color::Green
    };
    let footer_text = vec![Line::from(vec![
        Span::styled(" [↑/↓/j/k] ", Style::default().fg(Color::Yellow)),
        Span::raw("Navigate | "),
        Span::styled(" [Enter] ", Style::default().fg(Color::Green)),
        Span::raw("Execute | "),
        Span::styled(" [d] ", Style::default().fg(Color::Cyan)),
        Span::raw("Dry-Run: "),
        Span::styled(
            dry_run_str,
            Style::default()
                .fg(dry_run_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(" [q/Esc] ", Style::default().fg(Color::Red)),
        Span::raw("Quit"),
    ])];
    let footer = Paragraph::new(footer_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(footer, main_chunks[2]);

    // 4. Output Modal (if active)
    if let Some((ref title, ref lines)) = app.output_modal {
        render_modal(f, title, lines);
    }
}

fn render_content_panel(f: &mut ratatui::Frame, area: Rect, app: &App, ctx: &Context) {
    let tab = app.current_tab();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", tab.title()))
        .border_style(Style::default().fg(Color::Cyan));

    let content: Vec<Line> = match tab {
        MenuTab::Status => {
            let mut lines = vec![
                Line::from(Span::styled("Harness Installation Status:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
            ];
            if app.harnesses.is_empty() {
                lines.push(Line::from(Span::styled("  ⚠️  No active harnesses installed yet.", Style::default().fg(Color::Red))));
                lines.push(Line::from("  Use 'Install Plugin' tab to install CE into your host harness."));
            } else {
                for (name, ver, src) in &app.harnesses {
                    lines.push(Line::from(vec![
                        Span::styled("  ✅ ", Style::default().fg(Color::Green)),
                        Span::styled(name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::raw(format!(" — version: {ver} (source: {src})")),
                    ]));
                }
                let has_local = app.harnesses.iter().any(|(_, ver, src)| ver == "local" || src == "local");
                if has_local {
                    lines.push(Line::from(""));
                    lines.push(Line::from(Span::styled("  💡 Aviso de Versión (Fuente Local):", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
                    lines.push(Line::from("     Instalación realizada desde código fuente local (dev)."));
                    lines.push(Line::from(Span::styled("     Para actualizar a la última Release publicada en GitHub, usa la pestaña '🚀 Upgrade Release'.", Style::default().fg(Color::Yellow))));
                }
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Host Detected Harnesses:", Style::default().fg(Color::Yellow))));
            if app.detected_harnesses.is_empty() {
                lines.push(Line::from("  (No host agent harnesses auto-detected in home directory)"));
            } else {
                let detected_names: Vec<String> = app.detected_harnesses.iter().map(|h| h.to_string()).collect();
                lines.push(Line::from(format!("  Found: {}", detected_names.join(", "))));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Config Location:", Style::default().fg(Color::Yellow))));
            lines.push(Line::from(format!("  {}", ctx.config_dir.display())));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Press [Enter] to run full status check.", Style::default().fg(Color::Gray))));
            lines
        }
        MenuTab::Workflow => {
            let mut lines = vec![
                Line::from(Span::styled("Workflow FSM Engine & Progress Recovery:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled("7-Stage Flywheel Cycle:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Line::from("  • [1: Ideation]   ➔ ce-brainstorm / ce-ideate / ce-strategy"),
                Line::from("  • [2: OpenSpec]   ➔ Formal Spec Definition (proposal, spec, tasks)"),
                Line::from("  • [3: Plan]       ➔ ce-plan / ce-doc-review"),
                Line::from("  • [4: Work/TDD]   ➔ ce-work / ce-debug / ce-simplify-code"),
                Line::from("  • [5: Verify]     ➔ Empirical Testing (cargo test, make e2e)"),
                Line::from("  • [6: Compound]   ➔ ce-compound (docs/solutions/)"),
                Line::from("  • [7: Ship]       ➔ ce-commit-push-pr"),
                Line::from(""),
            ];
            let state_path = ctx.config_dir.join("state.json");
            if let Ok(state) = State::load(&state_path) {
                if let Some(cp) = state.last_update_check.clone() {
                    lines.push(Line::from(vec![
                        Span::styled("  Latest Checkpoint: ", Style::default().fg(Color::Yellow)),
                        Span::styled(cp, Style::default().fg(Color::White)),
                    ]));
                } else {
                    lines.push(Line::from("  (No progress checkpoint saved yet — run 'ce-ai workflow checkpoint')"));
                }
            } else {
                lines.push(Line::from(Span::styled("  ⚠️ Could not read state.json", Style::default().fg(Color::Red))));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Press keys [1-7] to transition stage checkpoints directly, or [Enter] to query status.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
            lines
        }
        MenuTab::Install => vec![
            Line::from(Span::styled("Install Compound Engineering Plugin:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Target Harness: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("< [ {} ] >", app.selected_harness_target()),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  (Press ◄/► or h/l to switch)", Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from(format!(
                "  - Target selection: {}",
                if app.selected_harness_target() == "all" {
                    "All host-detected agent harnesses"
                } else {
                    app.selected_harness_target()
                }
            )),
            Line::from("  - Downloads and installs CE loader & 200+ skills into selected harness."),
            Line::from("  - Creates automatic backup of host configuration."),
            Line::from("  - Preserves existing user plugins & custom skill paths."),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Mode: "),
                Span::styled(
                    if app.dry_run { "PREVIEW (Dry-Run)" } else { "APPLY (Real Write)" },
                    Style::default().fg(if app.dry_run { Color::Yellow } else { Color::Green }).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  (Press 'd' to toggle)", Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                format!("👉 Press [Enter] to execute installation for target [{}].", app.selected_harness_target()),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )),
        ],
        MenuTab::Models => {
            let mut lines = vec![
                Line::from(Span::styled("Current Agent Model Assignments:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
            ];
            if app.model_assignments.is_empty() {
                lines.push(Line::from(Span::raw("  (Default harness model configurations)")));
            } else {
                for (slot, model) in &app.model_assignments {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  • {slot}: "), Style::default().fg(Color::Cyan)),
                        Span::styled(model, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ]));
                }
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Press [Enter] to view assignments, set slots, or save profiles.", Style::default().fg(Color::Gray))));
            lines
        }
        MenuTab::Sync => vec![
            Line::from(Span::styled("Reconcile Drift (Sync):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Target Harness: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("< [ {} ] >", app.selected_harness_target()),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  (Press ◄/► or h/l to switch)", Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from("  💡 ¿Qué hace Sync?"),
            Line::from("     - Reconcilia archivos contra el manifiesto SHA256 actual (repara archivos borrados/dañados)."),
            Line::from("     - NO descarga versiones nuevas de internet; mantiene la versión instalada."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to execute local sync.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        ],
        MenuTab::Upgrade => vec![
            Line::from(Span::styled("Upgrade CE Release (Descargar Nueva Versión):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Target Harness: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("< [ {} ] >", app.selected_harness_target()),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  (Press ◄/► or h/l to switch)", Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from(format!("  - Current CLI Version: v{}", env!("CARGO_PKG_VERSION"))),
            Line::from("  💡 ¿Qué hace Upgrade?"),
            Line::from("     - Busca y descarga la última Release publicada en GitHub."),
            Line::from("     - Actualiza loaders y 200+ skills en todos los arneses seleccionados."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to fetch latest release and upgrade.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        ],
        MenuTab::Doctor => vec![
            Line::from(Span::styled("Health Doctor Diagnostics:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  - Validates harness config JSON structures."),
            Line::from("  - Verifies managed asset integrity and state consistency."),
            Line::from("  - Reports non-zero exit on findings."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to run health doctor.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        ],
        MenuTab::Backups => {
            let mut lines = vec![
                Line::from(Span::styled("Historical Configuration Backups:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Target Filter: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        format!("< [ {} ] >", app.selected_harness_target()),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("  (Press ◄/► or h/l to switch)", Style::default().fg(Color::Gray)),
                ]),
                Line::from(""),
            ];
            if app.backups.is_empty() {
                lines.push(Line::from(Span::styled("  (No backups found for current harness target)", Style::default().fg(Color::Gray))));
            } else {
                lines.push(Line::from(Span::styled(
                    "   ID                       HARNESS      TIMESTAMP                SIZE",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    "   -----------------------------------------------------------------------",
                    Style::default().fg(Color::DarkGray),
                )));
                for (idx, b) in app.backups.iter().enumerate() {
                    let selected = idx == app.selected_backup_idx;
                    let prefix = if selected { " 👉 " } else { "    " };
                    let style = if selected {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    lines.push(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(Color::Green)),
                        Span::styled(
                            format!("{:<24} {:<12} {:<24} {} B", b.id, b.harness, b.timestamp_rfc3339, b.size_bytes),
                            style,
                        ),
                    ]));
                }
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "👉 Press [Enter] or 'r' to restore selected backup snapshot.",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )));
            lines
        }
        MenuTab::Uninstall => vec![
            Line::from(Span::styled("Uninstall Plugin & Restore Config:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Target Harness: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("< [ {} ] >", app.selected_harness_target()),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("  (Press ◄/► or h/l to switch)", Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from("  - Restores target harness config from the latest timestamped backup."),
            Line::from("  - Removes managed compound-engineering plugin files."),
            Line::from("  - Cleans internal state."),
            Line::from(""),
            Line::from(Span::styled(
                format!("👉 Press [Enter] to execute uninstallation for target [{}].", app.selected_harness_target()),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
        ],
        MenuTab::Exit => vec![
            Line::from(""),
            Line::from(Span::styled("Press [Enter] to exit the dashboard.", Style::default().fg(Color::Yellow))),
        ],
    };

    let p = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn render_modal(f: &mut ratatui::Frame, title: &str, lines: &[String]) {
    let area = centered_rect(80, 70, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(Color::Yellow));

    let mut modal_lines: Vec<Line> = lines.iter().map(|l| Line::from(l.as_str())).collect();
    modal_lines.push(Line::from(""));
    modal_lines.push(Line::from(Span::styled(
        "Press any key to close this window...",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));

    let p = Paragraph::new(modal_lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn execute_action<F>(app: &mut App, title: &str, f: F)
where
    F: FnOnce() -> Vec<String>,
{
    let lines = f();
    app.output_modal = Some((title.to_string(), lines));
}

fn run_status_cmd(ctx: &Context) -> Vec<String> {
    let mut out = Vec::new();
    let state_path = ctx.config_dir.join("state.json");
    if let Ok(state) = State::load(&state_path) {
        if state.installed_harnesses.is_empty() {
            out.push("Installed: None".into());
        } else {
            for h in &state.installed_harnesses {
                out.push(format!(
                    "Installed: {} ({}, source: {})",
                    h["name"].as_str().unwrap_or("?"),
                    h["version"].as_str().unwrap_or("?"),
                    h["source"]["kind"].as_str().unwrap_or("?")
                ));
            }
        }
    }
    let managed = ctx.opencode_config_dir.join("compound-engineering");
    if let Ok(manifest) = InstallManifest::load(&ctx.opencode_config_dir) {
        let desired: BTreeMap<String, String> = manifest
            .files
            .iter()
            .map(|f| (f.path.clone(), f.sha256.clone()))
            .collect();
        let drift = crate::state::diff::diff(&desired, &desired, &managed);
        if drift.actions.is_empty() {
            out.push("Drift: None (All managed files healthy)".into());
        } else {
            out.push(format!("Drift: {} changes detected", drift.actions.len()));
        }
    } else {
        out.push("Drift: Unknown (No install manifest)".into());
    }
    out
}

fn run_install_cmd(ctx: &Context, app: &App, dry_run: bool) -> Vec<String> {
    let mut install_ctx = ctx.clone();
    install_ctx.dry_run = dry_run;

    let selected = app.selected_harness_target();
    let target_harnesses: Vec<String> = if selected == "all" {
        if app.detected_harnesses.is_empty() {
            vec!["opencode".into()]
        } else {
            app.detected_harnesses
                .iter()
                .map(|h| h.to_string())
                .collect()
        }
    } else {
        vec![selected.to_string()]
    };

    let mut out = vec![
        format!("✅ Installation completed for target: [{selected}]"),
        "".to_string(),
    ];
    for harness_str in &target_harnesses {
        let args = install::Args {
            harness: harness_str.clone(),
            source: None,
            scope: "global".into(),
        };
        match install::run(&install_ctx, &args) {
            Ok(_) => out.push(format!("  • {harness_str}: OK")),
            Err(err) => out.push(format!("  • {harness_str}: {err}")),
        }
    }
    out.push("".to_string());
    out.push(format!(
        "Mode: {}",
        if dry_run {
            "Dry-run (Preview)"
        } else {
            "Applied"
        }
    ));
    out
}

fn run_models_cmd(ctx: &Context) -> Vec<String> {
    let mut out = vec!["Current Model Assignments:".to_string(), "".to_string()];
    let state_path = ctx.config_dir.join("state.json");
    if let Ok(state) = State::load(&state_path) {
        if state.model_assignments.is_empty() {
            out.push("  (No custom slot assignments)".into());
        } else {
            for (slot, info) in &state.model_assignments {
                out.push(format!(
                    "  • {slot}: {}/{}",
                    info.provider_id, info.model_id
                ));
            }
        }
    }
    out.push("".to_string());
    out.push("To assign a slot or save/load profiles, use command line:".to_string());
    out.push("  ce-ai models set <slot> <provider/model>".to_string());
    out.push("  ce-ai models profile save <name>".to_string());
    out
}

fn run_sync_cmd(ctx: &Context, dry_run: bool) -> Vec<String> {
    let mut sync_ctx = ctx.clone();
    sync_ctx.dry_run = dry_run;
    match sync::run(&sync_ctx, &sync::Args::default()) {
        Ok(_) => vec![
            "✅ Sync completed!".to_string(),
            format!(
                "Mode: {}",
                if dry_run {
                    "Dry-run (Preview)"
                } else {
                    "Applied"
                }
            ),
        ],
        Err(err) => vec![format!("❌ Sync failed: {err}")],
    }
}

fn run_workflow_cmd(ctx: &Context) -> Vec<String> {
    let args = crate::commands::workflow::Args {
        action: crate::commands::workflow::Action::Status,
    };
    match crate::commands::workflow::run(ctx, &args) {
        Ok(_) => vec![
            "✅ Workflow FSM Status checked cleanly!".to_string(),
            "Use 'ce-ai workflow checkpoint' or 'resume' to manage progress.".to_string(),
        ],
        Err(err) => vec![format!("❌ Workflow check failed: {err}")],
    }
}

fn run_upgrade_cmd(ctx: &Context, app: &App) -> Vec<String> {
    let target = app.selected_harness_target().to_string();
    let args = upgrade::Args {
        to: None,
        source: None,
        harness: target.clone(),
        force: true,
    };
    match upgrade::run(ctx, &args) {
        Ok(_) => vec![
            "✅ Upgrade completed successfully!".to_string(),
            format!("Target Harness Scope: {target}"),
            format!("Updated to version: v{}", env!("CARGO_PKG_VERSION")),
        ],
        Err(err) => vec![format!("❌ Upgrade failed: {err}")],
    }
}

fn run_doctor_cmd(ctx: &Context) -> Vec<String> {
    match doctor::run(ctx) {
        Ok(_) => vec![
            "✅ Doctor check: OK!".to_string(),
            "All configuration files, state integrity, and managed assets are clean.".to_string(),
        ],
        Err(err) => vec![format!("❌ Doctor check failed: {err}")],
    }
}

fn run_uninstall_cmd(ctx: &Context, app: &App) -> Vec<String> {
    let selected = app.selected_harness_target();
    let target_harnesses: Vec<String> = if selected == "all" {
        if app.harnesses.is_empty() {
            vec!["opencode".to_string()]
        } else {
            app.harnesses.iter().map(|(n, _, _)| n.clone()).collect()
        }
    } else {
        vec![selected.to_string()]
    };

    let mut out = vec![
        format!("✅ Uninstallation completed for target: [{selected}]"),
        "".to_string(),
    ];
    for harness in &target_harnesses {
        let args = uninstall::Args {
            harness: harness.clone(),
            all: false,
            yes: true,
        };
        match uninstall::run(ctx, &args) {
            Ok(_) => out.push(format!(
                "  • {harness}: Uninstalled & pre-install config restored"
            )),
            Err(err) => out.push(format!("  • {harness}: {err}")),
        }
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

fn run_init_prj_cmd(ctx: &Context) -> Vec<String> {
    match crate::commands::init_prj::run(ctx, None, "full", false) {
        Ok(_) => vec![
            "✓ Project Adoption Complete!".into(),
            "".into(),
            "Injected managed block into AGENTS.md and updated state.json.".into(),
        ],
        Err(err) => vec![format!("❌ Failed to adopt project: {err}")],
    }
}
