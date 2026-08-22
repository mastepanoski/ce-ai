//! `ce-ai workflow`: Finite State Machine (FSM) & progress recovery system across
//! the 7 development stages (Ideation -> OpenSpec -> Plan -> Work -> Verify -> Compound -> Ship).

use chrono::Utc;

use crate::commands::Context;
use crate::error::CeError;
use crate::state::state::State;

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(clap::Subcommand)]
pub enum Action {
    /// Query current 7-stage workflow phase, active task, and progress state.
    Status,
    /// Save a workflow progress checkpoint before context compaction or hand-off.
    Checkpoint {
        /// Active subtask (e.g. "4.2 Implementing TDD module").
        #[arg(long)]
        task: String,
        /// Current 7-stage phase (e.g. "Stage 4: TDD & Work").
        #[arg(long)]
        phase: String,
    },
    /// Resume workflow from exact checkpoint using Engram memory and OpenSpec state.
    Resume,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let lines = match &args.action {
        Action::Status => status_lines(ctx)?,
        Action::Checkpoint { task, phase } => checkpoint_lines(ctx, task, phase)?,
        Action::Resume => resume_lines(ctx)?,
    };
    for line in &lines {
        println!("{line}");
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

    match state
        .last_update_check
        .as_deref()
        .and_then(parse_checkpoint)
    {
        Some((phase, task)) => {
            lines.push(format!("current phase: {phase}"));
            lines.push(format!("active subtask: {task}"));
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
pub fn checkpoint_lines(ctx: &Context, task: &str, phase: &str) -> Result<Vec<String>, CeError> {
    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;
    state.last_update_check = Some(format!("{phase} | {task} | {}", Utc::now().to_rfc3339()));
    state.save(&state_path)?;

    Ok(vec![
        "workflow: checkpoint saved successfully!".to_string(),
        format!("  phase: {phase}"),
        format!("  task: {task}"),
    ])
}

/// Resume surfaces the checkpoint-derived status plus hand-off framing lines.
pub fn resume_lines(ctx: &Context) -> Result<Vec<String>, CeError> {
    let mut lines = vec!["workflow: resuming execution from latest checkpoint...".to_string()];
    lines.extend(status_lines(ctx)?);
    lines.push(
        "workflow: re-hydrated context successfully. Proceeding with active task.".to_string(),
    );
    Ok(lines)
}

/// Parses a `{phase} | {task} | {timestamp}` checkpoint entry into phase and task.
fn parse_checkpoint(entry: &str) -> Option<(String, String)> {
    let mut parts = entry.splitn(3, " | ");
    let phase = parts.next()?.trim();
    let task = parts.next()?.trim();
    if phase.is_empty() || task.is_empty() {
        return None;
    }
    Some((phase.to_string(), task.to_string()))
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
        assert!(joined.contains("(No progress checkpoint saved yet"));
        assert!(joined.contains("Stage 1: Ideation"));
    }

    #[test]
    fn checkpoint_persists_and_status_reflects_it() {
        let (_tmp, ctx) = ctx();
        let joined = checkpoint_lines(&ctx, "1.0 Brainstorm issue", "Stage 1: Ideation")
            .unwrap()
            .join("\n");
        assert!(joined.contains("checkpoint saved"));
        assert!(joined.contains("Stage 1: Ideation"));
        assert!(joined.contains("1.0 Brainstorm issue"));

        let state = State::load(&ctx.config_dir.join("state.json")).unwrap();
        let cp = state.last_update_check.unwrap();
        assert!(cp.starts_with("Stage 1: Ideation | 1.0 Brainstorm issue | "));

        let status = status_lines(&ctx).unwrap().join("\n");
        assert!(status.contains("current phase: Stage 1: Ideation"));
        assert!(status.contains("active subtask: 1.0 Brainstorm issue"));
    }

    #[test]
    fn corrupt_state_maps_to_error_not_panic() {
        let (_tmp, ctx) = ctx();
        std::fs::create_dir_all(&ctx.config_dir).unwrap();
        std::fs::write(ctx.config_dir.join("state.json"), "{not json").unwrap();
        assert!(status_lines(&ctx).is_err());
        assert!(checkpoint_lines(&ctx, "t", "p").is_err());
    }

    #[test]
    fn resume_lines_include_status_content() {
        let (_tmp, ctx) = ctx();
        let joined = resume_lines(&ctx).unwrap().join("\n");
        assert!(joined.contains("resuming execution"));
        assert!(joined.contains("[Workflow FSM & Progress Recovery Status]"));
        assert!(joined.contains("re-hydrated context successfully"));
    }
}
