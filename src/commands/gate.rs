//! Observe-only gate check for agent tool write monitoring (Spike #333).
//!
//! Evaluates whether active Stage 4 (`ce-work`) tool invocations have approved
//! OpenSpec contracts (`proposal.md`, `spec.md`, `tasks.md`), isolating edge cases
//! (`mtime_fallback`, `worktree_uncommitted`, `stale_cycle_guard`), and recording
//! structured telemetry without ever interrupting or blocking execution.

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

use crate::commands::Context;
use crate::error::CeError;
use crate::state::state::{AdoptionTier, ExecutionMode, FeatureResolution, State, WorkflowStage};
pub use crate::state::state::{GateDecision, GateMode, GateReceipt};

/// Well-known structured log location relative to ce-ai config dir.
pub fn gate_events_log_path(config_dir: &Path) -> PathBuf {
    config_dir.join("gate-events.jsonl")
}

/// Discretely isolated edge-case categories (Issue #337).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateEdgeCase {
    /// Feature inferred via filesystem mtime fallback rather than branch.
    MtimeFallback,
    /// OpenSpec directory contains uncommitted changes in current worktree.
    WorktreeUncommitted,
    /// Multi-cycle reset on same branch detected in workflow task.
    StaleCycleGuard,
}

impl GateEdgeCase {
    pub fn as_str(&self) -> &'static str {
        match self {
            GateEdgeCase::MtimeFallback => "mtime_fallback",
            GateEdgeCase::WorktreeUncommitted => "worktree_uncommitted",
            GateEdgeCase::StaleCycleGuard => "stale_cycle_guard",
        }
    }
}

/// Structured record appended to `gate-events.jsonl` for each observed write.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GateEventRecord {
    pub timestamp: String,
    pub harness: String,
    pub tool: String,
    pub path: String,
    pub workspace: String,
    pub branch: Option<String>,
    pub stage: Option<u32>,
    pub feature: Option<String>,
    pub decision: GateDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edge_case: Option<GateEdgeCase>,
    pub reason: String,
}

/// Aggregated metrics parsed from `gate-events.jsonl`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GateStats {
    pub total_observed: usize,
    pub blocked: usize,
    pub would_block: usize,
    pub pass: usize,
    pub undetermined: usize,
    pub mtime_fallback: usize,
    pub worktree_uncommitted: usize,
    pub stale_cycle_guard: usize,
}

/// Subcommands for the `gate` command group.
#[derive(Subcommand, Debug, Clone)]
pub enum GateCommands {
    /// Observe and record gate telemetry for agent tool writes (Spike #333)
    Check(GateCheckArgs),
}

/// Command-line arguments for `ce-ai gate check`.
#[derive(Args, Debug, Default, Clone)]
pub struct GateCheckArgs {
    /// Tool name being invoked (e.g. Write, Edit)
    #[arg(long)]
    pub tool: Option<String>,

    /// Target file path of the write/edit operation
    #[arg(long)]
    pub path: Option<String>,

    /// Gate evaluation mode: enforce (blocking) or observe (advisory)
    #[arg(long)]
    pub mode: Option<String>,

    /// Optional explicit entry point (e.g. ce-debug, ce-work)
    #[arg(long)]
    pub entry_point: Option<String>,

    /// Optional execution mode override (auto, organic, compound)
    #[arg(long = "execution-mode")]
    pub execution_mode: Option<String>,

    /// Emergency kill-switch to immediately bypass gate check logic
    #[arg(long)]
    pub disabled: bool,
}

/// Resolves the active gate mode based on flag, environment variable, state, and default.
pub fn resolve_gate_mode(flag_mode: Option<&str>, state_mode: Option<GateMode>) -> GateMode {
    if let Some(m_str) = flag_mode {
        if let Some(m) = GateMode::parse(m_str) {
            return m;
        }
    }
    if let Ok(env_val) = std::env::var("CE_AI_GATE_MODE") {
        if let Some(m) = GateMode::parse(&env_val) {
            return m;
        }
    }
    state_mode.unwrap_or(GateMode::Enforce)
}

/// Evaluates whether the gate check kill-switch is active via flag or environment variables.
pub fn is_gate_kill_switched(disabled_flag: bool) -> bool {
    if disabled_flag {
        return true;
    }
    for var in ["CE_AI_DISABLE_GATE_CHECK", "CE_AI_GATE_CHECK_DISABLED"] {
        if let Ok(val) = std::env::var(var) {
            let v = val.trim().to_lowercase();
            if v == "1" || v == "true" || v == "yes" {
                return true;
            }
        }
    }
    false
}

/// Parses Claude Code `PreToolUse` JSON payload from stdin.
pub fn parse_tool_call_json(raw: &str) -> Option<(String, String)> {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(raw.trim()) else {
        return None;
    };
    let tool_name = val
        .get("tool_name")
        .or_else(|| val.get("tool"))
        .and_then(|v| v.as_str())?;

    let path = val
        .get("tool_input")
        .or_else(|| val.get("input"))
        .and_then(|input| {
            input
                .get("path")
                .or_else(|| input.get("file_path"))
                .or_else(|| input.get("target_file"))
                .and_then(|p| p.as_str())
        })
        .unwrap_or("");

    Some((tool_name.to_string(), path.to_string()))
}

/// Evaluates whether a tool call targets a code write under `src/**`.
pub fn is_target_write_operation(tool: &str, path: &str) -> bool {
    let t = tool.trim().to_lowercase();
    if t != "write" && t != "edit" {
        return false;
    }
    let p = path.trim().replace('\\', "/");
    p.starts_with("src/") || p.contains("/src/")
}

/// Appends a structured gate event record to `gate-events.jsonl`.
pub fn log_gate_event(config_dir: &Path, record: &GateEventRecord) -> Result<(), CeError> {
    let log_path = gate_events_log_path(config_dir);
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    let serialized = serde_json::to_string(record)
        .map_err(|e| CeError::Runtime(format!("failed to serialize gate record: {e}")))?;
    writeln!(file, "{serialized}")?;
    Ok(())
}

/// Formats a detailed, actionable remediation message for blocked tool writes.
pub fn format_blocked_remediation_message(
    path: &str,
    stage: &str,
    feature: &str,
    missing: &[String],
) -> String {
    let mut out = format!(
        "ce-ai gate check: Write blocked on '{path}'\n\nActive Stage: {stage} for feature '{feature}'\nMissing OpenSpec Contract Artifacts:\n"
    );
    for item in missing {
        let desc = match item.as_str() {
            "proposal.md" => "Missing problem statement, boundaries & risk evaluation",
            "spec.md" => "Missing formal WHEN/THEN requirements & acceptance criteria",
            "tasks.md" => "Missing implementation checklist with ~200 LOC work units",
            _ => "Missing required artifact",
        };
        out.push_str(&format!("  ✖ {item} — {desc}\n"));
    }
    out.push_str(&format!(
        "\nRemediation Steps:\n  1. Author the OpenSpec contract: Run `/ce-plan` or create missing files in `openspec/changes/{feature}/`.\n  2. If performing an emergency bugfix: Switch to direct entry via:\n     ce-ai workflow checkpoint --stage 4 --task \"ce-debug: <issue>\"\n  3. Emergency override: Set CE_AI_DISABLE_GATE_CHECK=1 in your environment.\n"
    ));
    out
}

/// Atomically persists a structured gate validation receipt into the feature's
/// `.validation.json` and updates `state.gate_receipts` (Issue #334).
pub fn write_gate_receipt(
    ctx: &Context,
    repo_root: &Path,
    receipt: &GateReceipt,
) -> Result<(), CeError> {
    // 1. Write .validation.json into feature change folder if it exists
    let feature = &receipt.feature;
    if !feature.is_empty() && feature != "unknown" {
        let change_dir = repo_root.join("openspec").join("changes").join(feature);
        if change_dir.is_dir() {
            let validation_path = change_dir.join(".validation.json");
            let json_bytes = serde_json::to_vec_pretty(receipt)
                .map_err(|e| CeError::Runtime(format!("failed to serialize gate receipt: {e}")))?;
            let _ = crate::state::write_atomic(&validation_path, &json_bytes);
        }
    }

    // 2. Persist into state.json under gate_receipts
    let state_path = ctx.config_dir.join("state.json");
    if state_path.exists() {
        if let Ok(mut state) = State::load(&state_path) {
            state
                .gate_receipts
                .insert(receipt.feature.clone(), receipt.clone());
            let _ = state.save(&state_path);
        }
    }

    Ok(())
}

/// Loads and aggregates gate metrics from `gate-events.jsonl`.
pub fn load_gate_stats(config_dir: &Path) -> Result<GateStats, CeError> {
    let log_path = gate_events_log_path(config_dir);
    if !log_path.exists() {
        return Ok(GateStats::default());
    }

    let file = File::open(&log_path)?;
    let reader = BufReader::new(file);
    let mut stats = GateStats::default();

    for line_res in reader.lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let record: GateEventRecord = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(_) => continue,
        };

        stats.total_observed += 1;
        match record.decision {
            GateDecision::Blocked => stats.blocked += 1,
            GateDecision::WouldBlock => stats.would_block += 1,
            GateDecision::Pass => stats.pass += 1,
            GateDecision::Undetermined => stats.undetermined += 1,
            GateDecision::EdgeCase => match record.edge_case {
                Some(GateEdgeCase::MtimeFallback) => stats.mtime_fallback += 1,
                Some(GateEdgeCase::WorktreeUncommitted) => stats.worktree_uncommitted += 1,
                Some(GateEdgeCase::StaleCycleGuard) => stats.stale_cycle_guard += 1,
                None => {}
            },
        }
    }

    Ok(stats)
}

/// Comprehensive policy engine for the gate check (Issue #334).
///
/// Evaluates declared stage, active feature name, resolution provenance,
/// edge case signals, adoption tier, entry point (e.g. ce-debug vs ce-work),
/// presence of OpenSpec contract artifacts, and enforcement mode.
/// Returns (decision, optional edge case, list of missing artifacts, explanatory reason).
#[allow(clippy::too_many_arguments)]
pub fn evaluate_gate_policy(
    declared_stage: Option<WorkflowStage>,
    feature_name: Option<&str>,
    resolution: Option<FeatureResolution>,
    is_new_cycle_task: bool,
    has_uncommitted_spec: bool,
    tier: AdoptionTier,
    entry_point: Option<&str>,
    has_proposal: bool,
    has_spec: bool,
    has_tasks: bool,
    mode: GateMode,
    execution_mode: ExecutionMode,
) -> (GateDecision, Option<GateEdgeCase>, Vec<String>, String) {
    // 1. Edge case isolation (never mixed with blocked or pass, never blocks)
    if resolution == Some(FeatureResolution::MtimeFallback) {
        return (
            GateDecision::EdgeCase,
            Some(GateEdgeCase::MtimeFallback),
            Vec::new(),
            "workflow feature resolved via mtime fallback without git branch".to_string(),
        );
    }
    if is_new_cycle_task {
        return (
            GateDecision::EdgeCase,
            Some(GateEdgeCase::StaleCycleGuard),
            Vec::new(),
            "same-branch multi-cycle reset detected in active task".to_string(),
        );
    }
    if has_uncommitted_spec {
        return (
            GateDecision::EdgeCase,
            Some(GateEdgeCase::WorktreeUncommitted),
            Vec::new(),
            "openspec directory has uncommitted files in current worktree".to_string(),
        );
    }

    // 2. Organic mode exemption (permits writes without formal OpenSpec contract)
    if execution_mode == ExecutionMode::Organic {
        return (
            GateDecision::Pass,
            None,
            Vec::new(),
            "organic execution mode permits writes without formal OpenSpec contract".to_string(),
        );
    }

    // 3. Undetermined if checkpoint is absent or ambiguous
    let stage = match declared_stage {
        Some(s) => s,
        None => {
            return (
                GateDecision::Undetermined,
                None,
                Vec::new(),
                "no active workflow checkpoint found".to_string(),
            );
        }
    };

    // 4. AdoptionTier::Minimal exemption
    if tier == AdoptionTier::Minimal {
        return (
            GateDecision::Pass,
            None,
            Vec::new(),
            "project tier minimal permits writes without full OpenSpec contract".to_string(),
        );
    }

    // 5. Direct Entry Point: ce-debug exemption (for bugfixes)
    if let Some(entry) = entry_point {
        let clean = entry.trim().to_lowercase();
        if clean.starts_with("ce-debug")
            || clean.contains("ce-debug")
            || clean.starts_with("debug")
            || clean.contains("debug:")
        {
            return (
                GateDecision::Pass,
                None,
                Vec::new(),
                "ce-debug direct entry point permits bug fix writes without formal OpenSpec contract".to_string(),
            );
        }
    }

    // 6. Non-Stage 4 writes pass
    if stage != WorkflowStage::WorkTdd {
        return (
            GateDecision::Pass,
            None,
            Vec::new(),
            format!(
                "Stage {} ({}) permits writes without Stage 4 OpenSpec contract",
                stage.number(),
                stage.as_str()
            ),
        );
    }

    // 7. Stage 4 (ce-work) contract evaluation
    let feat = feature_name.unwrap_or("unknown");
    let mut missing = Vec::new();
    if !has_proposal {
        missing.push("proposal.md".to_string());
    }
    if !has_spec {
        missing.push("spec.md".to_string());
    }
    if !has_tasks {
        missing.push("tasks.md".to_string());
    }

    if !missing.is_empty() {
        let decision = match mode {
            GateMode::Enforce => GateDecision::Blocked,
            GateMode::Observe => GateDecision::WouldBlock,
        };
        return (
            decision,
            None,
            missing.clone(),
            format!(
                "Stage 4 (ce-work) active for '{feat}' without approved OpenSpec contract (missing: {})",
                missing.join(", ")
            ),
        );
    }

    (
        GateDecision::Pass,
        None,
        Vec::new(),
        format!("Stage 4 (ce-work) active with complete OpenSpec contract for '{feat}'"),
    )
}

/// Pure decision engine for the legacy gate check spike (Issue #333 compatibility).
#[allow(clippy::too_many_arguments)]
pub fn evaluate_gate_decision(
    declared_stage: Option<WorkflowStage>,
    feature_name: Option<&str>,
    resolution: Option<FeatureResolution>,
    is_new_cycle_task: bool,
    has_uncommitted_spec: bool,
    has_proposal: bool,
    has_spec: bool,
    has_tasks: bool,
) -> (GateDecision, Option<GateEdgeCase>, String) {
    let (decision, edge, _missing, reason) = evaluate_gate_policy(
        declared_stage,
        feature_name,
        resolution,
        is_new_cycle_task,
        has_uncommitted_spec,
        AdoptionTier::Full,
        None,
        has_proposal,
        has_spec,
        has_tasks,
        GateMode::Observe,
        ExecutionMode::Compound,
    );
    (decision, edge, reason)
}

/// Main execution routine for `ce-ai gate check`.
///
/// Never blocks execution; errors during logging are safely ignored and exit code is always 0.
pub fn run_gate_check(ctx: &Context, args: &GateCheckArgs) -> Result<(), CeError> {
    // 1. Emergency kill-switch check: immediate no-op
    if is_gate_kill_switched(args.disabled) {
        return Ok(());
    }

    // 2. Ingest tool name and path from flags or stdin
    let (tool, path_str) = match (&args.tool, &args.path) {
        (Some(t), Some(p)) => (t.clone(), p.clone()),
        _ => {
            let mut stdin_buf = String::new();
            if std::io::stdin().read_to_string(&mut stdin_buf).is_ok()
                && !stdin_buf.trim().is_empty()
            {
                if let Some((t, p)) = parse_tool_call_json(&stdin_buf) {
                    (t, p)
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        }
    };

    // 3. Filter target writes under src/**
    if !is_target_write_operation(&tool, &path_str) {
        return Ok(());
    }

    // 4. Resolve workspace and git branch
    let repo_root = ctx.repo_root();
    let branch = crate::commands::workflow::probe_git_branch(&repo_root);

    // 5. Read declared checkpoint from state.json
    let state_path = ctx.config_dir.join("state.json");
    let state_opt = if state_path.exists() {
        State::load(&state_path).ok()
    } else {
        None
    };

    let wf_opt = state_opt
        .as_ref()
        .and_then(|s| s.current_workflow_for_branch(&repo_root, branch.as_deref()));

    let (declared_stage, feature_name, resolution, is_new_cycle_task, task_desc) =
        if let Some(ref wf) = wf_opt {
            (
                Some(wf.stage),
                wf.feature_name.clone(),
                wf.resolution,
                wf.new_cycle,
                Some(wf.task.clone()),
            )
        } else {
            (None, None, None, false, None)
        };

    // 6. Resolve adoption tier & gate mode
    let tier = state_opt
        .as_ref()
        .and_then(|s| s.project_for_path(&repo_root))
        .map(|p| p.tier)
        .unwrap_or(AdoptionTier::Full);

    let state_gate_mode = state_opt.as_ref().and_then(|s| s.gate_mode);
    let gate_mode = resolve_gate_mode(args.mode.as_deref(), state_gate_mode);

    // Turn-0 mode resolution:
    let cli_execution_mode = args
        .execution_mode
        .as_deref()
        .and_then(|m| ExecutionMode::parse(m).ok());
    let execution_mode = crate::commands::workflow::probe_execution_mode(
        &repo_root,
        branch.as_deref(),
        &wf_opt,
        tier,
        cli_execution_mode,
    );

    // 7. Resolve entry point (CLI argument > task description)
    let entry_point = args.entry_point.as_deref().or(task_desc.as_deref());

    // 8. Check edge cases & artifact presence
    let has_uncommitted_spec = feature_name
        .as_deref()
        .map(|f| crate::commands::workflow::probe_openspec_has_uncommitted(&repo_root, f))
        .unwrap_or(false);

    let (has_proposal, has_spec, has_tasks) = feature_name
        .as_deref()
        .map(|f| {
            let change_dir = repo_root.join("openspec").join("changes").join(f);
            let has_p = change_dir.join("proposal.md").is_file()
                && std::fs::metadata(change_dir.join("proposal.md"))
                    .map(|m| m.len() > 0)
                    .unwrap_or(false);
            let has_s = change_dir.join("spec.md").is_file()
                && std::fs::metadata(change_dir.join("spec.md"))
                    .map(|m| m.len() > 0)
                    .unwrap_or(false);
            let has_t = change_dir.join("tasks.md").is_file()
                && std::fs::metadata(change_dir.join("tasks.md"))
                    .map(|m| m.len() > 0)
                    .unwrap_or(false);
            (has_p, has_s, has_t)
        })
        .unwrap_or((false, false, false));

    // 9. Pure policy evaluation
    let (decision, edge_case, missing, reason) = evaluate_gate_policy(
        declared_stage,
        feature_name.as_deref(),
        resolution,
        is_new_cycle_task,
        has_uncommitted_spec,
        tier,
        entry_point,
        has_proposal,
        has_spec,
        has_tasks,
        gate_mode,
        execution_mode,
    );

    // 10. Append structured telemetry record (best effort, errors safely swallowed)
    let record = GateEventRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        harness: "claude".to_string(),
        tool,
        path: path_str.clone(),
        workspace: repo_root.display().to_string(),
        branch,
        stage: declared_stage.map(|s| s.number()),
        feature: feature_name.clone(),
        decision,
        edge_case,
        reason: reason.clone(),
    };

    let _ = log_gate_event(&ctx.config_dir, &record);

    // 11. Write structured validation receipt
    let receipt = GateReceipt {
        timestamp: record.timestamp.clone(),
        feature: feature_name.unwrap_or_else(|| "unknown".to_string()),
        target_path: path_str.clone(),
        decision,
        stage: declared_stage.map(|s| s.number()),
        entry_point: entry_point.map(|e| e.to_string()),
        tier: tier.as_str().to_string(),
        missing_artifacts: missing.clone(),
        reason: reason.clone(),
    };
    let _ = write_gate_receipt(ctx, &repo_root, &receipt);

    // 12. Handle blocked decision: emit stderr message and Exit Code 2 (CeError::Usage)
    if decision == GateDecision::Blocked {
        let stage_str = declared_stage
            .map(|s| format!("Stage {} ({})", s.number(), s.as_str()))
            .unwrap_or_else(|| "Stage 4 (work)".to_string());
        let msg =
            format_blocked_remediation_message(&path_str, &stage_str, &receipt.feature, &missing);
        eprintln!("{msg}");
        return Err(CeError::Usage(format!(
            "write blocked on '{path_str}': missing required OpenSpec contract artifacts ({})",
            missing.join(", ")
        )));
    }

    // 13. Observe-only advisory notice for Organic mode exceeding 200 LOC ceiling
    if execution_mode == ExecutionMode::Organic {
        let diff_loc = crate::commands::workflow::probe_git_diff_loc(&repo_root);
        if diff_loc > 200 && !ctx.quiet {
            eprintln!(
                "Notice: Organic task diff (+{diff_loc} LOC) exceeds 200 LOC ceiling. Consider running 'ce-ai workflow graduate' to formalize in OpenSpec."
            );
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests/gate.rs"]
mod tests;
