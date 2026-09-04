//! `ce-ai workflow`: Finite State Machine (FSM) & progress recovery system across
//! the 7 development stages (Ideation -> OpenSpec -> Plan -> Work -> Verify -> Compound -> Ship).

use std::collections::BTreeMap;
use std::io::{IsTerminal, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::commands::init_prj::{check_adoption_block_status, AdoptionBlockStatus};
use crate::commands::Context;
use crate::error::CeError;
use crate::opencode::manifest::InstallManifest;
use crate::opencode::plugins::MANAGED_DIR;
use crate::state::diff;
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
        /// Current 7-stage phase (e.g. "4", "work", "tdd").
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
        /// Antigravity PreInvocation hook mode (reads stdin, dedupes per conversationId, injects ephemeralMessage).
        #[arg(long)]
        pre_invocation: bool,
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
        Action::Resume {
            json,
            pre_invocation,
        } => {
            if *pre_invocation {
                handle_pre_invocation(ctx)?;
            } else if *json {
                let state_path = ctx.config_dir.join("state.json");
                let state = State::load(&state_path)?;
                let wf = state.current_workflow();
                let repo_state = probe_repo_state(ctx, &wf);
                let openspec_info = repo_state.openspec_context.clone();
                let text_lines = resume_lines(ctx)?;
                let additional_context = text_lines.join("\n");
                let payload = json!({
                    "additionalContext": additional_context,
                    "additional_context": additional_context,
                    "hookSpecificOutput": {
                        "hookEventName": "SessionStart",
                        "additionalContext": additional_context,
                    },
                    "workflow": wf,
                    "repo_state": repo_state,
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

    let mut lines = vec![
        "workflow: checkpoint saved successfully!".to_string(),
        format!(
            "  phase: Stage {}: {}",
            stage.number(),
            stage_display_name(stage)
        ),
        format!("  task: {task}"),
    ];

    let repo_state = probe_repo_state(ctx, &state.current_workflow());
    if repo_state.manifest_drift_count > 0 {
        lines.push(format!(
            "! Warning: Drift detected in {} managed files. Run 'ce-ai sync' to reconcile.",
            repo_state.manifest_drift_count
        ));
    }

    Ok(lines)
}

/// Resume surfaces the checkpoint-derived status plus hand-off framing lines.
pub fn resume_lines(ctx: &Context) -> Result<Vec<String>, CeError> {
    let mut lines = vec!["workflow: resuming execution from latest checkpoint...".to_string()];
    lines.extend(status_lines(ctx)?);

    let state_path = ctx.config_dir.join("state.json");
    let state = State::load(&state_path)?;
    let wf = state.current_workflow();
    let repo_state = probe_repo_state(ctx, &wf);

    lines.push(String::new());
    lines.push("== [Environment State & Drift Status] ==".to_string());
    if let Some(branch) = &repo_state.git_branch {
        let head = repo_state.head_sha.as_deref().unwrap_or("unknown");
        lines.push(format!("  git branch: {branch} (HEAD: {head})"));
    } else {
        lines.push("  git branch: non-git workspace".to_string());
    }

    if repo_state.is_git_clean {
        lines.push("  working tree: clean (0 uncommitted changes)".to_string());
    } else {
        let count = repo_state.modified_files.len();
        let preview = if count <= 3 {
            repo_state.modified_files.join(", ")
        } else {
            format!(
                "{}, +{} more",
                repo_state.modified_files[..3].join(", "),
                count - 3
            )
        };
        lines.push(format!(
            "  working tree: {count} modified files ({preview})"
        ));
    }

    if repo_state.manifest_drift_count == 0 {
        lines.push("  manifest integrity: clean (0 drifted files)".to_string());
    } else {
        lines.push(format!(
            "  manifest integrity: ! {} files modified outside ce-ai",
            repo_state.manifest_drift_count
        ));
        lines.push(
            "  ! Warning: Drift detected in managed files. Run 'ce-ai sync' to reconcile."
                .to_string(),
        );
    }

    if let Some(status) = &repo_state.adoption_status {
        match status {
            AdoptionBlockStatus::Ok => {
                lines.push("  adoption block: ok (SHA256 verified)".to_string())
            }
            AdoptionBlockStatus::StaleVersion { version } => {
                lines.push(format!("  adoption block: stale version (v{version})"))
            }
            AdoptionBlockStatus::DriftDetected => lines
                .push("  adoption block: ! drift detected (modified outside ce-ai)".to_string()),
            AdoptionBlockStatus::MalformedBlock => {
                lines.push("  adoption block: ! malformed markers".to_string())
            }
            AdoptionBlockStatus::BlockMissing => {
                lines.push("  adoption block: ! block missing".to_string())
            }
            AdoptionBlockStatus::FileMissing => {
                lines.push("  adoption block: ! file missing".to_string())
            }
            AdoptionBlockStatus::ReadError => {
                lines.push("  adoption block: ! read error".to_string())
            }
        }
    }

    if let Some(info) = &repo_state.openspec_context {
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

    lines.push(String::new());
    lines.push(
        "workflow: re-hydrated context successfully. Proceeding with active task.".to_string(),
    );
    Ok(lines)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoState {
    pub git_branch: Option<String>,
    pub head_sha: Option<String>,
    pub is_git_clean: bool,
    pub modified_files: Vec<String>,
    pub manifest_drift_count: usize,
    pub adoption_status: Option<AdoptionBlockStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openspec_context: Option<OpenSpecContextInfo>,
}

pub fn probe_git_branch(repo_root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if branch.is_empty() || branch == "HEAD" {
        None
    } else {
        Some(branch)
    }
}

pub fn probe_git_head_sha(repo_root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if sha.is_empty() {
        None
    } else {
        Some(sha)
    }
}

pub fn probe_git_dirty_files(repo_root: &Path) -> (bool, Vec<String>) {
    let out = match std::process::Command::new("git")
        .args(["status", "--porcelain=v1"])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return (true, Vec::new()),
    };

    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut modified = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.len() > 3 {
            let path = trimmed[3..].trim().to_string();
            if let Some((_, new_path)) = path.split_once(" -> ") {
                modified.push(new_path.to_string());
            } else {
                modified.push(path);
            }
        }
    }
    let is_clean = modified.is_empty();
    (is_clean, modified)
}

pub fn probe_manifest_drift_count(ctx: &Context) -> usize {
    let manifest = InstallManifest::load(&ctx.opencode_config_dir);
    let desired: BTreeMap<String, String> = match manifest {
        Ok(m) => m.files.into_iter().map(|f| (f.path, f.sha256)).collect(),
        Err(_) => return 0,
    };
    if desired.is_empty() {
        return 0;
    }
    let managed_dir = ctx.opencode_config_dir.join(MANAGED_DIR);
    let diff_result = diff::diff(&desired, &desired, &managed_dir);
    diff_result.actions.len()
}

pub fn probe_adoption_status(ctx: &Context) -> Option<AdoptionBlockStatus> {
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load(&state_path).ok()?;
    let repo_root = ctx.repo_root();
    let entry = state.project_for_path(&repo_root)?;
    let agents_file = repo_root.join(&entry.file);
    Some(check_adoption_block_status(&agents_file, entry.tier))
}

pub fn probe_repo_state(ctx: &Context, wf: &Option<WorkflowState>) -> RepoState {
    let repo_root = ctx.repo_root();
    let git_branch = probe_git_branch(&repo_root);
    let head_sha = probe_git_head_sha(&repo_root);
    let (is_git_clean, modified_files) = probe_git_dirty_files(&repo_root);
    let manifest_drift_count = probe_manifest_drift_count(ctx);
    let adoption_status = probe_adoption_status(ctx);
    let openspec_context = probe_openspec_context_in(&repo_root, wf);

    RepoState {
        git_branch,
        head_sha,
        is_git_clean,
        modified_files,
        manifest_drift_count,
        adoption_status,
        openspec_context,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenSpecContextInfo {
    pub feature: String,
    pub path: PathBuf,
    pub has_proposal: bool,
    pub has_spec: bool,
    pub has_tasks: bool,
    pub completed_tasks: usize,
    pub total_tasks: usize,
}

pub fn probe_openspec_context(wf: &Option<WorkflowState>) -> Option<OpenSpecContextInfo> {
    probe_openspec_context_in(Path::new("."), wf)
}

pub fn probe_openspec_context_in(
    repo_root: &Path,
    wf: &Option<WorkflowState>,
) -> Option<OpenSpecContextInfo> {
    let openspec_dir = repo_root.join("openspec").join("changes");
    if !openspec_dir.is_dir() {
        return None;
    }

    let target_feature = if let Some(feat) = wf
        .as_ref()
        .and_then(|w| w.feature_name.clone())
        .filter(|f| !f.trim().is_empty())
    {
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

#[derive(Deserialize, Default, Debug)]
struct PreInvocationPayload {
    #[serde(rename = "conversationId")]
    conversation_id: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    #[serde(rename = "invocationNum")]
    invocation_num: Option<u64>,
}

pub(crate) fn should_inject_pre_invocation(stdin_content: &str, marker_dir: &Path) -> bool {
    let payload =
        serde_json::from_str::<PreInvocationPayload>(stdin_content.trim()).unwrap_or_default();
    let conv_id = payload
        .conversation_id
        .as_deref()
        .or(payload.session_id.as_deref());

    if let Some(id) = conv_id {
        let safe_id: String = id
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let marker = marker_dir.join(format!("ce-ai-agy-session-{safe_id}.marker"));
        if marker.exists() {
            let is_stale = marker
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.elapsed().ok())
                .map(|dur| dur.as_secs() > 86400)
                .unwrap_or(false);
            if !is_stale {
                return false;
            }
        }
        let _ = std::fs::write(&marker, b"1");
        true
    } else {
        payload.invocation_num.unwrap_or(0) == 0
    }
}

fn agy_marker_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CE_AI_AGY_MARKER_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    std::env::temp_dir()
}

fn handle_pre_invocation(ctx: &Context) -> Result<(), CeError> {
    let mut stdin_content = String::new();
    if !std::io::stdin().is_terminal() {
        let _ = std::io::stdin().read_to_string(&mut stdin_content);
    }

    if !should_inject_pre_invocation(&stdin_content, &agy_marker_dir()) {
        println!("{{}}");
        return Ok(());
    }

    let lines = resume_lines(ctx)?;
    let msg = lines.join("\n");
    let resp = json!({
        "injectSteps": [
            {
                "ephemeralMessage": msg
            }
        ]
    });
    println!("{}", serde_json::to_string(&resp)?);
    Ok(())
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
#[path = "tests/workflow.rs"]
mod tests;
