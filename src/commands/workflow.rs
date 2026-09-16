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
use crate::state::state::{
    AdoptionTier, DocHygieneConfig, ExecutionMode, FeatureResolution, ReviewReceipt, State,
    WorkflowSource, WorkflowStage, WorkflowState,
};

pub use crate::commands::archive_compact::{
    probe_archive_compaction, run_archive_compact, ArchiveCompactionFinding, ArchiveSubcommand,
    CompactArgs, CompactionCandidate,
};

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
        /// Execution mode override: auto, organic (odd), compound (ce).
        #[arg(long, value_name = "MODE")]
        mode: Option<String>,
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
        /// Execution mode override: auto, organic (odd), compound (ce).
        #[arg(long, value_name = "MODE")]
        mode: Option<String>,
    },
    /// Resume workflow from exact checkpoint using Engram memory and OpenSpec state.
    Resume {
        /// Output machine-readable JSON format.
        #[arg(long)]
        json: bool,
        /// Antigravity PreInvocation hook mode (reads stdin, dedupes per conversationId, injects ephemeralMessage).
        #[arg(long)]
        pre_invocation: bool,
        /// Execution mode override: auto, organic (odd), compound (ce).
        #[arg(long, value_name = "MODE")]
        mode: Option<String>,
    },
    /// Record a code-review receipt for the current branch head (observe-only ship gate, issue #354).
    ReviewReceipt {
        /// Head SHA to stamp (defaults to the current HEAD).
        #[arg(long)]
        head: Option<String>,
        /// Record an explicit override reason instead of a real review (audited).
        #[arg(long)]
        override_reason: Option<String>,
    },
    /// Graduate an ODD task brief to formal OpenSpec change package.
    Graduate(GraduateArgs),
    /// Archive completed OpenSpec change packages to openspec/changes/archive/.
    Archive(ArchiveArgs),
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct GraduateArgs {
    /// Target feature name to graduate from odd/tasks/<feature>.md to openspec/changes/<feature>/.
    pub feature: Option<String>,
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct ArchiveArgs {
    /// Subcommand for archive operations (e.g. `compact`).
    #[command(subcommand)]
    pub subcommand: Option<ArchiveSubcommand>,

    /// Target change folder name to archive (defaults to active feature from state.json).
    pub feature: Option<String>,

    /// Archive all completed change folders detected in openspec/changes/.
    #[arg(long, default_value_t = false)]
    pub all: bool,

    /// Preview intended moves and ledger updates without modifying disk or git.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Criterion 2 STATUS attestation for features with incomplete tasks.
    #[arg(long)]
    pub status: Option<String>,

    /// Automatically promote delta requirements into target domain spec during archival.
    #[arg(long, default_value_t = false)]
    pub promote: bool,

    /// Target domain spec override for specification promotion.
    #[arg(long)]
    pub domain: Option<String>,
}

pub fn run(ctx: &Context, args: &Args) -> Result<(), CeError> {
    let repo_root = ctx.repo_root();
    let branch = probe_git_branch(&repo_root);
    let state_path = ctx.config_dir.join("state.json");

    match &args.action {
        Action::Status { json, mode } => {
            let cli_mode = mode.as_deref().map(ExecutionMode::parse).transpose()?;
            let _ = maybe_auto_checkpoint(ctx, &repo_root, &state_path);
            if *json {
                let state = State::load(&state_path)?;
                let wf = state.current_workflow_for_branch(&repo_root, branch.as_deref());
                println!("{}", serde_json::to_string_pretty(&wf)?);
            } else {
                for line in status_lines_with_mode(ctx, cli_mode)? {
                    println!("{line}");
                }
            }
        }
        Action::Checkpoint {
            task,
            stage,
            feature,
            json,
            mode,
        } => {
            let cli_mode = mode.as_deref().map(ExecutionMode::parse).transpose()?;
            let target_stage = WorkflowStage::parse(stage)?;
            let lines = checkpoint_lines(ctx, target_stage, task, feature.as_deref())?;
            if let Some(m) = cli_mode {
                let mut state = State::load(&state_path)?;
                state.set_execution_mode_for_branch(&repo_root, branch.as_deref(), Some(m));
                if !ctx.dry_run {
                    state.save(&state_path)?;
                }
            }
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
            mode,
        } => {
            let cli_mode = mode.as_deref().map(ExecutionMode::parse).transpose()?;
            let _ = maybe_auto_checkpoint(ctx, &repo_root, &state_path);
            if *pre_invocation {
                handle_pre_invocation(ctx)?;
            } else if *json {
                let state = State::load(&state_path)?;
                let wf = state.current_workflow_for_branch(&repo_root, branch.as_deref());
                let repo_state = probe_repo_state_with_mode(ctx, &wf, cli_mode);
                let openspec_info = repo_state.openspec_context.clone();
                let text_lines = resume_lines_with_mode(ctx, cli_mode)?;
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
                for line in resume_lines_with_mode(ctx, cli_mode)? {
                    println!("{line}");
                }
            }
        }
        Action::ReviewReceipt {
            head,
            override_reason,
        } => {
            let head_sha = head.clone().or_else(|| probe_git_head_sha(&repo_root));
            for line in review_receipt_lines(
                ctx,
                &repo_root,
                branch.as_deref(),
                head_sha,
                override_reason.clone(),
            )? {
                println!("{line}");
            }
        }
        Action::Graduate(graduate_args) => run_graduate(ctx, graduate_args)?,
        Action::Archive(archive_args) => run_archive(ctx, archive_args)?,
    }
    Ok(())
}

/// Real status content as renderable lines; the CLI prints them and the TUI
/// renders them verbatim in its result modal.
pub fn status_lines(ctx: &Context) -> Result<Vec<String>, CeError> {
    status_lines_with_mode(ctx, None)
}

pub fn status_lines_with_mode(
    ctx: &Context,
    cli_mode: Option<ExecutionMode>,
) -> Result<Vec<String>, CeError> {
    let repo_root = ctx.repo_root();
    let branch = probe_git_branch(&repo_root);
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load(&state_path)?;
    let wf = state.current_workflow_for_branch(&repo_root, branch.as_deref());
    let tier = probe_adoption_tier(&state, &repo_root);
    let mode = probe_execution_mode(&repo_root, branch.as_deref(), &wf, tier, cli_mode);

    let mut lines = Vec::new();
    if mode == ExecutionMode::Organic {
        lines.push("== [Workflow Mode: Organic Driven Development (ODD Fast-Path)] ==".to_string());
        lines.push("Tactical Execution Triad (odd/tasks/<feature>.md):".to_string());
        lines.push(
            "  • Problem Statement     ➔ Describe observed symptom, error, or tactical friction"
                .to_string(),
        );
        lines.push(
            "  • Guardrails & Limits   ➔ Inviolable safety, compatibility, and performance bounds"
                .to_string(),
        );
        lines.push(
            "  • Definition of Done    ➔ Concrete, testable checkboxes (- [ ] / - [x])".to_string(),
        );
        lines.push("  * Graduation Notice     ➔ For scope > 200 LOC, run 'ce-ai workflow graduate <feature>' to formalize in OpenSpec.".to_string());
        lines.push(String::new());
    } else {
        lines.push("== [Workflow FSM & Progress Recovery Status] ==".to_string());
        lines.push("7-Stage Cycle (Compound Engineering Skill Mappings):".to_string());
        lines.push("  • [1: Ideation]   ➔ ce-brainstorm / ce-ideate / ce-strategy".to_string());
        lines.push(
            "  • [2: OpenSpec]   ➔ Formal Spec Definition (proposal, spec, tasks)".to_string(),
        );
        lines.push("  • [3: Plan]       ➔ ce-plan / ce-doc-review".to_string());
        lines.push("  • [4: Work/TDD]   ➔ ce-work / ce-debug (Direct Entry Point for Bug Fixes) / ce-simplify-code".to_string());
        lines.push(
            "  • [5: Verify]     ➔ Empirical Testing (project test/e2e commands)".to_string(),
        );
        lines.push(
            "  • [6: Compound]   ➔ ce-compound / ce-compound-refresh (docs/solutions/)".to_string(),
        );
        lines.push(
            "  • [7: Ship]       ➔ ce-commit-push-pr / ce-commit / ce-resolve-pr-feedback"
                .to_string(),
        );
        lines.push(String::new());
    }

    if let Some(cp) = state.latest_release_tag.as_ref() {
        lines.push(format!("latest release: {cp}"));
    }

    match &wf {
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
            if wf.resolution == Some(FeatureResolution::MtimeFallback) {
                lines.push(format!(
                    "! Warning: Active feature '{}' resolved via mtime fallback (unreliable without git branch)",
                    wf.feature_name.as_deref().unwrap_or("unknown")
                ));
            }
            lines.push(format!("last updated: {}", wf.updated_at));
        }
        None => {
            if mode == ExecutionMode::Organic {
                lines.push("current phase: Organic Task (ODD Fast-Path)".to_string());
                lines.push("active subtask: No active task brief recorded".to_string());
            } else {
                lines.push("current phase: Stage 1: Ideation (ce-brainstorm)".to_string());
                lines.push("active subtask: No active task recorded".to_string());
                lines.push(
                    "(No progress checkpoint saved yet — run `ce-ai workflow checkpoint`)"
                        .to_string(),
                );
            }
        }
    }

    if mode == ExecutionMode::Compound {
        let repo_state = probe_repo_state(
            ctx,
            &state.current_workflow_for_branch(&repo_root, branch.as_deref()),
        );
        let stage = state
            .current_workflow_for_branch(&repo_root, branch.as_deref())
            .map(|w| w.stage)
            .unwrap_or_default();
        if repo_state.adoption_status.is_some() {
            for gap in repo_state.ship_readiness_gaps(stage) {
                lines.push(format!("! Warning: {}", gap.describe()));
            }
        }
        if let Some(desync) = &repo_state.task_desync {
            let warn = desync.warning_line();
            if !warn.is_empty() {
                lines.push(warn);
            }
        }
        if !repo_state.unarchived_completed_changes.is_empty() {
            let count = repo_state.unarchived_completed_changes.len();
            lines.push(format!(
                "! Warning: {count} OpenSpec change(s) complete but not archived — run 'ce-ai doctor' for details"
            ));
        }
        if let Some(debt) = &repo_state.doc_debt {
            if debt.has_debt() {
                lines.push(format!("! Warning: {}", debt.summary_line()));
            }
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
    if !repo_state.unarchived_completed_changes.is_empty() {
        let count = repo_state.unarchived_completed_changes.len();
        lines.push(format!(
            "! Warning: {count} OpenSpec change(s) complete but not archived — run 'ce-ai doctor' for details"
        ));
    }
    if let Some(debt) = &repo_state.doc_debt {
        if debt.has_debt() {
            lines.push(format!("! Warning: {}", debt.summary_line()));
        }
    }

    Ok(lines)
}

/// Resume surfaces the checkpoint-derived status plus hand-off framing lines.
pub fn resume_lines(ctx: &Context) -> Result<Vec<String>, CeError> {
    resume_lines_with_mode(ctx, None)
}

pub fn resume_lines_with_mode(
    ctx: &Context,
    cli_mode: Option<ExecutionMode>,
) -> Result<Vec<String>, CeError> {
    let repo_root = ctx.repo_root();
    let branch = probe_git_branch(&repo_root);
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load(&state_path)?;
    let wf = state.current_workflow_for_branch(&repo_root, branch.as_deref());
    let tier = probe_adoption_tier(&state, &repo_root);
    let mode = probe_execution_mode(&repo_root, branch.as_deref(), &wf, tier, cli_mode);

    let mut lines = vec!["workflow: resuming execution from latest checkpoint...".to_string()];
    lines.extend(status_lines_with_mode(ctx, Some(mode))?);

    let repo_state = probe_repo_state_with_mode(ctx, &wf, Some(mode));

    lines.push(String::new());
    lines.push("== [Environment State & Drift Status] ==".to_string());
    if let Some(branch) = &repo_state.git_branch {
        let head = repo_state.head_sha.as_deref().unwrap_or("unknown");
        lines.push(format!("  git branch: {branch} (HEAD: {head})"));
    } else {
        lines.push("  git branch: non-git workspace".to_string());
    }
    lines.push(format!("  execution mode: {}", mode.as_str()));

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

    if mode == ExecutionMode::Compound {
        if repo_state.unarchived_completed_changes.is_empty() {
            lines.push("  openspec ledger: clean (0 pending archival)".to_string());
        } else {
            let count = repo_state.unarchived_completed_changes.len();
            lines.push(format!(
                "  openspec ledger: ! {count} change(s) complete but not archived — run 'ce-ai doctor' for details"
            ));
        }
        if let Some(debt) = &repo_state.doc_debt {
            lines.push(format!("  {}", debt.summary_line()));
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
    } else if let Some(odd) = probe_active_odd_task(&repo_root, &wf) {
        lines.push(String::new());
        lines.push(format!("== [ODD Task Context: {}] ==", odd.feature));
        lines.push(format!("  task brief: {}", odd.path.display()));
        if odd.total_dod > 0 {
            lines.push(format!(
                "  dod progress: {}/{} completed ([x])",
                odd.completed_dod, odd.total_dod
            ));
        }
    }

    let stage = wf.as_ref().map(|w| w.stage).unwrap_or_default();
    if mode == ExecutionMode::Compound {
        lines.extend(ship_readiness_lines(&repo_state, stage));
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_full_sha: Option<String>,
    pub is_git_clean: bool,
    pub modified_files: Vec<String>,
    pub manifest_drift_count: usize,
    pub adoption_status: Option<AdoptionBlockStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub openspec_context: Option<OpenSpecContextInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_desync: Option<TaskDesyncReport>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unarchived_completed_changes: Vec<UnarchivedChange>,
    #[serde(default)]
    pub commits_ahead: u32,
    #[serde(default)]
    pub stage6_artifact_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_receipt: Option<ReviewReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_debt: Option<DocDebtReport>,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
}

/// Observe-only ship-readiness gap (issue #354). Nothing here blocks a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipReadinessGap {
    /// No `docs/solutions/**.md` artifact was added on the branch (Stage 6 missing).
    Stage6Missing,
    /// No code-review receipt was recorded for this branch.
    ReviewReceiptMissing,
    /// A receipt exists but was recorded for a different HEAD.
    ReviewReceiptStale,
    /// Commits are ahead of base while the checkpoint stage is before Verification.
    EarlyStageWithCommits,
}

impl ShipReadinessGap {
    pub fn describe(&self) -> String {
        match self {
            ShipReadinessGap::Stage6Missing => {
                "Stage 6 gap: no docs/solutions artifact on this branch".to_string()
            }
            ShipReadinessGap::ReviewReceiptMissing => {
                "code-review gap: no receipt (run `ce-ai workflow review-receipt` after ce-code-review)"
                    .to_string()
            }
            ShipReadinessGap::ReviewReceiptStale => {
                "code-review gap: receipt is stale (recorded for a different commit)".to_string()
            }
            ShipReadinessGap::EarlyStageWithCommits => {
                "resume: work already committed — resume at Verify/Ship, do not re-run earlier stages"
                    .to_string()
            }
        }
    }
}

/// Pure, deterministic ship-readiness evaluation. Empty when nothing is ahead.
pub fn evaluate_ship_readiness(
    commits_ahead: u32,
    stage6_artifact_present: bool,
    receipt: Option<&ReviewReceipt>,
    head_sha: Option<&str>,
    stage: WorkflowStage,
) -> Vec<ShipReadinessGap> {
    if commits_ahead == 0 {
        return Vec::new();
    }
    let mut gaps = Vec::new();
    if !stage6_artifact_present {
        gaps.push(ShipReadinessGap::Stage6Missing);
    }
    match receipt {
        None => gaps.push(ShipReadinessGap::ReviewReceiptMissing),
        Some(r) => {
            if let Some(current) = head_sha {
                let matches = r.head_sha == current
                    || (current.len() >= 7 && r.head_sha.starts_with(current))
                    || (r.head_sha.len() >= 7 && current.starts_with(&r.head_sha));
                if !current.is_empty() && !matches {
                    gaps.push(ShipReadinessGap::ReviewReceiptStale);
                }
            }
        }
    }
    if stage.number() < WorkflowStage::Verification.number() {
        gaps.push(ShipReadinessGap::EarlyStageWithCommits);
    }
    gaps
}

impl RepoState {
    /// Ship-readiness gaps for the current repository state and workflow stage.
    pub fn ship_readiness_gaps(&self, stage: WorkflowStage) -> Vec<ShipReadinessGap> {
        evaluate_ship_readiness(
            self.commits_ahead,
            self.stage6_artifact_present,
            self.review_receipt.as_ref(),
            self.head_full_sha.as_deref(),
            stage,
        )
    }
}

/// Observe-only ship-readiness block for `workflow resume`. Empty when nothing is
/// ahead or the workspace is not adopted. Gap warnings are emitted by
/// `status_lines`; this block only reports the signals (no duplicate warnings).
pub fn ship_readiness_lines(repo_state: &RepoState, _stage: WorkflowStage) -> Vec<String> {
    if repo_state.commits_ahead == 0 || repo_state.adoption_status.is_none() {
        return Vec::new();
    }
    let mut lines = vec![
        String::new(),
        "== [Ship Readiness (observe-only, issue #354)] ==".to_string(),
        format!("  commits ahead of base: {}", repo_state.commits_ahead),
        format!(
            "  stage 6 artifact (docs/solutions/): {}",
            if repo_state.stage6_artifact_present {
                "present"
            } else {
                "MISSING"
            }
        ),
    ];
    match &repo_state.review_receipt {
        Some(r) => {
            let override_note = r
                .override_reason
                .as_ref()
                .map(|reason| format!(" (override: {reason})"))
                .unwrap_or_default();
            lines.push(format!(
                "  code-review receipt: present @ {}{override_note}",
                r.head_sha
            ));
        }
        None => lines.push("  code-review receipt: MISSING".to_string()),
    }
    lines
}

/// Resolves a commit-ish to its full SHA (defaults to HEAD), for stable receipt comparison.
fn resolve_commit_sha(repo_root: &Path, input: Option<&str>) -> Option<String> {
    let spec = input.unwrap_or("HEAD");
    let rev = format!("{spec}^{{commit}}");
    let out = git_probe(repo_root, &["rev-parse", "--verify", "--quiet", &rev])?;
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

/// Records a code-review receipt for a branch head and returns confirmation lines.
pub fn review_receipt_lines(
    ctx: &Context,
    repo_root: &Path,
    branch: Option<&str>,
    head_sha: Option<String>,
    override_reason: Option<String>,
) -> Result<Vec<String>, CeError> {
    let resolved = resolve_commit_sha(repo_root, head_sha.as_deref()).ok_or_else(|| {
        CeError::Usage(
            "cannot resolve a commit SHA for the review receipt (pass --head <sha> or run inside a git repo with a HEAD)"
                .to_string(),
        )
    })?;

    let state_path = ctx.config_dir.join("state.json");
    let mut state = State::load(&state_path)?;
    let receipt = state.record_review_receipt(repo_root, branch, &resolved, override_reason);
    if !ctx.dry_run {
        state.save(&state_path)?;
    }

    let mut lines = vec![
        "workflow: code-review receipt recorded (observe-only ship gate, issue #354).".to_string(),
        format!("  branch: {}", branch.unwrap_or("(detached HEAD)")),
        format!("  head: {}", receipt.head_sha),
    ];
    if let Some(reason) = &receipt.override_reason {
        lines.push(format!("  override reason: {reason}"));
    }
    if ctx.dry_run {
        lines.push("  (dry-run: state.json not written)".to_string());
    }
    Ok(lines)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnarchivedChange {
    pub feature: String,
    pub completed_tasks: usize,
    pub total_tasks: usize,
}

/// Tri-state indicator for diagnostic probes.
///
/// In a non-git directory or when the necessary substrate is unavailable,
/// probes return `Unknown` rather than silently reporting `Clean`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "status", content = "data", rename_all = "lowercase")]
pub enum ProbeStatus<T> {
    #[default]
    Clean,
    Debt(T),
    Unknown,
}

impl<T> ProbeStatus<T> {
    pub fn is_debt(&self) -> bool {
        matches!(self, Self::Debt(_))
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    pub fn is_clean(&self) -> bool {
        matches!(self, Self::Clean)
    }

    pub fn as_debt(&self) -> Option<&T> {
        match self {
            Self::Debt(d) => Some(d),
            _ => None,
        }
    }
}

/// A detected desynchronization between an OpenSpec change and git/release state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenSpecDesyncFinding {
    pub feature: String,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub reason: DesyncReason,
}

/// Root cause of an OpenSpec change desynchronization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesyncReason {
    ParentTasksCompleteSubtasksOpen,
    CodeMergedToMain,
    ReleaseVersionSurpassed(String),
}

/// A pending OpenSpec change that has been inactive beyond the configured threshold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StalePendingFinding {
    pub feature: String,
    pub days_inactive: u32,
    pub completed_tasks: usize,
    pub total_tasks: usize,
    pub source: InactivitySource,
}

/// Source used to determine inactivity timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InactivitySource {
    GitCommitDate,
    FilesystemMtime,
}

/// A drift finding in the `docs/solutions/` knowledge library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolutionDriftFinding {
    pub solution_path: String,
    pub dead_paths: Vec<String>,
    pub missing_frontmatter_fields: Vec<String>,
}

/// Umbrella report aggregating documentation technical debt findings across probes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocDebtReport {
    pub git_available: bool,
    pub openspec_desync: ProbeStatus<Vec<OpenSpecDesyncFinding>>,
    pub stale_pending: ProbeStatus<Vec<StalePendingFinding>>,
    pub solution_drift: ProbeStatus<Vec<SolutionDriftFinding>>,
    pub archive_compaction: ProbeStatus<ArchiveCompactionFinding>,
}

impl DocDebtReport {
    pub fn has_debt(&self) -> bool {
        self.openspec_desync.is_debt()
            || self.stale_pending.is_debt()
            || self.solution_drift.is_debt()
            || self.archive_compaction.is_debt()
    }

    pub fn summary_line(&self) -> String {
        if !self.has_debt() {
            if self.git_available {
                return "doc debt: clean".to_string();
            } else {
                return "doc debt: clean [git: n/a]".to_string();
            }
        }

        let mut parts = Vec::new();

        if let ProbeStatus::Debt(desync) = &self.openspec_desync {
            let count = desync.len();
            let all_subtasks_open = desync
                .iter()
                .all(|d| matches!(d.reason, DesyncReason::ParentTasksCompleteSubtasksOpen));
            let label = if all_subtasks_open {
                format!("{count} unarchived (open subtasks)")
            } else {
                format!("{count} unarchived (code merged)")
            };
            parts.push(label);
        }

        if let ProbeStatus::Debt(stale) = &self.stale_pending {
            let count = stale.len();
            let max_days = stale.iter().map(|s| s.days_inactive).max().unwrap_or(0);
            let spec_word = if count == 1 { "spec" } else { "specs" };
            if self.git_available {
                parts.push(format!("{count} stale {spec_word} ({max_days}d)"));
            } else {
                parts.push(format!("{count} stale {spec_word} [git: n/a]"));
            }
        }

        if let ProbeStatus::Debt(drift) = &self.solution_drift {
            let dead_count: usize = drift.iter().map(|d| d.dead_paths.len()).sum();
            let fm_count: usize = drift
                .iter()
                .filter(|d| !d.missing_frontmatter_fields.is_empty())
                .count();

            if dead_count > 0 {
                let link_word = if dead_count == 1 {
                    "dead solution link"
                } else {
                    "dead solution links"
                };
                parts.push(format!("{dead_count} {link_word}"));
            }
            if fm_count > 0 {
                let fm_word = if fm_count == 1 {
                    "solution missing frontmatter"
                } else {
                    "solutions missing frontmatter"
                };
                parts.push(format!("{fm_count} {fm_word}"));
            }
        }

        if let ProbeStatus::Debt(compact) = &self.archive_compaction {
            parts.push(format!(
                "{} uncompacted archive pkgs (>{} threshold)",
                compact.uncompacted_count, compact.threshold
            ));
        }

        if !self.git_available && !parts.iter().any(|p| p.contains("[git: n/a]")) {
            parts.push("[git: n/a]".to_string());
        }

        format!("doc debt: {}", parts.join(", "))
    }
}

impl Default for DocDebtReport {
    fn default() -> Self {
        Self {
            git_available: true,
            openspec_desync: ProbeStatus::Clean,
            stale_pending: ProbeStatus::Clean,
            solution_drift: ProbeStatus::Clean,
            archive_compaction: ProbeStatus::Clean,
        }
    }
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

/// Full 40-char HEAD SHA, used for stable code-review receipt comparison.
pub fn probe_git_head_full_sha(repo_root: &Path) -> Option<String> {
    resolve_commit_sha(repo_root, None)
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

/// Harnesses beyond OpenCode whose managed-tree manifests participate in
/// drift detection (`harness-manifest-sha256-coverage`).
pub(crate) const DRIFT_PROBE_HARNESSES: [crate::harness::HarnessKind; 2] = [
    crate::harness::HarnessKind::Claude,
    crate::harness::HarnessKind::Kimi,
];

/// Diffs one harness's install manifest against its managed tree on disk;
/// returns the number of drift actions. Zero when the manifest is missing,
/// malformed, or has no desired files.
fn manifest_drift_count_for(config_dir: &Path) -> usize {
    let Ok(manifest) = InstallManifest::load(config_dir) else {
        return 0;
    };
    let desired: BTreeMap<String, String> = manifest
        .files
        .into_iter()
        .map(|f| (f.path, f.sha256))
        .collect();
    if desired.is_empty() {
        return 0;
    }
    let managed_dir = config_dir.join(MANAGED_DIR);
    diff::diff(&desired, &desired, &managed_dir).actions.len()
}

pub fn probe_manifest_drift_count(ctx: &Context) -> usize {
    let mut total = manifest_drift_count_for(&ctx.opencode_config_dir);
    let home = crate::harness::home_dir_from_ctx(ctx);
    for kind in DRIFT_PROBE_HARNESSES {
        total += manifest_drift_count_for(&kind.harness_dir(&home));
    }
    total
}

pub fn probe_adoption_status(ctx: &Context) -> Option<AdoptionBlockStatus> {
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load(&state_path).ok()?;
    let repo_root = ctx.repo_root();
    let entry = state.project_for_path(&repo_root)?;
    let agents_file = repo_root.join(&entry.file);
    Some(check_adoption_block_status(&agents_file, entry.tier))
}

pub fn probe_adoption_tier(state: &State, repo_root: &Path) -> AdoptionTier {
    state
        .projects
        .iter()
        .find(|p| p.path == repo_root)
        .map(|p| p.tier)
        .unwrap_or(AdoptionTier::Full)
}

pub fn probe_execution_mode(
    repo_root: &Path,
    branch: Option<&str>,
    wf: &Option<WorkflowState>,
    tier: AdoptionTier,
    cli_override: Option<ExecutionMode>,
) -> ExecutionMode {
    if let Some(mode) = cli_override {
        if mode != ExecutionMode::Auto {
            return mode;
        }
    }

    // 1. OpenSpec precedence rule (R4b): if an unresolved directory exists in openspec/changes/ (excluding "archive")
    let openspec_changes_dir = repo_root.join("openspec").join("changes");
    if openspec_changes_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&openspec_changes_dir) {
            let has_active_openspec = entries.flatten().any(|e| {
                let p = e.path();
                p.is_dir() && e.file_name() != "archive"
            });
            if has_active_openspec {
                return ExecutionMode::Compound;
            }
        }
    }

    // 2. Git branch prefix heuristics (R3 / R4)
    if let Some(b) = branch {
        let b_clean = b.trim();
        if b_clean.starts_with("fix/")
            || b_clean.starts_with("chore/")
            || b_clean.starts_with("spike/")
            || b_clean.starts_with("test/")
        {
            return ExecutionMode::Organic;
        }
        if b_clean.starts_with("feat/") || b_clean.starts_with("spec/") {
            return ExecutionMode::Compound;
        }
    }

    // 3. Active odd/tasks/<feature>.md exists
    let odd_tasks_dir = repo_root.join("odd").join("tasks");
    if odd_tasks_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&odd_tasks_dir) {
            let has_odd_tasks = entries.flatten().any(|e| {
                let p = e.path();
                p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("md")
            });
            if has_odd_tasks {
                return ExecutionMode::Organic;
            }
        }
    }

    // 4. Stored execution_mode in WorkflowState if present
    if let Some(m) = wf.as_ref().and_then(|w| w.execution_mode) {
        if m != ExecutionMode::Auto {
            return m;
        }
    }

    // 5. Non-git workspace / tier fallback (R3b)
    if tier == AdoptionTier::Minimal {
        ExecutionMode::Organic
    } else {
        ExecutionMode::Compound
    }
}

pub fn probe_git_diff_loc(repo_root: &Path) -> usize {
    let out = match git_probe(repo_root, &["diff", "HEAD", "--numstat"]) {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => match git_probe(repo_root, &["diff", "--numstat"]) {
            Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => return 0,
        },
    };

    let mut total_loc = 0;
    for line in out.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let added: usize = parts[0].parse().unwrap_or(0);
            let deleted: usize = parts[1].parse().unwrap_or(0);
            total_loc += added + deleted;
        }
    }
    total_loc
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OddTaskContent {
    pub feature: String,
    pub problem_statement: String,
    pub guardrails: String,
    pub dod_items: Vec<OddDoDItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OddDoDItem {
    pub checked: bool,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OddTaskSummary {
    pub feature: String,
    pub path: PathBuf,
    pub completed_dod: usize,
    pub total_dod: usize,
}

pub fn generate_odd_task_template(feature: &str) -> String {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    format!(
        r#"---
feature: {feature}
mode: organic
created: {today}
status: active
---

# Problem Statement
<!-- Describe the observed symptom, error, or tactical friction -->

# Guardrails & Invariants
<!-- Non-negotiable safety, performance, and compatibility limits -->

# Definition of Done (DoD)
- [ ] Add reproduction test case
- [ ] Implement fix
- [ ] cargo clippy and cargo test pass cleanly
"#
    )
}

pub fn parse_odd_task_file(path: &Path) -> Result<OddTaskContent, CeError> {
    if !path.exists() {
        return Err(CeError::Usage(format!(
            "ODD task file '{}' does not exist",
            path.display()
        )));
    }
    let content = std::fs::read_to_string(path)?;
    parse_odd_task_content(&content, path)
}

pub fn parse_odd_task_content(content: &str, path: &Path) -> Result<OddTaskContent, CeError> {
    let default_feature = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("task")
        .to_string();

    let mut feature = default_feature;
    let mut body = content;

    if let Some(rest) = content.strip_prefix("---") {
        if let Some(end_idx) = rest.find("---") {
            let frontmatter = &rest[..end_idx];
            body = rest[end_idx + 3..].trim();
            for line in frontmatter.lines() {
                let trimmed = line.trim();
                if let Some(val) = trimmed.strip_prefix("feature:") {
                    let feat_val = val.trim().trim_matches('"').trim_matches('\'');
                    if !feat_val.is_empty() {
                        feature = feat_val.to_string();
                    }
                }
            }
        }
    }

    let mut problem_statement = String::new();
    let mut guardrails = String::new();
    let mut dod_items = Vec::new();

    let mut current_section = "";
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(stripped) = trimmed.strip_prefix("# ") {
            let title = stripped.trim().to_lowercase();
            if title.contains("problem") {
                current_section = "problem";
            } else if title.contains("guardrail") || title.contains("invariant") {
                current_section = "guardrails";
            } else if title.contains("definition of done") || title.contains("dod") {
                current_section = "dod";
            } else {
                current_section = "other";
            }
            continue;
        }

        match current_section {
            "problem" => {
                if !problem_statement.is_empty() {
                    problem_statement.push('\n');
                }
                problem_statement.push_str(line);
            }
            "guardrails" => {
                if !guardrails.is_empty() {
                    guardrails.push('\n');
                }
                guardrails.push_str(line);
            }
            "dod" => {
                if trimmed.starts_with("- [ ] ") || trimmed.starts_with("- [ ]") {
                    let item_text = if trimmed.len() > 5 {
                        trimmed[5..].trim().to_string()
                    } else {
                        String::new()
                    };
                    dod_items.push(OddDoDItem {
                        checked: false,
                        title: item_text,
                    });
                } else if trimmed.starts_with("- [x] ")
                    || trimmed.starts_with("- [x]")
                    || trimmed.starts_with("- [X] ")
                    || trimmed.starts_with("- [X]")
                {
                    let item_text = if trimmed.len() > 5 {
                        trimmed[5..].trim().to_string()
                    } else {
                        String::new()
                    };
                    dod_items.push(OddDoDItem {
                        checked: true,
                        title: item_text,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(OddTaskContent {
        feature,
        problem_statement: problem_statement.trim().to_string(),
        guardrails: guardrails.trim().to_string(),
        dod_items,
    })
}

pub fn probe_active_odd_task(
    repo_root: &Path,
    wf: &Option<WorkflowState>,
) -> Option<OddTaskSummary> {
    let odd_tasks_dir = repo_root.join("odd").join("tasks");
    if !odd_tasks_dir.is_dir() {
        return None;
    }

    if let Some(feat) = wf.as_ref().and_then(|w| w.feature_name.as_deref()) {
        let direct_file = odd_tasks_dir.join(format!("{feat}.md"));
        if direct_file.is_file() {
            if let Ok(parsed) = parse_odd_task_file(&direct_file) {
                let total = parsed.dod_items.len();
                let completed = parsed.dod_items.iter().filter(|i| i.checked).count();
                return Some(OddTaskSummary {
                    feature: parsed.feature,
                    path: direct_file,
                    completed_dod: completed,
                    total_dod: total,
                });
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir(&odd_tasks_dir) {
        let mut md_files: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("md"))
            .collect();
        md_files.sort();
        if let Some(file) = md_files.into_iter().next() {
            if let Ok(parsed) = parse_odd_task_file(&file) {
                let total = parsed.dod_items.len();
                let completed = parsed.dod_items.iter().filter(|i| i.checked).count();
                return Some(OddTaskSummary {
                    feature: parsed.feature,
                    path: file,
                    completed_dod: completed,
                    total_dod: total,
                });
            }
        }
    }
    None
}

pub fn run_graduate(ctx: &Context, args: &GraduateArgs) -> Result<(), CeError> {
    let repo_root = ctx.repo_root();
    let branch = probe_git_branch(&repo_root);
    let state_path = ctx.config_dir.join("state.json");
    let state = if state_path.exists() {
        State::load(&state_path)?
    } else {
        State::default()
    };
    let wf = state.current_workflow_for_branch(&repo_root, branch.as_deref());

    let feature_name = if let Some(f) = &args.feature {
        f.trim().to_string()
    } else if let Some(f) = wf.as_ref().and_then(|w| w.feature_name.clone()) {
        f
    } else if let Some(odd) = probe_active_odd_task(&repo_root, &wf) {
        odd.feature
    } else {
        return Err(CeError::Usage(
            "specify a feature name to graduate, e.g. 'ce-ai workflow graduate <feature>'"
                .to_string(),
        ));
    };

    if feature_name.is_empty() {
        return Err(CeError::Usage("feature name cannot be empty".to_string()));
    }

    let source_file = repo_root
        .join("odd")
        .join("tasks")
        .join(format!("{feature_name}.md"));
    if !source_file.is_file() {
        return Err(CeError::Usage(format!(
            "ODD source task file not found at '{}'",
            source_file.display()
        )));
    }

    let dest_dir = repo_root
        .join("openspec")
        .join("changes")
        .join(&feature_name);
    if dest_dir.exists() {
        return Err(CeError::State(format!(
            "OpenSpec change directory already exists at '{}'. Refusing to overwrite.",
            dest_dir.display()
        )));
    }

    let parsed = parse_odd_task_file(&source_file)?;

    std::fs::create_dir_all(&dest_dir)?;

    let rel_source = source_file
        .strip_prefix(&repo_root)
        .unwrap_or(&source_file)
        .display();
    let problem_text = if parsed.problem_statement.is_empty() {
        "No problem statement provided in ODD task brief."
    } else {
        &parsed.problem_statement
    };
    let proposal_content = format!(
        "# Proposal: {feature_name}\n\n## Problem Statement\n{problem_text}\n\n## In-Scope\n- Promoted from Organic Task `{rel_source}`.\n- Implement all Definition of Done deliverables.\n\n## Out-of-Scope\n- Unrelated architectural refactorings outside the scoped problem statement.\n"
    );
    std::fs::write(dest_dir.join("proposal.md"), proposal_content)?;

    let guardrails_text = if parsed.guardrails.is_empty() {
        "No explicit guardrails defined in ODD task brief."
    } else {
        &parsed.guardrails
    };
    let spec_content = format!(
        "# Specification: {feature_name}\n\n## Requirements & Guardrails\n{guardrails_text}\n"
    );
    std::fs::write(dest_dir.join("spec.md"), spec_content)?;

    let mut tasks_content = format!("# Tasks: {feature_name}\n\n- [ ] **Work Unit 1: Implementation & Verification** (~200 LOC)\n");
    if parsed.dod_items.is_empty() {
        tasks_content.push_str("  - [ ] Implement deliverables\n  - [ ] Run test suite\n");
    } else {
        for item in &parsed.dod_items {
            let mark = if item.checked { "x" } else { " " };
            tasks_content.push_str(&format!("  - [{mark}] {}\n", item.title));
        }
    }
    std::fs::write(dest_dir.join("tasks.md"), tasks_content)?;

    std::fs::remove_file(&source_file)?;

    let mut new_state = state;
    let key = State::workspace_branch_key(&repo_root, branch.as_deref());
    let new_wf = WorkflowState {
        stage: WorkflowStage::WorkTdd,
        task: format!("Working on {feature_name} (graduated from ODD)"),
        feature_name: Some(feature_name.clone()),
        updated_at: chrono::Utc::now().to_rfc3339(),
        source: WorkflowSource::Manual,
        resolution: branch.as_ref().map(|_| FeatureResolution::Branch),
        new_cycle: false,
        execution_mode: Some(ExecutionMode::Compound),
    };
    new_state.workflows.insert(key, new_wf.clone());
    new_state.workflow = Some(new_wf);
    if !ctx.dry_run {
        new_state.save(&state_path)?;
    }

    println!("workflow: graduated ODD task '{feature_name}' to OpenSpec successfully!");
    println!("  source removed: {}", source_file.display());
    println!("  package created: {}", dest_dir.display());
    println!("  files generated: proposal.md, spec.md, tasks.md");
    println!("  stage transitioned: Stage 4 (Work/TDD)");
    println!("  execution mode: compound");

    Ok(())
}

pub fn probe_repo_state(ctx: &Context, wf: &Option<WorkflowState>) -> RepoState {
    probe_repo_state_with_mode(ctx, wf, None)
}

pub fn probe_repo_state_with_mode(
    ctx: &Context,
    wf: &Option<WorkflowState>,
    cli_mode: Option<ExecutionMode>,
) -> RepoState {
    let repo_root = ctx.repo_root();
    let git_branch = probe_git_branch(&repo_root);
    let head_sha = probe_git_head_sha(&repo_root);
    let head_full_sha = probe_git_head_full_sha(&repo_root);
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

    let unarchived_completed_changes = probe_unarchived_completed_changes(&repo_root);

    let commits_ahead = probe_commits_ahead(&repo_root);
    let stage6_artifact_present = probe_stage6_artifact(&repo_root, &modified_files);
    let review_receipt = State::load(&ctx.config_dir.join("state.json"))
        .ok()
        .and_then(|s| {
            s.review_receipt_for_branch(&repo_root, git_branch.as_deref())
                .cloned()
        });

    let doc_hygiene_cfg =
        State::load_with_workspace_overrides(&ctx.config_dir.join("state.json"), Some(&repo_root))
            .ok()
            .map(|s| s.doc_hygiene())
            .unwrap_or_default();
    let doc_debt = Some(probe_doc_debt(
        &repo_root,
        git_branch.as_deref(),
        &doc_hygiene_cfg,
    ));

    let tier = State::load(&ctx.config_dir.join("state.json"))
        .ok()
        .map(|s| probe_adoption_tier(&s, &repo_root))
        .unwrap_or(AdoptionTier::Full);

    let execution_mode =
        probe_execution_mode(&repo_root, git_branch.as_deref(), wf, tier, cli_mode);

    RepoState {
        git_branch,
        head_sha,
        head_full_sha,
        is_git_clean,
        modified_files,
        manifest_drift_count,
        adoption_status,
        openspec_context,
        task_desync,
        unarchived_completed_changes,
        commits_ahead,
        stage6_artifact_present,
        review_receipt,
        doc_debt,
        execution_mode,
    }
}

pub fn probe_openspec_has_uncommitted(repo_root: &Path, feature: &str) -> bool {
    let change_rel_path = format!("openspec/changes/{feature}");
    let mut cmd = std::process::Command::new("git");
    for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        cmd.env_remove(var);
    }
    let out = match cmd
        .args(["status", "--porcelain=v1", "-uall", "--", &change_rel_path])
        .current_dir(repo_root)
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return false,
    };

    let stdout = String::from_utf8_lossy(&out.stdout);
    !stdout.trim().is_empty()
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<FeatureResolution>,
    #[serde(default)]
    pub is_uncommitted: bool,
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

    let (target_feature, resolution) = if let Some(branch_feat) = probe_git_branch(repo_root)
        .map(|b| sanitize_feature_name(&b))
        .filter(|f| openspec_dir.join(f).is_dir())
    {
        (branch_feat, Some(FeatureResolution::Branch))
    } else if let Some(feat) = wf
        .as_ref()
        .and_then(|w| w.feature_name.clone())
        .filter(|f| !f.trim().is_empty() && openspec_dir.join(f).is_dir())
    {
        let res = wf.as_ref().and_then(|w| w.resolution);
        (feat, res)
    } else {
        // Fallback: find most recently modified directory in openspec/changes/
        let mut entries: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
        if let Ok(read) = std::fs::read_dir(&openspec_dir) {
            for entry in read.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str == "archive" || name_str.starts_with('.') {
                    continue;
                }
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
        (
            path.file_name()?.to_string_lossy().to_string(),
            Some(FeatureResolution::MtimeFallback),
        )
    };

    let change_dir = openspec_dir.join(&target_feature);
    if !change_dir.is_dir() {
        return None;
    }

    let has_proposal = change_dir.join("proposal.md").exists();
    let has_spec = change_dir.join("spec.md").exists();
    let tasks_path = change_dir.join("tasks.md");
    let has_tasks = tasks_path.exists();

    let (completed_tasks, total_tasks) = if has_tasks {
        count_task_checkboxes(&tasks_path)
    } else {
        (0, 0)
    };

    let is_uncommitted = probe_openspec_has_uncommitted(repo_root, &target_feature);

    Some(OpenSpecContextInfo {
        feature: target_feature,
        path: change_dir,
        has_proposal,
        has_spec,
        has_tasks,
        completed_tasks,
        total_tasks,
        resolution,
        is_uncommitted,
    })
}

/// Parses tasks content string and returns (completed_tasks, total_tasks).
pub fn count_task_checkboxes_content(content: &str) -> (usize, usize) {
    let mut completed_tasks = 0;
    let mut total_tasks = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            completed_tasks += 1;
            total_tasks += 1;
        } else if trimmed.starts_with("- [ ]") {
            total_tasks += 1;
        }
    }
    (completed_tasks, total_tasks)
}

/// Parses a tasks.md file and returns (completed_tasks, total_tasks).
/// Gracefully returns (0, 0) if the file cannot be read or does not exist.
pub fn count_task_checkboxes(tasks_path: &Path) -> (usize, usize) {
    if let Ok(content) = std::fs::read_to_string(tasks_path) {
        count_task_checkboxes_content(&content)
    } else {
        (0, 0)
    }
}

/// Scans openspec/changes/ across the entire repository for completed features that have not been moved to archive/.
/// Gracefully ignores unreadable directories/files (TOCTOU resilience).
pub fn probe_unarchived_completed_changes(repo_root: &Path) -> Vec<UnarchivedChange> {
    let openspec_dir = repo_root.join("openspec").join("changes");
    let mut completed_changes = Vec::new();
    let entries = match std::fs::read_dir(&openspec_dir) {
        Ok(read) => read,
        Err(_) => return completed_changes,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if dir_name == "archive" {
            continue;
        }

        let tasks_path = path.join("tasks.md");
        if !tasks_path.is_file() {
            continue;
        }

        let (completed, total) = count_task_checkboxes(&tasks_path);
        if total > 0 && completed == total {
            completed_changes.push(UnarchivedChange {
                feature: dir_name.to_string(),
                completed_tasks: completed,
                total_tasks: total,
            });
        }
    }

    completed_changes.sort_by(|a, b| a.feature.cmp(&b.feature));
    completed_changes
}

/// Inspects tasks content to determine whether all top-level (parent) tasks are completed
/// while one or more subtasks remain uncompleted.
pub fn is_parent_tasks_complete_subtasks_open_content(content: &str) -> bool {
    let mut parent_total = 0;
    let mut parent_completed = 0;
    let mut has_open_subtask = false;

    for line in content.lines() {
        if line.starts_with("- [") || line.starts_with("* [") {
            let rest = &line[2..];
            if rest.starts_with("[x]") || rest.starts_with("[X]") {
                parent_total += 1;
                parent_completed += 1;
            } else if rest.starts_with("[ ]") {
                parent_total += 1;
            }
        } else {
            let trimmed = line.trim_start();
            if trimmed.starts_with("- [ ]") || trimmed.starts_with("* [ ]") {
                has_open_subtask = true;
            }
        }
    }

    parent_total > 0 && parent_completed == parent_total && has_open_subtask
}

/// Inspects tasks.md to determine whether all top-level (parent) tasks are completed
/// while one or more subtasks remain uncompleted.
pub fn is_parent_tasks_complete_subtasks_open(tasks_path: &Path) -> bool {
    if let Ok(content) = std::fs::read_to_string(tasks_path) {
        is_parent_tasks_complete_subtasks_open_content(&content)
    } else {
        false
    }
}

/// Helper to parse a 3-part SemVer token (stripping leading 'v' and boundary punctuation).
fn parse_semver_token(token: &str) -> Option<(String, (u32, u32, u32))> {
    let clean = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
    let version_str = clean.strip_prefix('v').unwrap_or(clean);
    let parts: Vec<&str> = version_str.split('.').collect();
    if parts.len() == 3 {
        if let (Ok(maj), Ok(min), Ok(patch)) = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
        ) {
            return Some((version_str.to_string(), (maj, min, patch)));
        }
    }
    None
}

/// Extracts the version declared in the repository's `Cargo.toml`.
pub fn extract_cargo_version(repo_root: &Path) -> Option<(u32, u32, u32)> {
    let cargo_path = repo_root.join("Cargo.toml");
    let content = std::fs::read_to_string(cargo_path).ok()?;
    let mut in_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            in_package = true;
        } else if trimmed.starts_with('[') {
            in_package = false;
        } else if in_package && trimmed.starts_with("version") {
            if let Some(val) = trimmed.split('=').nth(1) {
                let clean = val.trim().trim_matches('"').trim_matches('\'');
                if let Some((_, ver)) = parse_semver_token(clean) {
                    return Some(ver);
                }
            }
        }
    }
    None
}

/// Extracts any release version cited in tasks.md (e.g. "bump version to 1.50.1" or "Release tag v1.50.1").
pub fn extract_cited_version(tasks_content: &str) -> Option<(String, (u32, u32, u32))> {
    let mut highest: Option<(String, (u32, u32, u32))> = None;
    for line in tasks_content.lines() {
        let lower = line.to_lowercase();
        if lower.contains("version")
            || lower.contains("release")
            || lower.contains("tag")
            || lower.contains("bump")
        {
            for word in line.split_whitespace() {
                if let Some((v_str, v_tuple)) = parse_semver_token(word) {
                    match &highest {
                        Some((_, cur_tuple)) if &v_tuple > cur_tuple => {
                            highest = Some((v_str, v_tuple));
                        }
                        None => {
                            highest = Some((v_str, v_tuple));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    highest
}

/// Checks whether git history on the base branch contains commits referencing the feature.
fn check_feature_merged_to_main(repo_root: &Path, feature: &str) -> bool {
    for target in ["origin/main", "main", "origin/master", "master"] {
        if let Some(out) = git_probe(
            repo_root,
            &["log", target, "-n", "100", "--grep", feature, "--format=%H"],
        ) {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if !stdout.trim().is_empty() {
                    return true;
                }
            }
        }
    }
    false
}

/// Probes documentation technical debt across OpenSpec synchronization, pending inactivity, and solution library drift.
pub fn probe_doc_debt(
    repo_root: &Path,
    git_branch: Option<&str>,
    config: &DocHygieneConfig,
) -> DocDebtReport {
    let git_available = git_branch.is_some()
        || repo_root.join(".git").exists()
        || git_probe(repo_root, &["rev-parse", "--git-dir"])
            .map(|o| o.status.success())
            .unwrap_or(false);

    let openspec_desync = probe_openspec_desync(repo_root, git_available);
    let stale_pending =
        probe_stale_pending_openspecs(repo_root, config.stale_spec_days, git_available);
    let solution_drift = probe_solution_drift(repo_root, config);
    let archive_compaction = probe_archive_compaction(repo_root, config);

    DocDebtReport {
        git_available,
        openspec_desync,
        stale_pending,
        solution_drift,
        archive_compaction,
    }
}

/// Probe 1: Scans `openspec/changes/` for desynchronized changes where tasks remain unchecked
/// but top-level parent tasks are complete, code was merged to main, or the cited version was surpassed.
pub fn probe_openspec_desync(
    repo_root: &Path,
    git_available: bool,
) -> ProbeStatus<Vec<OpenSpecDesyncFinding>> {
    let openspec_dir = repo_root.join("openspec").join("changes");
    let entries = match std::fs::read_dir(&openspec_dir) {
        Ok(read) => read,
        Err(_) => {
            return if git_available {
                ProbeStatus::Clean
            } else {
                ProbeStatus::Unknown
            };
        }
    };

    let mut findings = Vec::new();
    let cargo_version = extract_cargo_version(repo_root);

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if dir_name == "archive" || dir_name.starts_with('.') {
            continue;
        }

        let tasks_path = path.join("tasks.md");
        let content = match std::fs::read_to_string(&tasks_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (completed, total) = count_task_checkboxes_content(&content);
        if total == 0 || completed == total {
            continue;
        }

        // 1. Parent tasks complete with subtasks open
        if is_parent_tasks_complete_subtasks_open_content(&content) {
            findings.push(OpenSpecDesyncFinding {
                feature: dir_name.to_string(),
                completed_tasks: completed,
                total_tasks: total,
                reason: DesyncReason::ParentTasksCompleteSubtasksOpen,
            });
            continue;
        }

        // 2. Git commits merged to main/HEAD (only when git is available)
        if git_available && check_feature_merged_to_main(repo_root, dir_name) {
            findings.push(OpenSpecDesyncFinding {
                feature: dir_name.to_string(),
                completed_tasks: completed,
                total_tasks: total,
                reason: DesyncReason::CodeMergedToMain,
            });
            continue;
        }

        // 3. Cited version surpassed in Cargo.toml
        if let Some(c_ver) = cargo_version {
            if let Some((cited_str, cited_ver)) = extract_cited_version(&content) {
                if cited_ver < c_ver {
                    findings.push(OpenSpecDesyncFinding {
                        feature: dir_name.to_string(),
                        completed_tasks: completed,
                        total_tasks: total,
                        reason: DesyncReason::ReleaseVersionSurpassed(cited_str),
                    });
                }
            }
        }
    }

    findings.sort_by(|a, b| a.feature.cmp(&b.feature));

    if !findings.is_empty() {
        ProbeStatus::Debt(findings)
    } else if git_available {
        ProbeStatus::Clean
    } else {
        ProbeStatus::Unknown
    }
}

/// Probe 2: Scans `openspec/changes/` for pending changes whose last activity exceeds `stale_days`.
/// Uses git commit timestamps when git is available; falls back to filesystem mtime in No-Git environments.
pub fn probe_stale_pending_openspecs(
    repo_root: &Path,
    stale_days: u32,
    git_available: bool,
) -> ProbeStatus<Vec<StalePendingFinding>> {
    let openspec_dir = repo_root.join("openspec").join("changes");
    let entries = match std::fs::read_dir(&openspec_dir) {
        Ok(read) => read,
        Err(_) => return ProbeStatus::Clean,
    };

    let mut findings = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => continue,
        };
        if dir_name == "archive" || dir_name.starts_with('.') {
            continue;
        }

        let tasks_path = path.join("tasks.md");
        if !tasks_path.is_file() {
            continue;
        }

        let (completed, total) = count_task_checkboxes(&tasks_path);
        if total == 0 || completed == total {
            continue;
        }

        let mut days_inactive = 0;
        let mut source = InactivitySource::FilesystemMtime;

        if git_available {
            let change_rel = format!("openspec/changes/{dir_name}");
            if let Some(out) =
                git_probe(repo_root, &["log", "-1", "--format=%ct", "--", &change_rel])
            {
                if out.status.success() {
                    let ts_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if let Ok(ts) = ts_str.parse::<i64>() {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        if now > ts {
                            days_inactive = ((now - ts) / 86400) as u32;
                            source = InactivitySource::GitCommitDate;
                        }
                    }
                }
            }
        }

        if source != InactivitySource::GitCommitDate {
            source = InactivitySource::FilesystemMtime;
            if let Ok(metadata) = std::fs::metadata(&tasks_path) {
                if let Ok(mtime) = metadata.modified() {
                    let now = std::time::SystemTime::now();
                    if let Ok(duration) = now.duration_since(mtime) {
                        days_inactive = (duration.as_secs() / 86400) as u32;
                    }
                }
            }
        }

        if days_inactive >= stale_days {
            findings.push(StalePendingFinding {
                feature: dir_name.to_string(),
                days_inactive,
                completed_tasks: completed,
                total_tasks: total,
                source,
            });
        }
    }

    findings.sort_by(|a, b| a.feature.cmp(&b.feature));

    if !findings.is_empty() {
        ProbeStatus::Debt(findings)
    } else {
        ProbeStatus::Clean
    }
}

/// Probe 5: Scans `docs/solutions/` for dead code references and missing YAML frontmatter fields.
pub fn probe_solution_drift(
    repo_root: &Path,
    config: &DocHygieneConfig,
) -> ProbeStatus<Vec<SolutionDriftFinding>> {
    if !config.check_solution_paths && !config.require_solution_frontmatter {
        return ProbeStatus::Clean;
    }

    let solutions_dir = repo_root.join("docs").join("solutions");
    if !solutions_dir.is_dir() {
        return ProbeStatus::Clean;
    }

    let mut solution_files = Vec::new();
    collect_solution_files(&solutions_dir, &mut solution_files);
    solution_files.sort();

    let mut findings = Vec::new();

    for file_path in solution_files {
        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let rel_solution_path = file_path
            .strip_prefix(repo_root)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .replace('\\', "/");

        let missing_frontmatter_fields = if config.require_solution_frontmatter {
            check_solution_frontmatter(&content)
        } else {
            Vec::new()
        };

        let dead_paths = if config.check_solution_paths {
            check_solution_dead_paths(repo_root, &content)
        } else {
            Vec::new()
        };

        if !missing_frontmatter_fields.is_empty() || !dead_paths.is_empty() {
            findings.push(SolutionDriftFinding {
                solution_path: rel_solution_path,
                dead_paths,
                missing_frontmatter_fields,
            });
        }
    }

    findings.sort_by(|a, b| a.solution_path.cmp(&b.solution_path));

    if !findings.is_empty() {
        ProbeStatus::Debt(findings)
    } else {
        ProbeStatus::Clean
    }
}

fn collect_solution_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_solution_files(&path, files);
            } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                files.push(path);
            }
        }
    }
}

fn extract_yaml_frontmatter(content: &str) -> Option<Vec<&str>> {
    let mut lines = content.trim_start().lines();
    let first = lines.next()?.trim();
    if first != "---" {
        return None;
    }
    let mut header = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            return Some(header);
        }
        header.push(line);
    }
    None
}

fn check_solution_frontmatter(content: &str) -> Vec<String> {
    let mut missing = Vec::new();
    let header_lines = match extract_yaml_frontmatter(content) {
        Some(lines) => lines,
        None => {
            return vec![
                "title".into(),
                "category".into(),
                "problem_type".into(),
                "tags".into(),
                "applies_when".into(),
            ];
        }
    };

    let mut has_title = false;
    let mut has_category_or_module = false;
    let mut has_problem_type = false;
    let mut has_tags = false;
    let mut has_applies_when = false;

    for line in header_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') {
            if let Some((k, _)) = trimmed.split_once(':') {
                let key = k.trim().to_lowercase();
                match key.as_str() {
                    "title" => has_title = true,
                    "category" | "module" => has_category_or_module = true,
                    "problem_type" => has_problem_type = true,
                    "tags" => has_tags = true,
                    "applies_when" => has_applies_when = true,
                    _ => {}
                }
            }
        }
    }

    if !has_title {
        missing.push("title".into());
    }
    if !has_category_or_module {
        missing.push("category".into());
    }
    if !has_problem_type {
        missing.push("problem_type".into());
    }
    if !has_tags {
        missing.push("tags".into());
    }
    if !has_applies_when {
        missing.push("applies_when".into());
    }

    missing
}

fn clean_code_path(token: &str) -> Option<&str> {
    let trimmed = token.trim();
    if trimmed.contains('*') || trimmed.contains('<') || trimmed.contains('>') {
        return None;
    }
    if !trimmed.starts_with("src/") && !trimmed.starts_with("tests/") {
        return None;
    }

    let path_no_anchor = trimmed.split_once('#').map_or(trimmed, |(base, _)| base);
    let candidate = match path_no_anchor.split_once(':') {
        Some((base, suffix))
            if suffix
                .chars()
                .all(|c| c.is_ascii_digit() || c == '-' || c == ',') =>
        {
            base
        }
        _ => path_no_anchor,
    };

    if candidate.ends_with(".rs") {
        Some(candidate)
    } else {
        None
    }
}

fn check_solution_dead_paths(repo_root: &Path, content: &str) -> Vec<String> {
    let mut dead = Vec::new();
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        let mut start = None;
        for (i, c) in line.char_indices() {
            if c == '`' {
                if let Some(s) = start {
                    let token = &line[s + 1..i];
                    if let Some(cleaned) = clean_code_path(token) {
                        let target = repo_root.join(cleaned);
                        if !target.exists() {
                            dead.push(cleaned.to_string());
                        }
                    }
                    start = None;
                } else {
                    start = Some(i);
                }
            }
        }
    }

    dead.sort();
    dead.dedup();
    dead
}

/// Criterion met for archiving an OpenSpec change package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveCriterion {
    Mechanical {
        completed: usize,
        total: usize,
    },
    StatusAttested {
        status: String,
        completed: usize,
        total: usize,
    },
}

/// The result of an archival operation on a feature package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveOutcome {
    pub feature: String,
    pub source_path: PathBuf,
    pub dest_path: PathBuf,
    pub criterion: ArchiveCriterion,
}

/// Validates criteria and moves an OpenSpec change folder to `openspec/changes/archive/`.
pub fn validate_and_archive_feature(
    repo_root: &Path,
    feature: &str,
    status: Option<&str>,
    dry_run: bool,
) -> Result<ArchiveOutcome, CeError> {
    if feature.is_empty()
        || feature == "."
        || feature == ".."
        || feature.contains('/')
        || feature.contains('\\')
    {
        return Err(CeError::Usage(format!("invalid feature name '{feature}'")));
    }

    let source_path = repo_root.join("openspec").join("changes").join(feature);
    if !source_path.is_dir() {
        return Err(CeError::Usage(format!(
            "openspec change '{feature}' not found at '{}'",
            source_path.display()
        )));
    }

    let dest_parent = repo_root.join("openspec").join("changes").join("archive");
    let dest_path = dest_parent.join(feature);
    if dest_path.exists() {
        return Err(CeError::State(format!(
            "cannot archive '{feature}': destination '{}' already exists",
            dest_path.display()
        )));
    }

    let tasks_path = source_path.join("tasks.md");
    let (completed, total) = if tasks_path.is_file() {
        count_task_checkboxes(&tasks_path)
    } else {
        (0, 0)
    };

    let criterion = if total > 0 && completed == total {
        ArchiveCriterion::Mechanical { completed, total }
    } else if let Some(ref_status) = status {
        let trimmed = ref_status.trim();
        if trimmed.is_empty() || trimmed.len() < 5 {
            return Err(CeError::Usage(
                "status attestation must be at least 5 characters citing release evidence (Criterion 2)"
                    .to_string(),
            ));
        }
        ArchiveCriterion::StatusAttested {
            status: trimmed.to_string(),
            completed,
            total,
        }
    } else {
        return Err(CeError::Verification(format!(
            "openspec change '{feature}' has {} open task(s) ({completed}/{total} completed); complete all tasks (Criterion 1) or supply --status '<evidence>' (Criterion 2)",
            total.saturating_sub(completed)
        )));
    };

    let (is_clean, modified_files) = probe_git_dirty_files(repo_root);
    if !is_clean {
        let feature_prefix = format!("openspec/changes/{feature}/");
        let foreign_modified: Vec<&String> = modified_files
            .iter()
            .filter(|f| f.starts_with(&feature_prefix) && !f.ends_with("tasks.md"))
            .collect();
        if !foreign_modified.is_empty() {
            return Err(CeError::Verification(format!(
                "openspec change '{feature}' contains uncommitted modifications in non-tasks files: {foreign_modified:?}"
            )));
        }
    }

    if !dry_run {
        // Prepend status to tasks.md if Criterion 2
        if let ArchiveCriterion::StatusAttested { status: ref st, .. } = &criterion {
            if tasks_path.is_file() {
                let current_content = std::fs::read_to_string(&tasks_path).map_err(CeError::Io)?;
                if !current_content.trim_start().starts_with("> STATUS:") {
                    let new_content = format!("> STATUS: {st}\n\n{current_content}");
                    crate::state::write_atomic(&tasks_path, new_content.as_bytes())?;
                }
            }
        }

        std::fs::create_dir_all(&dest_parent).map_err(CeError::Io)?;

        // Try git mv first
        let git_mv_success = std::process::Command::new("git")
            .args([
                "mv",
                source_path.to_str().unwrap_or_default(),
                dest_path.to_str().unwrap_or_default(),
            ])
            .current_dir(repo_root)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !git_mv_success {
            std::fs::rename(&source_path, &dest_path).map_err(CeError::Io)?;
            let _ = std::process::Command::new("git")
                .args(["add", "-A", "openspec/changes"])
                .current_dir(repo_root)
                .output();
        }
    }

    Ok(ArchiveOutcome {
        feature: feature.to_string(),
        source_path,
        dest_path,
        criterion,
    })
}

/// Updates openspec/changes/archive/README.md with records of archived features.
pub fn sync_archive_readme_ledger(
    repo_root: &Path,
    outcomes: &[ArchiveOutcome],
    dry_run: bool,
) -> Result<(), CeError> {
    if dry_run || outcomes.is_empty() {
        return Ok(());
    }
    let readme_path = repo_root
        .join("openspec")
        .join("changes")
        .join("archive")
        .join("README.md");
    if !readme_path.is_file() {
        return Ok(());
    }

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let entry = if outcomes.len() == 1 {
        let o = &outcomes[0];
        match &o.criterion {
            ArchiveCriterion::Mechanical { completed, total } => {
                format!(
                    "- {}: archived ({completed}/{total} tasks) under criterion (1) on {today}.\n",
                    o.feature
                )
            }
            ArchiveCriterion::StatusAttested {
                status,
                completed,
                total,
            } => {
                format!(
                    "- {}: archived ({completed}/{total} tasks, STATUS: {status}) under criterion (2) on {today}.\n",
                    o.feature
                )
            }
        }
    } else {
        let count = outcomes.len();
        format!("- {today} sweep: {count} folders archived via 'ce-ai archive --all'.\n")
    };

    let current = std::fs::read_to_string(&readme_path).map_err(CeError::Io)?;
    let mut updated = current;
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&entry);

    crate::state::write_atomic(&readme_path, updated.as_bytes())?;
    let _ = std::process::Command::new("git")
        .args(["add", "openspec/changes/archive/README.md"])
        .current_dir(repo_root)
        .output();

    Ok(())
}

/// Reconciles state.json by clearing the active feature pointer if it was archived.
pub fn reconcile_state_active_feature(
    ctx: &Context,
    archived_features: &[String],
    dry_run: bool,
) -> Result<(), CeError> {
    if dry_run || archived_features.is_empty() {
        return Ok(());
    }

    let state_path = ctx.config_dir.join("state.json");
    let mut state = match State::load(&state_path) {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };

    let mut changed = false;

    if let Some(wf) = &mut state.workflow {
        if let Some(name) = &wf.feature_name {
            if archived_features.contains(name) {
                wf.feature_name = None;
                changed = true;
            }
        }
    }

    for wf in state.workflows.values_mut() {
        if let Some(name) = &wf.feature_name {
            if archived_features.contains(name) {
                wf.feature_name = None;
                changed = true;
            }
        }
    }

    if changed {
        state.save(&state_path)?;
    }

    Ok(())
}

/// Executes the archive command workflow across single feature or batch mode.
pub fn run_archive(ctx: &Context, args: &ArchiveArgs) -> Result<(), CeError> {
    if let Some(ArchiveSubcommand::Compact(compact_args)) = &args.subcommand {
        return run_archive_compact(ctx, compact_args);
    }

    let repo_root = ctx.repo_root();
    let state_path = ctx.config_dir.join("state.json");
    let state = State::load(&state_path).ok();
    let branch = probe_git_branch(&repo_root);

    let mut outcomes = Vec::new();

    if args.all {
        let unarchived = probe_unarchived_completed_changes(&repo_root);
        if unarchived.is_empty() {
            println!("archive: no completed OpenSpec changes pending archival");
            return Ok(());
        }
        for item in &unarchived {
            match validate_and_archive_feature(
                &repo_root,
                &item.feature,
                args.status.as_deref(),
                args.dry_run,
            ) {
                Ok(outcome) => {
                    outcomes.push(outcome);
                }
                Err(err) => {
                    eprintln!("warning: skipping '{}': {err}", item.feature);
                }
            }
        }
        if outcomes.is_empty() {
            return Err(CeError::Verification(
                "no features could be archived successfully".to_string(),
            ));
        }
    } else {
        let target_feature = if let Some(f) = &args.feature {
            f.clone()
        } else if let Some(st) = &state {
            if let Some(wf) = st.current_workflow_for_branch(&repo_root, branch.as_deref()) {
                if let Some(f) = wf.feature_name {
                    f
                } else {
                    return Err(CeError::Usage(
                        "no target feature specified and no active feature recorded in state"
                            .to_string(),
                    ));
                }
            } else {
                return Err(CeError::Usage(
                    "no target feature specified and no active workflow recorded in state"
                        .to_string(),
                ));
            }
        } else {
            return Err(CeError::Usage(
                "no target feature specified and no state available".to_string(),
            ));
        };

        let outcome = validate_and_archive_feature(
            &repo_root,
            &target_feature,
            args.status.as_deref(),
            args.dry_run,
        )?;
        outcomes.push(outcome);
    }

    if args.dry_run {
        println!(
            "dry-run: would archive {} completed change(s):",
            outcomes.len()
        );
        for o in &outcomes {
            match &o.criterion {
                ArchiveCriterion::Mechanical { completed, total } => {
                    println!(
                        "  - {} ({}/{} tasks) -> {}",
                        o.feature,
                        completed,
                        total,
                        o.dest_path.display()
                    );
                }
                ArchiveCriterion::StatusAttested {
                    status,
                    completed,
                    total,
                } => {
                    println!(
                        "  - {} ({}/{} tasks, STATUS: '{}') -> {}",
                        o.feature,
                        completed,
                        total,
                        status,
                        o.dest_path.display()
                    );
                }
            }
        }

        if args.promote {
            for o in &outcomes {
                match crate::commands::spec::promote_delta_to_domain_spec(
                    &repo_root,
                    &o.feature,
                    args.domain.as_deref(),
                    true,
                ) {
                    Ok(promo) => {
                        println!(
                            "  [dry-run] would promote {} requirement(s) from '{}' into domain spec '{}'",
                            promo.requirements_promoted, promo.change, promo.domain
                        );
                    }
                    Err(err) => {
                        eprintln!(
                            "warning: [dry-run] failed to preview promotion for '{}': {err}",
                            o.feature
                        );
                    }
                }
            }
        }
        println!("dry-run: 0 filesystem mutations applied");
        return Ok(());
    }

    sync_archive_readme_ledger(&repo_root, &outcomes, false)?;

    let archived_names: Vec<String> = outcomes.iter().map(|o| o.feature.clone()).collect();
    reconcile_state_active_feature(ctx, &archived_names, false)?;

    if args.promote {
        for o in &outcomes {
            match crate::commands::spec::promote_delta_to_domain_spec(
                &repo_root,
                &o.feature,
                args.domain.as_deref(),
                false,
            ) {
                Ok(promo) => {
                    println!(
                        "promoted: {} requirement(s) from '{}' into openspec/specs/{}.md",
                        promo.requirements_promoted, o.feature, promo.domain
                    );
                }
                Err(err) => {
                    eprintln!(
                        "warning: failed to promote requirements for '{}': {err}",
                        o.feature
                    );
                }
            }
        }
    }

    if outcomes.len() == 1 {
        let o = &outcomes[0];
        match &o.criterion {
            ArchiveCriterion::Mechanical { completed, total } => {
                println!(
                    "archived: '{}' -> {} ({}/{} tasks complete)",
                    o.feature,
                    o.dest_path.display(),
                    completed,
                    total
                );
            }
            ArchiveCriterion::StatusAttested {
                status,
                completed,
                total,
            } => {
                println!(
                    "archived (Criterion 2 STATUS-attested: '{}'): '{}' -> {} ({}/{} tasks)",
                    status,
                    o.feature,
                    o.dest_path.display(),
                    completed,
                    total
                );
            }
        }
    } else {
        println!(
            "archived {} completed OpenSpec change(s) successfully to openspec/changes/archive/",
            outcomes.len()
        );
    }

    Ok(())
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

/// Runs a git command against `repo_root`, stripping outer hook env vars
/// (`GIT_DIR` etc.) so temporary-repo fixtures behave under the pre-commit hook.
pub(crate) fn git_probe(repo_root: &Path, args: &[&str]) -> Option<std::process::Output> {
    let mut cmd = std::process::Command::new("git");
    for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        cmd.env_remove(var);
    }
    cmd.args(args).current_dir(repo_root).output().ok()
}

/// Resolves the base ref for branch-relative git diffs, trying common defaults.
pub fn resolve_branch_base(repo_root: &Path) -> Option<String> {
    for base_ref in [
        "origin/main",
        "main",
        "origin/master",
        "master",
        "@{upstream}",
    ] {
        if let Some(out) = git_probe(repo_root, &["merge-base", "HEAD", base_ref]) {
            if out.status.success() {
                let base = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !base.is_empty() {
                    return Some(base);
                }
            }
        }
    }
    None
}

pub fn probe_branch_committed_files(repo_root: &Path) -> Vec<String> {
    let diff_target = match resolve_branch_base(repo_root) {
        Some(base) => format!("{base}...HEAD"),
        None => "HEAD~1...HEAD".to_string(),
    };

    let out = match git_probe(repo_root, &["diff", "--name-only", &diff_target]) {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Vec::new(),
    };

    out.lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Number of commits on HEAD that are not on the resolved base branch.
pub fn probe_commits_ahead(repo_root: &Path) -> u32 {
    let Some(base) = resolve_branch_base(repo_root) else {
        return 0;
    };
    let range = format!("{base}..HEAD");
    match git_probe(repo_root, &["rev-list", "--count", &range]) {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .trim()
            .parse::<u32>()
            .unwrap_or(0),
        _ => 0,
    }
}

/// Whether a Stage 6 learning (`docs/solutions/**.md`) is present on the branch.
/// Counts files **added/modified** in the committed range (not deletions) and
/// dirty/untracked files that still exist on disk.
pub fn probe_stage6_artifact(repo_root: &Path, dirty_files: &[String]) -> bool {
    let is_solutions_md = |f: &String| f.starts_with("docs/solutions/") && f.ends_with(".md");
    let committed = probe_branch_added_files(repo_root);
    committed.iter().any(is_solutions_md)
        || dirty_files
            .iter()
            .any(|f| is_solutions_md(f) && repo_root.join(f).exists())
}

/// Files added or modified on the branch range (`--diff-filter=ACMR`), excluding deletions/renames-out.
pub fn probe_branch_added_files(repo_root: &Path) -> Vec<String> {
    let diff_target = match resolve_branch_base(repo_root) {
        Some(base) => format!("{base}...HEAD"),
        None => "HEAD~1...HEAD".to_string(),
    };
    let out = match git_probe(
        repo_root,
        &["diff", "--name-only", "--diff-filter=ACMR", &diff_target],
    ) {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
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
) -> Option<(
    WorkflowStage,
    String,
    Option<String>,
    Option<FeatureResolution>,
)> {
    if is_transitory_git_state(repo_root) {
        return None;
    }

    let openspec_dir = repo_root.join("openspec").join("changes");

    // 1. Resolve feature candidate from branch or probe
    let candidate = branch.map(sanitize_feature_name);
    let (resolved_feature, resolution) =
        if let Some(feat) = candidate.filter(|f| openspec_dir.join(f).is_dir()) {
            (Some(feat), Some(FeatureResolution::Branch))
        } else if let Some(info) = probe_openspec_context_in(repo_root, &None) {
            (Some(info.feature), info.resolution)
        } else {
            (None, None)
        };

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
                    resolution,
                ));
            } else if completed_tasks > 0 && completed_tasks < total_tasks {
                return Some((
                    WorkflowStage::WorkTdd,
                    format!(
                        "Implementing tasks ({completed_tasks}/{total_tasks} completed) for {feat}"
                    ),
                    Some(feat.clone()),
                    resolution,
                ));
            } else if total_tasks > 0 && completed_tasks == total_tasks {
                // All tasks completed: check Stage 7 (Ship), Stage 6 (Compound), or Stage 5 (Verify)
                if check_gh_pr_shipping(repo_root) {
                    return Some((
                        WorkflowStage::GitShipping,
                        format!("Shipping changes / pull request for {feat}"),
                        Some(feat.clone()),
                        resolution,
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
                        resolution,
                    ));
                }

                return Some((
                    WorkflowStage::Verification,
                    format!("Verifying test gates for {feat}"),
                    Some(feat.clone()),
                    resolution,
                ));
            }
        }

        if has_proposal && has_spec {
            return Some((
                WorkflowStage::OpenSpec,
                format!("Authoring OpenSpec contract for {feat}"),
                Some(feat.clone()),
                resolution,
            ));
        }
    }

    // 3. Ideation (Stage 1):
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
            None,
        ));
    }

    // 4. Direct Entry Bypass for Stage 4 (Work/TDD):
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
                    Some(FeatureResolution::Branch),
                ));
            }
        }
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
    let (inferred_stage, inferred_task, inferred_feature, inferred_resolution) =
        match infer_stage_from_repo(repo_root, branch.as_deref()) {
            Some(inf) => inf,
            None => return Ok(None),
        };

    let current_wf = state.current_workflow_for_branch(repo_root, branch.as_deref());
    let current_stage = current_wf
        .as_ref()
        .map(|wf| wf.stage)
        .unwrap_or(WorkflowStage::Ideation);

    let is_new_cycle = match (&current_wf, &inferred_feature) {
        (Some(wf), feat) => {
            inferred_stage.number() == 1 && wf.feature_name.as_deref() != feat.as_deref()
        }
        _ => false,
    };

    // Monotonic provenance guard: Inferred checkpoints can NEVER regress or clobber a Manual checkpoint at equal or higher stage,
    // UNLESS a new cycle is detected (inferred_feature != current_wf.feature_name && inferred_stage.number() == 1)
    if let Some(ref wf) = current_wf {
        if !is_new_cycle {
            if wf.source == WorkflowSource::Manual
                && inferred_stage.number() <= current_stage.number()
            {
                return Ok(None);
            }
            if inferred_stage.number() < current_stage.number() {
                return Ok(None);
            }
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

    let task_to_record = if is_new_cycle {
        format!("{inferred_task} (nuevo ciclo detectado)")
    } else {
        inferred_task
    };

    let updated = State::atomic_update_workflow(state_path, repo_root, branch.as_deref(), |s| {
        s.validate_and_set_workflow_for_branch_with_resolution(
            repo_root,
            branch.as_deref(),
            inferred_stage,
            &task_to_record,
            inferred_feature,
            WorkflowSource::Inferred,
            inferred_resolution,
        )?;
        if is_new_cycle {
            let key = State::workspace_branch_key(repo_root, branch.as_deref());
            if let Some(wf) = s.workflows.get_mut(&key) {
                wf.new_cycle = true;
            }
            if let Some(ref mut wf) = s.workflow {
                wf.new_cycle = true;
            }
        }
        Ok(s.current_workflow_for_branch(repo_root, branch.as_deref()))
    })?;
    Ok(Some(updated))
}

#[cfg(test)]
#[path = "tests/workflow.rs"]
mod tests;
