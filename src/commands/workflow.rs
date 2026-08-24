//! `ce-ai workflow`: Finite State Machine (FSM) & progress recovery system across
//! the 7 development stages (Ideation -> OpenSpec -> Plan -> Work -> Verify -> Compound -> Ship).

use std::path::PathBuf;

use serde_json::json;

use crate::commands::Context;
use crate::error::CeError;
use crate::state::state::{State, WorkflowStage, WorkflowState};

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(clap::Subcommand)]
pub enum Action {
    /// Query current 7-stage workflow phase, active task, and progress state.
    Status {
        /// Output machine-readable JSON format.
        #[arg(long)]
        json: bool,
    },
    /// Save a workflow progress checkpoint before context compaction or hand-off.
    Checkpoint {
        /// Active subtask (e.g. "Implementing TDD module").
        #[arg(long, short = 't')]
        task: String,
        /// Current 7-stage phase (e.g. "4" or "work" or "Stage 4: TDD & Work").
        #[arg(long, short = 's', alias = "phase")]
        stage: String,
        /// Optional feature or change package name.
        #[arg(long, short = 'f')]
        feature: Option<String>,
        /// Output machine-readable JSON format.
        #[arg(long)]
        json: bool,
    },
    /// Resume workflow from exact checkpoint using Engram memory and OpenSpec state.
    Resume {
        /// Output machine-readable JSON format.
        #[arg(long)]
        json: bool,
    },
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    match &args.action {
        Action::Status { json } => {
            if *json {
                let state_path = ctx.config_dir.join("state.json");
                let state = State::load(&state_path)?;
                let wf = state.current_workflow();
                println!("{}", serde_json::to_string_pretty(&wf)?);
            } else {
                for line in status_lines(ctx)? {
                    println!("{line}");
                }
            }
        }
        Action::Checkpoint {
            task,
            stage,
            feature,
            json,
        } => {
            let target_stage = WorkflowStage::parse(stage)?;
            let lines = checkpoint_lines(ctx, target_stage, task, feature.as_deref())?;
            if *json {
                let state_path = ctx.config_dir.join("state.json");
                let state = State::load(&state_path)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&state.current_workflow())?
                );
            } else {
                for line in &lines {
                    println!("{line}");
                }
            }
        }
        Action::Resume { json } => {
            if *json {
                let state_path = ctx.config_dir.join("state.json");
                let state = State::load(&state_path)?;
                let wf = state.current_workflow();
                let openspec_info = probe_openspec_context(&wf);
                let payload = json!({
                    "workflow": wf,
                    "openspec_context": openspec_info,
                });
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                for line in resume_lines(ctx)? {
                    println!("{line}");
                }
            }
        }
    }
    Ok(())
}

/// Real status content as renderable lines; the CLI prints them and the TUI
/// renders them verbatim in its result modal.
pub fn status_lines(ctx: &Context) -> Result<Vec<String>, CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load(&state_path)?;

    let mut lines = vec![
        "== [Workflow FSM & Progress Recovery Status] ==".to_string(),
        "7-Stage Cycle (Compound Engineering Skill Mappings):".to_string(),
        "  • [1: Ideation]   ➔ ce-brainstorm / ce-ideate / ce-strategy".to_string(),
        "  • [2: OpenSpec]   ➔ Formal Spec Definition (proposal, spec, tasks)".to_string(),
        "  • [3: Plan]       ➔ ce-plan / ce-doc-review".to_string(),
        "  • [4: Work/TDD]   ➔ ce-work / ce-debug (Direct Entry Point for Bug Fixes) / ce-simplify-code".to_string(),
        "  • [5: Verify]     ➔ Empirical Testing (project test/e2e commands)".to_string(),
        "  • [6: Compound]   ➔ ce-compound / ce-compound-refresh (docs/solutions/)".to_string(),
        "  • [7: Ship]       ➔ ce-commit-push-pr / ce-commit / ce-resolve-pr-feedback".to_string(),
        String::new(),
    ];

    if let Some(cp) = state.latest_release_tag.as_ref() {
        lines.push(format!("latest release: {cp}"));
    }

    match state.current_workflow() {
        Some(wf) => {
            lines.push(format!(
                "current phase: Stage {}: {} ({})",
                wf.stage.number(),
                stage_display_name(wf.stage),
                wf.stage.as_str()
            ));
            lines.push(format!("active subtask: {}", wf.task));
            if let Some(feat) = &wf.feature_name {
                lines.push(format!("active feature: {feat}"));
            }
            lines.push(format!("last updated: {}", wf.updated_at));
        }
        None => {
            lines.push("current phase: Stage 1: Ideation (ce-brainstorm)".to_string());
            lines.push("active subtask: No active task recorded".to_string());
            lines.push(
                "(No progress checkpoint saved yet — run `ce-ai workflow checkpoint`)".to_string(),
            );
        }
    }
    Ok(lines)
}

/// Saves a stage-transition checkpoint and returns confirmation lines.
pub fn checkpoint_lines(
    ctx: &Context,
    stage: WorkflowStage,
    task: &str,
    feature: Option<&str>,
) -> Result<Vec<String>, CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;

    state.validate_and_set_workflow(stage, task, feature.map(String::from))?;

    if !ctx.dry_run {
        state.save(&state_path)?;
    }

    Ok(vec![
        "workflow: checkpoint saved successfully!".to_string(),
        format!(
            "  phase: Stage {}: {}",
            stage.number(),
            stage_display_name(stage)
        ),
        format!("  task: {task}"),
    ])
}

/// Resume surfaces the checkpoint-derived status plus hand-off framing lines.
pub fn resume_lines(ctx: &Context) -> Result<Vec<String>, CeError> {
    let mut lines = vec!["workflow: resuming execution from latest checkpoint...".to_string()];
    lines.extend(status_lines(ctx)?);

    let state_path = ctx.config_dir.join("state.json");
    let state = State::load(&state_path)?;
    let wf = state.current_workflow();

    if let Some(info) = probe_openspec_context(&wf) {
        lines.push(String::new());
        lines.push(format!("== [Context Re-hydration: {}] ==", info.feature));
        lines.push(format!("  spec location: {}", info.path.display()));
        lines.push(format!("  has proposal: {}", info.has_proposal));
        lines.push(format!("  has spec: {}", info.has_spec));
        lines.push(format!("  has tasks: {}", info.has_tasks));
        if info.total_tasks > 0 {
            lines.push(format!(
                "  tasks progress: {}/{} completed ([x])",
                info.completed_tasks, info.total_tasks
            ));
        }
    }

    lines.push(
        "workflow: re-hydrated context successfully. Proceeding with active task.".to_string(),
    );
    Ok(lines)
}

#[derive(Debug, serde::Serialize)]
pub struct OpenSpecContextInfo {
    pub feature: String,
    pub path: PathBuf,
    pub has_proposal: bool,
    pub has_spec: bool,
    pub has_tasks: bool,
    pub completed_tasks: usize,
    pub total_tasks: usize,
}

fn probe_openspec_context(wf: &Option<WorkflowState>) -> Option<OpenSpecContextInfo> {
    let openspec_dir = PathBuf::from("openspec").join("changes");
    if !openspec_dir.is_dir() {
        return None;
    }

    let target_feature = if let Some(feat) = wf.as_ref().and_then(|w| w.feature_name.clone()) {
        feat
    } else {
        // Fallback: find most recently modified directory in openspec/changes/
        let mut entries: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
        if let Ok(read) = std::fs::read_dir(&openspec_dir) {
            for entry in read.flatten() {
                if entry.path().is_dir() {
                    let mtime = entry
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    entries.push((entry.path(), mtime));
                }
            }
        }
        entries.sort_by_key(|(_, mtime)| *mtime);
        let (path, _) = entries.pop()?;
        path.file_name()?.to_string_lossy().to_string()
    };

    let change_dir = openspec_dir.join(&target_feature);
    if !change_dir.is_dir() {
        return None;
    }

    let has_proposal = change_dir.join("proposal.md").exists();
    let has_spec = change_dir.join("spec.md").exists();
    let tasks_path = change_dir.join("tasks.md");
    let has_tasks = tasks_path.exists();

    let mut completed_tasks = 0;
    let mut total_tasks = 0;
    if has_tasks {
        if let Ok(content) = std::fs::read_to_string(&tasks_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
                    completed_tasks += 1;
                    total_tasks += 1;
                } else if trimmed.starts_with("- [ ]") {
                    total_tasks += 1;
                }
            }
        }
    }

    Some(OpenSpecContextInfo {
        feature: target_feature,
        path: change_dir,
        has_proposal,
        has_spec,
        has_tasks,
        completed_tasks,
        total_tasks,
    })
}

fn stage_display_name(stage: WorkflowStage) -> &'static str {
    match stage {
        WorkflowStage::Ideation => "Ideation (ce-brainstorm)",
        WorkflowStage::OpenSpec => "OpenSpec Definition",
        WorkflowStage::ExecutionPlan => "Execution Plan (ce-plan)",
        WorkflowStage::WorkTdd => "TDD & Work (ce-work)",
        WorkflowStage::Verification => "Verification (cargo test / make e2e)",
        WorkflowStage::KnowledgeCapture => "Knowledge Capture (ce-compound)",
        WorkflowStage::GitShipping => "Git Shipping (ce-commit-push-pr)",
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::commands::Context;
    use tempfile::TempDir;

    fn ctx() -> (TempDir, Context) {
        let tmp = TempDir::new().unwrap();
        let ctx = Context::resolve(Some(tmp.path().join("ce-ai")), false, false, true).unwrap();
        (tmp, ctx)
    }

    #[test]
    fn status_lists_stages_and_defaults_without_checkpoint() {
        let (_tmp, ctx) = ctx();
        let joined = status_lines(&ctx).unwrap().join("\n");
        assert!(joined.contains("[1: Ideation]"));
        assert!(joined.contains("[7: Ship]"));
    }

    #[test]
    fn checkpoint_validates_transitions_and_saves_workflow_state() {
        let (_tmp, ctx) = ctx();

        // Stage 1 -> Stage 2: Valid
        let res = checkpoint_lines(
            &ctx,
            WorkflowStage::OpenSpec,
            "Authoring spec",
            Some("my-feat"),
        );
        assert!(res.is_ok());

        // Stage 2 -> Stage 5: Invalid jump (2 -> 5)
        let res_err = checkpoint_lines(&ctx, WorkflowStage::Verification, "Testing", None);
        assert!(res_err.is_err());
        assert!(res_err
            .unwrap_err()
            .to_string()
            .contains("invalid workflow transition"));

        // Stage 2 -> Stage 3: Valid advance
        let res3 = checkpoint_lines(&ctx, WorkflowStage::ExecutionPlan, "Planning tasks", None);
        assert!(res3.is_ok());
    }
}
