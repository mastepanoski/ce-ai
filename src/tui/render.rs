//! TUI render layer (KTD3, R5) — extracted from monolithic tui.rs.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

use super::app::App;
use super::tabs::MenuTab;
use crate::commands::Context;

pub(crate) fn ui(f: &mut ratatui::Frame, app: &App, ctx: &Context) {
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
            format!("   ██        █████     Harnesses: 11 Supported | {} Detected | {} Active", app.detected_harnesses.len(), app.harnesses.len()),
            Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "    ███                Supported: opencode, claude, pi, cursor, copilot, codex, grok, kimi, agy, fx, custom",
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
        render_modal(f, title, lines, app.output_scroll);
    }

    // 5. Model Picker Modal (if active)
    if app.model_picker_open {
        render_picker(f, app);
    }
}

pub(crate) fn render_picker(f: &mut ratatui::Frame, app: &App) {
    let slot = app
        .model_slots
        .get(app.selected_model_idx)
        .cloned()
        .unwrap_or_default();
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(
            " Pick model for '{slot}' ({}) ",
            app.selected_harness_target()
        ))
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

pub(crate) fn render_content_panel(f: &mut ratatui::Frame, area: Rect, app: &App, ctx: &Context) {
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
            if let Some(err) = &app.state_error {
                lines.push(Line::from(Span::styled(
                    format!("  ❌ state.json corrupt: {err} — run ce-ai doctor"),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
            }
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
                    lines.push(Line::from(Span::styled("  💡 Local Source Notice:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
                    lines.push(Line::from("     Installed from local source tree (dev)."));
                    lines.push(Line::from(Span::styled("     To fetch the latest published Release, use the '🚀 Upgrade Release' tab.", Style::default().fg(Color::Yellow))));
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
        MenuTab::Workflow => super::workflow_panel_lines(ctx),
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
            let harness = app.selected_harness_target().to_string();
            let mut lines = vec![
                Line::from(Span::styled("Agent Model Assignments (editable):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(vec![
                    Span::raw("  Harness scope: "),
                    Span::styled(
                        format!("[ {harness} ]"),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "  (switch with ◄/► or h/l)",
                        Style::default().fg(Color::Gray),
                    ),
                ]),
                Line::from(""),
            ];
            if !crate::commands::models::discovery_supported(&harness) {
                lines.push(Line::from(Span::styled(
                    "  ⚠ no live catalog discovery for this harness yet; assign via CLI",
                    Style::default().fg(Color::Yellow),
                )));
                lines.push(Line::from(""));
            }
            if app.model_slots.is_empty() {
                lines.push(Line::from(Span::raw("  (No editable slots — run install first)")));
            } else {
                for (idx, slot) in app.model_slots.iter().enumerate() {
                    let selected = idx == app.selected_model_idx;
                    let prefix = if selected { "👉 " } else { "   " };
                    let model = app
                        .model_scope
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
                "CLI: ce-ai models set --harness <harness> <slot> <provider/model>",
                Style::default().fg(Color::Gray),
            )));
            lines
        }
        MenuTab::Skills => vec![
            Line::from(Span::styled("Skills Registry (🧩):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  - List indexed skills across all harnesses (global catalog)."),
            Line::from("  - Resolve exact SKILL.md paths for prompt injection."),
            Line::from("  - Doctor checks registry integrity; Adopt puts ce-* copies under management."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to list skills (skills list).", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled("   [a] adopt — ce-ai skills adopt --harness <name> --yes", Style::default().fg(Color::Gray))),
            Line::from(Span::styled("   CLI: ce-ai skills resolve --harness <harness> --query <q>", Style::default().fg(Color::Gray))),
        ],
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
            Line::from("  💡 What does Sync do?"),
            Line::from("     - Reconciles files against current SHA256 manifest (repairs deleted/corrupted files)."),
            Line::from("     - Does NOT download new versions; keeps installed version."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to execute local sync.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        ],
        MenuTab::Upgrade => vec![
            Line::from(Span::styled("Upgrade CE Release (Download New Version):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled(
                "  Target: All active harnesses (global upgrade — harness selector ignored)",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
            Line::from(format!("  - Current CLI Version: v{}", env!("CARGO_PKG_VERSION"))),
            Line::from("  💡 What does Upgrade do?"),
            Line::from("     - Fetches the latest published Release from GitHub."),
            Line::from("     - Updates loaders and 200+ skills on all installed harnesses."),
            Line::from(""),
            Line::from(Span::styled("   Tip: set CE_AI_GITHUB_TOKEN to avoid 403 rate-limit, or pin with --to <tag> / --source <path>", Style::default().fg(Color::Gray))),
            Line::from(Span::styled("👉 Press [Enter] to fetch latest release and upgrade (all).", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
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
        MenuTab::Tools => vec![
            Line::from(Span::styled("Tools & Sidecars (🛠️):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  - CodeGraph, Engram, Context7, RTK — companion sidecars."),
            Line::from("  - Status checks versions and health; Install provisions a tool."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to check tools status.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled("   CLI: ce-ai tools install <codegraph|engram|context7|rtk>", Style::default().fg(Color::Gray))),
        ],
        MenuTab::Usage => vec![
            Line::from(Span::styled("Usage Analytics (📈):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  - Capture harness usage into shard-per-author ledger."),
            Line::from("  - Report aggregates with filters; zen-free fallback if no ledger."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to report usage.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled("   CLI: ce-ai usage sync / ce-ai usage report --json", Style::default().fg(Color::Gray))),
        ],
        MenuTab::Audit => vec![
            Line::from(Span::styled("Audit & Quality (🔍):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  - Token-efficiency and context-quality audit across harnesses."),
            Line::from("  - Checks .codegraph/ index and docs/solutions grounding."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to run audit.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        ],
        MenuTab::InitPrj => vec![
            Line::from(Span::styled("Project Adopt (📁):", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("  - Injects managed CE workflow block into AGENTS.md (tier: full/minimal/orchestrator)."),
            Line::from("  - Deinit removes it cleanly; --force overwrites modified blocks."),
            Line::from(""),
            Line::from(Span::styled("👉 Press [Enter] to adopt current project (init-prj).", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled("   [Shift+I] deinit — ce-ai deinit-prj", Style::default().fg(Color::Gray))),
        ],
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

pub(crate) fn render_modal(f: &mut ratatui::Frame, title: &str, lines: &[String], scroll: usize) {
    let area = centered_rect(80, 70, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(Color::Yellow));

    let scroll = scroll.min(lines.len().saturating_sub(1));
    let mut modal_lines: Vec<Line> = lines
        .iter()
        .skip(scroll)
        .map(|l| Line::from(l.as_str()))
        .collect();
    modal_lines.push(Line::from(""));
    if scroll > 0 {
        modal_lines.push(Line::from(Span::styled(
            format!("↑ scrolled {scroll} — PgUp/PgDn/j/k to scroll, Esc/Enter/q to close"),
            Style::default().fg(Color::Yellow),
        )));
    } else {
        modal_lines.push(Line::from(Span::styled(
            "j/k/PgUp/PgDn to scroll, Esc/Enter/q to close",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }

    let p = Paragraph::new(modal_lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(p, area);
}

pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
