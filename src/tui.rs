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
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Terminal;

use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;
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
    model_slots: Vec<String>,                 // editable slots (defaults + tracked)
    selected_model_idx: usize,
    model_picker_open: bool,
    picker_items: Vec<String>,
    picker_selected: usize,
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
            model_slots: Vec::new(),
            selected_model_idx: 0,
            model_picker_open: false,
            picker_items: Vec::new(),
            picker_selected: 0,
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
        self.model_slots.clear();

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

        // Editable slot list: CE workflow slots first (structural orchestrator
        // + tracked stage slots), then any extra user-assigned slot.
        for slot in crate::harness::agents::CE_AGENT_SLOTS {
            self.model_slots.push(slot.to_string());
        }
        for (slot, _) in &self.model_assignments {
            if !self.model_slots.contains(slot) {
                self.model_slots.push(slot.clone());
            }
        }
        if self.selected_model_idx >= self.model_slots.len() {
            self.selected_model_idx = self.model_slots.len().saturating_sub(1);
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
                            app.model_picker_open = false;
                            let lines = match crate::commands::models::set(ctx, &slot, &model) {
                                Ok(()) => vec![
                                    format!("✅ Set {slot} = {model}"),
                                    "Applied atomically to opencode.json and state.json."
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
                        KeyCode::Char('m') => match crate::commands::models::discover_models() {
                            Ok(mut models) => {
                                if let Some((_, current)) =
                                    app.model_assignments.iter().find(|(s, _)| {
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
                        },
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

    // 5. Model Picker Modal (if active)
    if app.model_picker_open {
        render_picker(f, app);
    }
}

fn render_picker(f: &mut ratatui::Frame, app: &App) {
    let slot = app
        .model_slots
        .get(app.selected_model_idx)
        .cloned()
        .unwrap_or_default();
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Pick model for '{slot}' "))
        .border_style(Style::default().fg(Color::Cyan));

    let items: Vec<Line> = app
        .picker_items
        .iter()
        .enumerate()
        .map(|(idx, model)| {
            let selected = idx == app.picker_selected;
            Line::from(Span::styled(
                format!(" {model}"),
                if selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            ))
        })
        .collect();

    let p = Paragraph::new(items).block(block);
    f.render_widget(p, area);
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
        MenuTab::Workflow => workflow_panel_lines(ctx),
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
                Line::from(Span::styled("Agent Model Assignments (editable):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
            ];
            if app.model_slots.is_empty() {
                lines.push(Line::from(Span::raw("  (No editable slots — run install first)")));
            } else {
                for (idx, slot) in app.model_slots.iter().enumerate() {
                    let selected = idx == app.selected_model_idx;
                    let prefix = if selected { "👉 " } else { "   " };
                    let model = app
                        .model_assignments
                        .iter()
                        .find(|(s, _)| s == slot)
                        .map(|(_, m)| m.clone())
                        .unwrap_or_else(|| "(not assigned — press m to pick)".to_string());
                    lines.push(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(Color::Green)),
                        Span::styled(
                            format!("{slot}: "),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(if selected { Modifier::BOLD } else { Modifier::empty() }),
                        ),
                        Span::styled(model, Style::default().fg(Color::White)),
                    ]));
                }
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Keys: [n/p] select slot · [m] pick model from harness catalog · [Enter] list assignments",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                "CLI: ce-ai models set <slot> <provider/model>",
                Style::default().fg(Color::Gray),
            )));
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

/// Runs the installed ce-ai binary as a subprocess and captures its output.
/// Commands that `println!` would otherwise paint straight onto the
/// alternate screen and corrupt the dashboard layout (#72-class breakage).
fn capture_cli(args: &[&str]) -> Vec<String> {
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
    capture_cli(&["status"])
}

fn run_install_cmd(_ctx: &Context, app: &App, dry_run: bool) -> Vec<String> {
    let mut args = vec!["install", "--harness", app.selected_harness_target()];
    if dry_run {
        args.push("--dry-run");
    }
    capture_cli(&args)
}

fn run_models_cmd(_ctx: &Context) -> Vec<String> {
    let mut lines = capture_cli(&["models", "list"]);
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
        capture_cli(&["sync", "--dry-run"])
    } else {
        capture_cli(&["sync"])
    }
}

fn run_workflow_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&["workflow", "status"])
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
            "5.0 Verification (project test/e2e commands)",
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
    let mut lines = match crate::commands::workflow::checkpoint_lines(ctx, task, phase) {
        Ok(out) => out,
        Err(err) => return workflow_failure_lines(&err),
    };
    lines.insert(0, format!("✅ Workflow Checkpoint Saved to {phase}!"));
    lines.push("Stage transition recorded in state.json successfully.".to_string());
    lines
}

fn run_upgrade_cmd(_ctx: &Context, app: &App) -> Vec<String> {
    let target = app.selected_harness_target().to_string();
    let mut lines = capture_cli(&["upgrade", "--harness", &target, "--force"]);
    lines.push(format!("Target Harness Scope: {target}"));
    lines
}

fn run_doctor_cmd(_ctx: &Context) -> Vec<String> {
    let mut lines = capture_cli(&["doctor"]);
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
        out.extend(capture_cli(&["uninstall", "--harness", harness, "--yes"]));
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
    capture_cli(&["init-prj"])
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
        assert!(joined.contains("✅ Workflow Checkpoint Saved to Stage 2: OpenSpec Definition!"));
        assert!(joined.contains("Stage transition recorded"));

        let state = State::load(&ctx.config_dir.join("state.json")).unwrap();
        assert!(state
            .last_update_check
            .unwrap()
            .starts_with("Stage 2: OpenSpec Definition | "));
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
