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
use crate::state::state::{State, WorkflowSource, WorkflowStage, WorkflowState};

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
    let repo_root = ctx.repo_root();
    let branch = probe_git_branch(&repo_root);
    let state_path = ctx.config_dir.join("state.json");

    match &args.action {
        Action::Status { json } => {
            let _ = maybe_auto_checkpoint(ctx, &repo_root, &state_path);
            if *json {
                let state = State::load(&state_path)?;
                let wf = state.current_workflow_for_branch(&repo_root, branch.as_deref());
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
                let state = State::load(&state_path)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &state.current_workflow_for_branch(&repo_root, branch.as_deref())
                    )?
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
            let _ = maybe_auto_checkpoint(ctx, &repo_root, &state_path);
            if *pre_invocation {
                handle_pre_invocation(ctx)?;
            } else if *json {
                let state = State::load(&state_path)?;
                let wf = state.current_workflow_for_branch(&repo_root, branch.as_deref());
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
    let repo_root = ctx.repo_root();
    let branch = probe_git_branch(&repo_root);
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

    match state.current_workflow_for_branch(&repo_root, branch.as_deref()) {
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

    let repo_state = probe_repo_state(
        ctx,
        &state.current_workflow_for_branch(&repo_root, branch.as_deref()),
    );
    if let Some(desync) = &repo_state.task_desync {
        let warn = desync.warning_line();
        if !warn.is_empty() {
            lines.push(warn);
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
    let repo_root = ctx.repo_root();
    let branch = probe_git_branch(&repo_root);
    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;

    state.validate_and_set_workflow_for_branch(
        &repo_root,
        branch.as_deref(),
        stage,
        task,
        feature.map(String::from),
        WorkflowSource::Manual,
    )?;

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

    let repo_state = probe_repo_state(
        ctx,
        &state.current_workflow_for_branch(&repo_root, branch.as_deref()),
    );
    if repo_state.manifest_drift_count > 0 {
        lines.push(format!(
            "! Warning: Drift detected in {} managed files. Run 'ce-ai sync' to reconcile.",
            repo_state.manifest_drift_count
        ));
    }
    if let Some(desync) = &repo_state.task_desync {
        let warn = desync.warning_line();
        if !warn.is_empty() {
            lines.push(warn);
        }
    }

    Ok(lines)
}

/// Resume surfaces the checkpoint-derived status plus hand-off framing lines.
pub fn resume_lines(ctx: &Context) -> Result<Vec<String>, CeError> {
    let repo_root = ctx.repo_root();
    let branch = probe_git_branch(&repo_root);
    let mut lines = vec!["workflow: resuming execution from latest checkpoint...".to_string()];
    lines.extend(status_lines(ctx)?);

    let state_path = ctx.config_dir.join("state.json");
    let state = State::load(&state_path)?;
    let wf = state.current_workflow_for_branch(&repo_root, branch.as_deref());
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
        if let Some(desync) = &repo_state.task_desync {
            let warn = desync.warning_line();
            if !warn.is_empty() {
                lines.push(format!("  {warn}"));
            }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_desync: Option<TaskDesyncReport>,
}

pub fn probe_git_branch(repo_root: &Path) -> Option<String> {
    if let Ok(out) = std::process::Command::new("git")
        .args(["symbolic-ref", "--short", "HEAD"])
        .current_dir(repo_root)
        .output()
    {
        if out.status.success() {
            let branch = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !branch.is_empty() {
                return Some(branch);
            }
        }
    }
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
        .args(["status", "--porcelain=v1", "-uall"])
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

    let task_desync = openspec_context.as_ref().and_then(|info| {
        let touched_files = probe_feature_touched_files(&repo_root);
        reconcile_tasks_with_git(
            &repo_root,
            &info.feature,
            &info.path.join("tasks.md"),
            &touched_files,
        )
    });

    RepoState {
        git_branch,
        head_sha,
        is_git_clean,
        modified_files,
        manifest_drift_count,
        adoption_status,
        openspec_context,
        task_desync,
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
    } else if let Some(branch_feat) = probe_git_branch(repo_root)
        .map(|b| sanitize_feature_name(&b))
        .filter(|f| openspec_dir.join(f).is_dir())
    {
        branch_feat
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
        entries.sort_by_key(|(path, mtime)| (*mtime, path.clone()));
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

/// Match details for an unchecked task that correlates with modified files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDesyncMatch {
    pub task_index: usize,
    pub task_text: String,
    pub matched_files: Vec<String>,
}

/// Comprehensive report on tasks.md progress vs real git changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDesyncReport {
    pub feature: String,
    pub tasks_path: PathBuf,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub desynced_tasks: Vec<TaskDesyncMatch>,
    pub is_aggregate_desync: bool,
}

impl TaskDesyncReport {
    pub fn has_desync(&self) -> bool {
        !self.desynced_tasks.is_empty() || self.is_aggregate_desync
    }

    pub fn warning_line(&self) -> String {
        if !self.desynced_tasks.is_empty() {
            let count = self.desynced_tasks.len();
            let mut sample_files: Vec<String> = Vec::new();
            for m in &self.desynced_tasks {
                for f in &m.matched_files {
                    if !sample_files.contains(f) {
                        sample_files.push(f.clone());
                    }
                }
            }
            let preview = if sample_files.len() <= 2 {
                sample_files.join(", ")
            } else {
                format!(
                    "{}, +{} more",
                    sample_files[..2].join(", "),
                    sample_files.len() - 2
                )
            };
            format!(
                "! Warning: Tasks desync detected — {count} unchecked task(s) reference modified files ({preview}), but tasks.md shows {}/{} completed. Update tasks.md (- [x]) to reflect progress.",
                self.completed_tasks, self.total_tasks
            )
        } else if self.is_aggregate_desync {
            format!(
                "! Warning: Tasks desync detected — working tree / branch contains modified code, but tasks.md shows 0/{} completed. Update tasks.md (- [x]) to reflect progress.",
                self.total_tasks
            )
        } else {
            String::new()
        }
    }
}

fn is_potential_path(token: &str) -> bool {
    if token.is_empty() || token.contains(' ') || token.contains('\n') {
        return false;
    }
    if token.starts_with("http://") || token.starts_with("https://") {
        return false;
    }
    let lower = token.to_lowercase();
    if lower.starts_with("openspec/") || lower == "openspec" || lower.starts_with(".git/") {
        return false;
    }
    if lower == "cargo.lock" || lower.ends_with(".lock") {
        return false;
    }
    let has_slash = token.contains('/');
    let has_ext = [
        ".rs", ".ts", ".js", ".json", ".toml", ".md", ".sh", ".yml", ".yaml",
    ]
    .iter()
    .any(|ext| lower.ends_with(ext));

    has_slash || has_ext
}

pub fn extract_paths_from_task_text(text: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut in_backtick = false;
    let mut current_bt = String::new();
    for ch in text.chars() {
        if ch == '`' {
            if in_backtick {
                let trimmed = current_bt.trim().trim_matches(['\'', '"', '(', ')']);
                if is_potential_path(trimmed) && !candidates.iter().any(|c| c == trimmed) {
                    candidates.push(trimmed.to_string());
                }
                current_bt.clear();
                in_backtick = false;
            } else {
                in_backtick = true;
            }
        } else if in_backtick {
            current_bt.push(ch);
        }
    }
    for word in text.split_whitespace() {
        let clean = word
            .trim_start_matches(['`', '(', '[', '"', '\''])
            .trim_end_matches(['`', ')', ']', '"', '\'', ':', ',', '.']);
        if is_potential_path(clean) && !candidates.iter().any(|c| c == clean) {
            candidates.push(clean.to_string());
        }
    }
    candidates
}

pub fn probe_branch_committed_files(repo_root: &Path) -> Vec<String> {
    let merge_base = std::process::Command::new("git")
        .args(["merge-base", "HEAD", "origin/main"])
        .current_dir(repo_root)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::process::Command::new("git")
                .args(["merge-base", "HEAD", "main"])
                .current_dir(repo_root)
                .output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .filter(|s| !s.is_empty())
        });

    let diff_target = match merge_base {
        Some(base) => format!("{base}...HEAD"),
        None => "HEAD~1...HEAD".to_string(),
    };

    let out = match std::process::Command::new("git")
        .args(["diff", "--name-only", &diff_target])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Vec::new(),
    };

    out.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

pub fn probe_feature_touched_files(repo_root: &Path) -> Vec<String> {
    let (_, dirty) = probe_git_dirty_files(repo_root);
    let committed = probe_branch_committed_files(repo_root);

    let mut touched = Vec::new();
    for f in dirty.into_iter().chain(committed) {
        let lower = f.to_lowercase();
        if lower.starts_with("openspec/") || lower.starts_with(".git/") || lower.ends_with(".lock")
        {
            continue;
        }
        if !touched.contains(&f) {
            touched.push(f);
        }
    }
    touched.sort();
    touched
}

pub fn reconcile_tasks_with_git(
    _repo_root: &Path,
    feature: &str,
    tasks_path: &Path,
    touched_files: &[String],
) -> Option<TaskDesyncReport> {
    if !tasks_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(tasks_path).ok()?;

    let mut completed_tasks = 0;
    let mut total_tasks = 0;
    let mut unchecked_tasks: Vec<(usize, String)> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            completed_tasks += 1;
            total_tasks += 1;
        } else if trimmed.starts_with("- [ ]") {
            let task_text = trimmed
                .strip_prefix("- [ ]")
                .unwrap_or("")
                .trim()
                .to_string();
            unchecked_tasks.push((total_tasks + 1, task_text));
            total_tasks += 1;
        }
    }

    if total_tasks == 0 {
        return None;
    }

    let mut desynced_tasks = Vec::new();
    for (idx, task_text) in unchecked_tasks {
        let candidate_paths = extract_paths_from_task_text(&task_text);
        let mut matched_files = Vec::new();
        for p in candidate_paths {
            for touched in touched_files {
                let is_match = touched == &p
                    || (p.ends_with('/') && touched.starts_with(&p))
                    || (touched.ends_with('/') && p.starts_with(touched))
                    || touched.starts_with(&format!("{p}/"))
                    || p.starts_with(&format!("{touched}/"))
                    || touched.ends_with(&format!("/{p}"));
                if is_match && !matched_files.contains(touched) {
                    matched_files.push(touched.clone());
                }
            }
        }
        if !matched_files.is_empty() {
            desynced_tasks.push(TaskDesyncMatch {
                task_index: idx,
                task_text,
                matched_files,
            });
        }
    }

    // Aggregate fallback (R2):
    // If no unchecked task had explicit matching paths, but completed_tasks == 0 and
    // touched_files contains implementation code (under src/, tests/, or skills/)
    let is_aggregate_desync = desynced_tasks.is_empty()
        && completed_tasks == 0
        && total_tasks > 0
        && touched_files
            .iter()
            .any(|f| f.starts_with("src/") || f.starts_with("tests/") || f.starts_with("skills/"));

    let report = TaskDesyncReport {
        feature: feature.to_string(),
        tasks_path: tasks_path.to_path_buf(),
        completed_tasks,
        total_tasks,
        desynced_tasks,
        is_aggregate_desync,
    };

    if report.has_desync() {
        Some(report)
    } else {
        None
    }
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

pub fn is_transitory_git_state(repo_root: &Path) -> bool {
    let git_dir = repo_root.join(".git");
    let actual_git_dir = if git_dir.is_file() {
        // In worktrees or submodules, .git is a file containing `gitdir: <path>`
        std::fs::read_to_string(&git_dir)
            .ok()
            .and_then(|content| {
                content.lines().next().and_then(|line| {
                    line.strip_prefix("gitdir: ").map(|p| {
                        let trimmed = p.trim();
                        let path = PathBuf::from(trimmed);
                        if path.is_absolute() {
                            path
                        } else {
                            repo_root.join(path)
                        }
                    })
                })
            })
            .unwrap_or(git_dir)
    } else {
        git_dir
    };

    actual_git_dir.join("rebase-merge").exists()
        || actual_git_dir.join("rebase-apply").exists()
        || actual_git_dir.join("CHERRY_PICK_HEAD").exists()
        || actual_git_dir.join("MERGE_HEAD").exists()
}

pub fn sanitize_feature_name(branch: &str) -> String {
    let stripped = branch
        .trim_start_matches("refs/heads/")
        .trim_start_matches("feature/")
        .trim_start_matches("feat/")
        .trim_start_matches("fix/");
    let sanitized: String = stripped
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = sanitized.trim_matches('-');
    if trimmed.is_empty() {
        "default".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn check_gh_pr_shipping(repo_root: &Path) -> bool {
    let out = match std::process::Command::new("gh")
        .args(["pr", "view", "--json", "state", "-q", ".state"])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_uppercase(),
        _ => return false,
    };
    out == "OPEN" || out == "MERGED"
}

pub fn has_committed_solutions_on_branch(repo_root: &Path) -> bool {
    let out = match std::process::Command::new("git")
        .args(["diff-tree", "--no-commit-id", "--name-only", "-r", "HEAD"])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return false,
    };
    out.lines()
        .any(|f| f.starts_with("docs/solutions/") && f.ends_with(".md"))
}

pub fn infer_stage_from_repo(
    repo_root: &Path,
    branch: Option<&str>,
) -> Option<(WorkflowStage, String, Option<String>)> {
    if is_transitory_git_state(repo_root) {
        return None;
    }

    let openspec_dir = repo_root.join("openspec").join("changes");

    // 1. Resolve feature candidate from branch or probe
    let candidate = branch.map(sanitize_feature_name);
    let resolved_feature = candidate
        .filter(|f| openspec_dir.join(f).is_dir())
        .or_else(|| probe_openspec_context_in(repo_root, &None).map(|info| info.feature));

    // 2. OpenSpec deduction (Stages 2, 3, 4, 5, 6, 7)
    if let Some(ref feat) = resolved_feature {
        let change_dir = openspec_dir.join(feat);
        let has_proposal = change_dir.join("proposal.md").exists();
        let has_spec = change_dir.join("spec.md").exists();
        let tasks_path = change_dir.join("tasks.md");

        if tasks_path.exists() {
            let mut completed_tasks = 0;
            let mut total_tasks = 0;
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

            if total_tasks > 0 && completed_tasks == 0 {
                return Some((
                    WorkflowStage::ExecutionPlan,
                    format!("Execution plan authored for {feat}"),
                    Some(feat.clone()),
                ));
            } else if completed_tasks > 0 && completed_tasks < total_tasks {
                return Some((
                    WorkflowStage::WorkTdd,
                    format!(
                        "Implementing tasks ({completed_tasks}/{total_tasks} completed) for {feat}"
                    ),
                    Some(feat.clone()),
                ));
            } else if total_tasks > 0 && completed_tasks == total_tasks {
                // All tasks completed: check Stage 7 (Ship), Stage 6 (Compound), or Stage 5 (Verify)
                if check_gh_pr_shipping(repo_root) {
                    return Some((
                        WorkflowStage::GitShipping,
                        format!("Shipping changes / pull request for {feat}"),
                        Some(feat.clone()),
                    ));
                }

                let (_, modified_files) = probe_git_dirty_files(repo_root);
                let has_solutions_dirty = modified_files
                    .iter()
                    .any(|f| f.starts_with("docs/solutions/") && f.ends_with(".md"));
                let has_solutions_committed = has_committed_solutions_on_branch(repo_root);

                if has_solutions_dirty || has_solutions_committed {
                    return Some((
                        WorkflowStage::KnowledgeCapture,
                        format!("Capturing solution in docs/solutions/ for {feat}"),
                        Some(feat.clone()),
                    ));
                }

                return Some((
                    WorkflowStage::Verification,
                    format!("Verifying test gates for {feat}"),
                    Some(feat.clone()),
                ));
            }
        }

        if has_proposal && has_spec {
            return Some((
                WorkflowStage::OpenSpec,
                format!("Authoring OpenSpec contract for {feat}"),
                Some(feat.clone()),
            ));
        }
    }

    // 3. Direct Entry Bypass for Stage 4 (Work/TDD):
    // If no OpenSpec, but on fix/* or feat/* branch with dirty files
    if let Some(b) = branch {
        let is_work_branch = b.starts_with("fix/")
            || b.starts_with("feat/")
            || b.starts_with("fix-")
            || b.starts_with("feat-");
        if is_work_branch {
            let (is_clean, _) = probe_git_dirty_files(repo_root);
            if !is_clean {
                let feat_name = sanitize_feature_name(b);
                return Some((
                    WorkflowStage::WorkTdd,
                    format!("Direct entry bugfix / work on {b}"),
                    Some(feat_name),
                ));
            }
        }
    }

    // 4. Ideation (Stage 1):
    // docs/ideation/ or docs/brainstorms/*.md exists and no openspec change dir
    let ideation_dir = repo_root.join("docs").join("ideation");
    let brainstorms_dir = repo_root.join("docs").join("brainstorms");
    let has_brainstorms = brainstorms_dir.is_dir()
        && std::fs::read_dir(&brainstorms_dir)
            .ok()
            .map(|r| {
                r.flatten()
                    .any(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
            })
            .unwrap_or(false);

    if ideation_dir.is_dir() || has_brainstorms {
        return Some((
            WorkflowStage::Ideation,
            "Ideation & brainstorming in progress".to_string(),
            None,
        ));
    }

    None
}

pub fn maybe_auto_checkpoint(
    ctx: &Context,
    repo_root: &Path,
    state_path: &Path,
) -> Result<Option<WorkflowState>, CeError> {
    if ctx.dry_run {
        return Ok(None);
    }
    let state = if state_path.exists() {
        State::load(state_path)?
    } else {
        return Ok(None);
    };

    if !state.is_project_adopted(repo_root) {
        return Ok(None);
    }

    if !state.is_auto_checkpoint_enabled() {
        return Ok(None);
    }

    if is_transitory_git_state(repo_root) {
        return Ok(None);
    }

    let branch = probe_git_branch(repo_root);
    let (inferred_stage, inferred_task, inferred_feature) =
        match infer_stage_from_repo(repo_root, branch.as_deref()) {
            Some(inf) => inf,
            None => return Ok(None),
        };

    let current_wf = state.current_workflow_for_branch(repo_root, branch.as_deref());
    let current_stage = current_wf
        .as_ref()
        .map(|wf| wf.stage)
        .unwrap_or(WorkflowStage::Ideation);

    // Monotonic provenance guard: Inferred checkpoints can NEVER regress or clobber a Manual checkpoint at equal or higher stage
    if let Some(ref wf) = current_wf {
        if wf.source == WorkflowSource::Manual && inferred_stage.number() <= current_stage.number()
        {
            return Ok(None);
        }
        if inferred_stage.number() < current_stage.number() {
            return Ok(None);
        }
    }

    if !current_stage.can_transition_to(inferred_stage) {
        return Ok(None);
    }

    // Desync guard (R6): Do NOT auto-advance to Verification (Stage 5), KnowledgeCapture (Stage 6),
    // or GitShipping (Stage 7) if tasks are desynced
    if inferred_stage.number() >= WorkflowStage::Verification.number() {
        let repo_state = probe_repo_state(ctx, &current_wf);
        if repo_state
            .task_desync
            .as_ref()
            .is_some_and(|d| d.has_desync())
        {
            return Ok(None);
        }
    }

    let updated = State::atomic_update_workflow(state_path, repo_root, branch.as_deref(), |s| {
        s.validate_and_set_workflow_for_branch(
            repo_root,
            branch.as_deref(),
            inferred_stage,
            &inferred_task,
            inferred_feature,
            WorkflowSource::Inferred,
        )?;
        Ok(s.current_workflow_for_branch(repo_root, branch.as_deref()))
    })?;
    Ok(Some(updated))
}

#[cfg(test)]
#[path = "tests/workflow.rs"]
mod tests;
