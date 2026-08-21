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
    Install,
    Models,
    Sync,
    Upgrade,
    Doctor,
    Uninstall,
    Exit,
}

impl MenuTab {
    fn all() -> Vec<Self> {
        vec![
            MenuTab::Status,
            MenuTab::Install,
            MenuTab::Models,
            MenuTab::Sync,
            MenuTab::Upgrade,
            MenuTab::Doctor,
            MenuTab::Uninstall,
            MenuTab::Exit,
        ]
    }

    fn title(&self) -> &'static str {
        match self {
            MenuTab::Status => "📊  Status & Harnesses",
            MenuTab::Install => "📥  Install Plugin",
            MenuTab::Models => "🤖  Models & Profiles",
            MenuTab::Sync => "🔄  Sync & Reconcile",
            MenuTab::Upgrade => "🚀  Upgrade Release",
            MenuTab::Doctor => "🩺  Health Doctor",
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
        };
        app.reload_state(ctx);
        app
    }

    fn reload_state(&mut self, ctx: &Context) {
        self.harnesses.clear();
        self.detected_harnesses.clear();
        self.model_assignments.clear();

        if let Ok(home) = std::env::var("HOME") {
            self.detected_harnesses =
                HarnessKind::detect_installed_harnesses(std::path::Path::new(&home));
        }

        // Load harness status
        let state_path = ctx.config_dir.join("state.json");
        if let Ok(state) = State::load(&state_path) {
            for h in &state.installed_harnesses {
                let name = h["name"].as_str().unwrap_or("unknown").to_string();
                let version = h["version"].as_str().unwrap_or("unknown").to_string();
                let source = h["source"]["kind"].as_str().unwrap_or("local").to_string();
                self.harnesses.push((name, version, source));
            }
            for (slot, model_info) in &state.model_assignments {
                self.model_assignments.push((
                    slot.clone(),
                    format!("{}/{}", model_info.provider_id, model_info.model_id),
                ));
            }
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
                    KeyCode::Char('d') => {
                        app.dry_run = !app.dry_run;
                    }
                    KeyCode::Enter => {
                        let dry_run = app.dry_run;
                        match app.current_tab() {
                            MenuTab::Status => {
                                execute_action(app, "Harness Status", || run_status_cmd(ctx));
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
                                execute_action(app, "Upgrade Release", || run_upgrade_cmd(ctx));
                            }
                            MenuTab::Doctor => {
                                execute_action(app, "Doctor Diagnostics", || run_doctor_cmd(ctx));
                            }
                            MenuTab::Uninstall => {
                                let lines = run_uninstall_cmd(ctx, app);
                                execute_action(app, "Uninstall Plugin", move || lines);
                            }
                            MenuTab::Exit => break,
                        }
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
                lines.push(Line::from(Span::styled("  ⚠️  No harnesses installed yet.", Style::default().fg(Color::Red))));
                lines.push(Line::from("  Press [Enter] or select 'Install Plugin' to install CE."));
            } else {
                for (name, ver, src) in &app.harnesses {
                    lines.push(Line::from(vec![
                        Span::styled("  ✅ ", Style::default().fg(Color::Green)),
                        Span::styled(name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::raw(format!(" — version: {ver} (source: {src})")),
                    ]));
                }
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Config Location:", Style::default().fg(Color::Yellow))));
            lines.push(Line::from(format!("  {}", ctx.opencode_config_dir.join("opencode.json").display())));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("Press [Enter] to run full status check.", Style::default().fg(Color::Gray))));
            lines
        }
        MenuTab::Install => vec![
            Line::from(Span::styled("Install Compound Engineering Plugin:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  - Downloads and installs the CE loader & 200+ skills into OpenCode."),
            Line::from("  - Creates automatic backup of ~/.config/opencode/opencode.json."),
            Line::from("  - Preserves existing user plugins & skill paths."),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Mode: "),
                Span::styled(
                    if app.dry_run { "PREVIEW (Dry-Run)" } else { "APPLY (Real Write)" },
                    Style::default().fg(if app.dry_run { Color::Yellow } else { Color::Green }).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to execute installation.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
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
            Line::from("  - Compares managed files against the installed SHA256 manifest."),
            Line::from("  - Restores tampered or missing plugin files."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to execute sync.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        ],
        MenuTab::Upgrade => vec![
            Line::from(Span::styled("Upgrade CE Plugin:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  - Fetches the latest release from GitHub (everyinc/compound-engineering-plugin)."),
            Line::from("  - Caches tarball and SHA256 digest under ~/.ce-ai/cache/."),
            Line::from("  - Runs sync to update local skills and loaders."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to run upgrade.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        ],
        MenuTab::Doctor => vec![
            Line::from(Span::styled("Health Doctor Diagnostics:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  - Validates opencode.json JSON structure."),
            Line::from("  - Verifies managed asset integrity and state consistency."),
            Line::from("  - Reports non-zero exit on findings."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to run health doctor.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        ],
        MenuTab::Uninstall => vec![
            Line::from(Span::styled("Uninstall Plugin & Restore Config:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  - Restores opencode.json from the latest timestamped backup."),
            Line::from("  - Removes ~/.config/opencode/compound-engineering/."),
            Line::from("  - Cleans internal state."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to execute uninstallation.", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
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
    let target_harnesses = if app.detected_harnesses.is_empty() {
        vec![HarnessKind::Opencode]
    } else {
        app.detected_harnesses.clone()
    };

    let mut out = vec![
        "✅ Multi-harness installation completed!".to_string(),
        "".to_string(),
    ];
    for harness_kind in &target_harnesses {
        let args = install::Args {
            harness: harness_kind.to_string(),
            source: None,
        };
        match install::run(&install_ctx, &args) {
            Ok(_) => out.push(format!("  • {harness_kind}: OK")),
            Err(err) => out.push(format!("  • {harness_kind}: {err}")),
        }
    }
    out.push("".to_string());
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
    match sync::run(&sync_ctx) {
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

fn run_upgrade_cmd(ctx: &Context) -> Vec<String> {
    let args = upgrade::Args {
        to: None,
        source: None,
    };
    match upgrade::run(ctx, &args) {
        Ok(_) => vec!["✅ Upgrade completed successfully!".to_string()],
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
    let target_harnesses = if app.harnesses.is_empty() {
        vec!["opencode".to_string()]
    } else {
        app.harnesses.iter().map(|(n, _, _)| n.clone()).collect()
    };

    let mut out = vec![
        "✅ Multi-harness uninstallation completed!".to_string(),
        "".to_string(),
    ];
    for harness in &target_harnesses {
        let args = uninstall::Args {
            harness: harness.clone(),
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
