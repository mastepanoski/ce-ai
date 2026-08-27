//! TUI command handlers and panel builders (KTD3, R5) — extracted from mod.rs.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use super::app::App;
use super::spawn;
use crate::commands::Context;
use crate::error::CeError;
use crate::harness::HarnessKind;
use crate::state::state::State;

pub(crate) fn capture_cli(args: &[String]) -> Vec<String> {
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

pub(crate) fn run_status_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::status_args())
}

pub(crate) fn run_install_cmd(_ctx: &Context, app: &App, dry_run: bool) -> Vec<String> {
    capture_cli(&spawn::install_cmd_args(
        app.selected_harness_target(),
        dry_run,
    ))
}

pub(crate) fn run_models_cmd(_ctx: &Context) -> Vec<String> {
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

pub(crate) fn run_sync_cmd(_ctx: &Context, dry_run: bool) -> Vec<String> {
    if dry_run {
        capture_cli(&spawn::sync_cmd_args(true))
    } else {
        capture_cli(&spawn::sync_cmd_args(false))
    }
}

pub(crate) fn run_workflow_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::workflow_status_args())
}

/// Workflow panel content: native actions marked `[run]`, agent-session stages
/// marked `skill:` — text-perceivable, never color-only (R6).
pub(crate) fn workflow_panel_lines(ctx: &Context) -> Vec<Line<'static>> {
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
pub(crate) fn workflow_failure_lines(err: &CeError) -> Vec<String> {
    vec![
        format!("❌ Workflow command failed: {err}"),
        "Remedy: check state.json corruption or permissions, then run `ce-ai doctor`.".into(),
    ]
}

/// Saves the `[1-7]` stage-transition checkpoint and builds its confirmation modal.
pub(crate) fn workflow_stage_transition_lines(ctx: &Context, stage_num: u32) -> Vec<String> {
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
pub(crate) fn run_upgrade_cmd(_ctx: &Context, _app: &App) -> Vec<String> {
    // Plain `upgrade` reconciles every active harness; --harness/--force were
    // removed from the CLI contract in v1.18.1 (Issue #161).
    capture_cli(&spawn::upgrade_cmd_args())
}

pub(crate) fn run_doctor_cmd(_ctx: &Context) -> Vec<String> {
    let mut lines = capture_cli(&spawn::doctor_cmd_args());
    if !lines.iter().any(|l| l.contains("doctor: ok")) {
        lines.push("❌ doctor reported findings (exit non-zero)".to_string());
    }
    lines
}

pub(crate) fn run_uninstall_cmd(_ctx: &Context, app: &App) -> Vec<String> {
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

pub(crate) fn run_restore_backup_cmd(ctx: &Context, app: &mut App) -> Vec<String> {
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

pub(crate) fn run_init_prj_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::init_prj_args())
}

pub(crate) fn run_skills_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::skills_list_args())
}

pub(crate) fn run_tools_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::tools_status_args())
}

pub(crate) fn run_usage_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::usage_report_args())
}

pub(crate) fn run_audit_cmd(_ctx: &Context) -> Vec<String> {
    capture_cli(&spawn::audit_args())
}
