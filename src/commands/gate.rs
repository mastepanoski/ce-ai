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
use crate::state::state::{FeatureResolution, State, WorkflowStage};

/// Well-known structured log location relative to ce-ai config dir.
pub fn gate_events_log_path(config_dir: &Path) -> PathBuf {
    config_dir.join("gate-events.jsonl")
}

/// The primary decision outcome of a gate check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateDecision {
    /// Stage 4 (`ce-work`) write with missing OpenSpec contract artifacts.
    WouldBlock,
    /// Authorized write (complete OpenSpec contract, non-Stage 4, or non-target path).
    Pass,
    /// Ambiguous, corrupt, or unadopted checkpoint state.
    Undetermined,
    /// Environmental or lifecycle anomaly isolated from the happy path.
    EdgeCase,
}

impl GateDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            GateDecision::WouldBlock => "would_block",
            GateDecision::Pass => "pass",
            GateDecision::Undetermined => "undetermined",
            GateDecision::EdgeCase => "edge_case",
        }
    }
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

    /// Emergency kill-switch to immediately bypass gate check logic
    #[arg(long)]
    pub disabled: bool,
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

/// Pure decision engine for the gate check spike.
///
/// Evaluates declared stage, active feature name, resolution provenance,
/// edge case signals, and presence of OpenSpec contract artifacts.
/// Returns the decision enum, optional edge case category, and explanatory reason.
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
    // 1. Undetermined if checkpoint is absent or ambiguous
    let stage = match declared_stage {
        Some(s) => s,
        None => {
            return (
                GateDecision::Undetermined,
                None,
                "no active workflow checkpoint found".to_string(),
            );
        }
    };

    // 2. Edge case isolation (never mixed with would-block or pass)
    if resolution == Some(FeatureResolution::MtimeFallback) {
        return (
            GateDecision::EdgeCase,
            Some(GateEdgeCase::MtimeFallback),
            "workflow feature resolved via mtime fallback without git branch".to_string(),
        );
    }
    if is_new_cycle_task {
        return (
            GateDecision::EdgeCase,
            Some(GateEdgeCase::StaleCycleGuard),
            "same-branch multi-cycle reset detected in active task".to_string(),
        );
    }
    if has_uncommitted_spec {
        return (
            GateDecision::EdgeCase,
            Some(GateEdgeCase::WorktreeUncommitted),
            "openspec directory has uncommitted files in current worktree".to_string(),
        );
    }

    // 3. Stage 4 (ce-work) contract evaluation
    if stage == WorkflowStage::WorkTdd {
        let feat = feature_name.unwrap_or("unknown");
        if !has_proposal || !has_spec || !has_tasks {
            let mut missing = Vec::new();
            if !has_proposal {
                missing.push("proposal.md");
            }
            if !has_spec {
                missing.push("spec.md");
            }
            if !has_tasks {
                missing.push("tasks.md");
            }
            return (
                GateDecision::WouldBlock,
                None,
                format!(
                    "Stage 4 (ce-work) active for '{feat}' without approved OpenSpec contract (missing: {})",
                    missing.join(", ")
                ),
            );
        }
        return (
            GateDecision::Pass,
            None,
            format!("Stage 4 (ce-work) active with complete OpenSpec contract for '{feat}'"),
        );
    }

    // Non-Stage 4 writes pass
    (
        GateDecision::Pass,
        None,
        format!(
            "Stage {} ({}) permits writes without Stage 4 OpenSpec contract",
            stage.number(),
            stage.as_str()
        ),
    )
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
    let (declared_stage, feature_name, resolution, is_new_cycle_task) = if state_path.exists() {
        if let Ok(state) = State::load(&state_path) {
            if let Some(wf) = state.current_workflow_for_branch(&repo_root, branch.as_deref()) {
                let is_new_cycle = wf.new_cycle;
                (Some(wf.stage), wf.feature_name, wf.resolution, is_new_cycle)
            } else {
                (None, None, None, false)
            }
        } else {
            (None, None, None, false)
        }
    } else {
        (None, None, None, false)
    };

    // 6. Check edge cases & artifact presence
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

    // 7. Pure decision evaluation
    let (decision, edge_case, reason) = evaluate_gate_decision(
        declared_stage,
        feature_name.as_deref(),
        resolution,
        is_new_cycle_task,
        has_uncommitted_spec,
        has_proposal,
        has_spec,
        has_tasks,
    );

    // 8. Append structured telemetry record (best effort, errors safely swallowed)
    let record = GateEventRecord {
        timestamp: chrono::Utc::now().to_rfc3339(),
        harness: "claude".to_string(),
        tool,
        path: path_str,
        workspace: repo_root.display().to_string(),
        branch,
        stage: declared_stage.map(|s| s.number()),
        feature: feature_name,
        decision,
        edge_case,
        reason,
    };

    let _ = log_gate_event(&ctx.config_dir, &record);

    // 9. Invariant: Spike NEVER blocks writes
    Ok(())
}

#[cfg(test)]
#[path = "tests/gate.rs"]
mod tests;
