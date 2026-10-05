//! Observable workflow capabilities state and observation engine.
//!
//! Replaces rigid stage cursor tracking in `state.json` with an on-demand,
//! artifact-derived view of repository reality.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::compat::CeDocsConfig;
use crate::error::CeError;
use crate::state::state::State;

/// Real-time, artifact-derived state of repository workflow capabilities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ObservableWorkflowState {
    /// Whether work is actively underway (uncommitted changes or active feature branch).
    pub active_work: bool,
    /// Currently resolved git branch name, if inside a git repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_branch: Option<String>,
    /// Observed planning artifact in `docs_root/plans/`, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<PlanObservation>,
    /// Empirical verification and testing readiness markers.
    pub verification: VerificationObservation,
    /// Detected handoff documentation or receipts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handoff: Option<HandoffObservation>,
    /// Knowledge capture status and solution documentation detection.
    pub knowledge_capture: KnowledgeObservation,
    /// Optional OpenSpec change package, if explicitly authored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openspec: Option<OpenSpecObservation>,
}

/// Observed details of a markdown execution or requirements plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanObservation {
    /// Relative or absolute path to the active plan file.
    pub path: PathBuf,
    /// Extracted plan title from `# <title>` heading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Count of checked checklist items (`- [x]`).
    pub completed_items: usize,
    /// Total count of checklist items (`- [ ]` + `- [x]`).
    pub total_items: usize,
    /// Whether the plan defines requirements without executable task checkboxes.
    pub is_requirements_only: bool,
}

/// Observed verification readiness and review receipt status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VerificationObservation {
    /// Whether empirical verification evidence exists (review receipt or test artifacts).
    pub has_evidence: bool,
    /// Number of uncommitted files detected in the working tree.
    pub uncommitted_changes: usize,
    /// Whether a code-review receipt is recorded for the active branch head.
    pub review_receipt_stamped: bool,
}

/// Observed handoff artifact for inter-agent or session transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandoffObservation {
    /// Indicates that handoff documentation or receipt was located.
    pub exists: bool,
    /// Path to the handoff file.
    pub path: PathBuf,
    /// RFC3339 timestamp of the last handoff modification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

/// Observed knowledge capture necessity and solution document presence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct KnowledgeObservation {
    /// Whether recent modifications touch core code (`src/**`) warranting knowledge capture.
    pub required: bool,
    /// Whether a solution document exists in `docs_root/solutions/`.
    pub doc_detected: bool,
    /// Path to the detected solution document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_path: Option<PathBuf>,
}

/// Observed optional OpenSpec change package details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenSpecObservation {
    /// Name of the active OpenSpec change package.
    pub feature: String,
    /// Directory path of the change package.
    pub path: PathBuf,
    /// Whether all tasks in `tasks.md` are checked (`- [x]`).
    pub is_sealed: bool,
}

impl ObservableWorkflowState {
    /// Derives the real-time workflow capabilities state directly from repo artifacts.
    pub fn observe(repo_root: &Path, config_dir: &Path) -> Result<Self, CeError> {
        let branch = crate::commands::workflow::probe_git_branch(repo_root);
        let (is_dirty, modified_files) =
            crate::commands::workflow::probe_git_dirty_files(repo_root);
        let uncommitted_count = modified_files.len();

        let is_feature_branch = branch
            .as_deref()
            .map(|b| b != "main" && b != "master" && !b.is_empty())
            .unwrap_or(false);

        let active_work = (is_dirty && uncommitted_count > 0) || is_feature_branch;

        let plan = observe_plan(repo_root, branch.as_deref());
        let verification =
            observe_verification(repo_root, config_dir, branch.as_deref(), uncommitted_count);
        let handoff = observe_handoff(repo_root);
        let knowledge_capture = observe_knowledge(repo_root, &modified_files);
        let openspec = observe_openspec(repo_root, branch.as_deref());

        Ok(Self {
            active_work,
            active_branch: branch,
            plan,
            verification,
            handoff,
            knowledge_capture,
            openspec,
        })
    }
}

/// Scans `plans_dir` for relevant markdown plans and computes checkbox progress.
pub fn observe_plan(repo_root: &Path, branch: Option<&str>) -> Option<PlanObservation> {
    let docs_config = CeDocsConfig::discover(repo_root);
    let plans_dir = docs_config.plans_dir(repo_root);
    if !plans_dir.is_dir() {
        return None;
    }

    let entries = fs::read_dir(&plans_dir).ok()?;
    let mut plan_files: Vec<(PathBuf, SystemTime)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
            let mtime = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            plan_files.push((path, mtime));
        }
    }

    if plan_files.is_empty() {
        return None;
    }

    // Try matching plan filename to branch slug if branch is provided
    let matched_file = if let Some(b) = branch {
        let clean_branch = b
            .trim_start_matches("feat/")
            .trim_start_matches("fix/")
            .trim_start_matches("chore/")
            .trim_start_matches("refactor/");
        plan_files.iter().find(|(path, _)| {
            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            fname.contains(clean_branch)
        })
    } else {
        None
    };

    let selected_path = match matched_file {
        Some((path, _)) => path.clone(),
        None => {
            // Sort by mtime descending to get the most recent plan
            plan_files.sort_by_key(|b| std::cmp::Reverse(b.1));
            plan_files[0].0.clone()
        }
    };

    let content = fs::read_to_string(&selected_path).ok()?;
    let mut completed_items = 0;
    let mut incomplete_items = 0;
    let mut title = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if title.is_none() && trimmed.starts_with("# ") {
            title = Some(trimmed.trim_start_matches('#').trim().to_string());
        }
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            completed_items += 1;
        } else if trimmed.starts_with("- [ ]") {
            incomplete_items += 1;
        }
    }

    let total_items = completed_items + incomplete_items;
    let lower = content.to_lowercase();
    let is_requirements_only = total_items == 0
        && (lower.contains("requirement")
            || lower.contains("problem statement")
            || lower.contains("user story"));

    Some(PlanObservation {
        path: selected_path,
        title,
        completed_items,
        total_items,
        is_requirements_only,
    })
}

/// Inspects verification status and review receipts.
pub fn observe_verification(
    repo_root: &Path,
    config_dir: &Path,
    branch: Option<&str>,
    uncommitted_changes: usize,
) -> VerificationObservation {
    let state_path = config_dir.join("state.json");
    let review_receipt_stamped = if let Ok(state) = State::load(&state_path) {
        state.review_receipt_for_branch(repo_root, branch).is_some()
    } else {
        false
    } || repo_root.join(".review-receipt.json").exists();

    let has_evidence = review_receipt_stamped
        || repo_root.join("target").join("coverage").exists()
        || repo_root.join(".test-receipt.json").exists();

    VerificationObservation {
        has_evidence,
        uncommitted_changes,
        review_receipt_stamped,
    }
}

/// Inspects handoff artifacts in well-known locations.
pub fn observe_handoff(repo_root: &Path) -> Option<HandoffObservation> {
    let candidates = [
        repo_root.join(".compound-engineering").join("handoff.md"),
        repo_root.join("docs").join("plans").join("handoff.md"),
        repo_root.join(".handoff.json"),
    ];

    for candidate in candidates {
        if candidate.is_file() {
            let mtime_str = candidate
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .map(|t| {
                    let dt: DateTime<Utc> = t.into();
                    dt.to_rfc3339()
                });
            return Some(HandoffObservation {
                exists: true,
                path: candidate,
                last_modified: mtime_str,
            });
        }
    }

    None
}

/// Evaluates knowledge capture necessity against changed files and solution docs.
pub fn observe_knowledge(repo_root: &Path, modified_files: &[String]) -> KnowledgeObservation {
    let has_core_changes = modified_files.iter().any(|f| {
        let p = f.replace('\\', "/");
        p.starts_with("src/") || p.starts_with("lib/")
    });

    let docs_config = CeDocsConfig::discover(repo_root);
    let solutions_dir = docs_config.solutions_dir(repo_root);

    let mut doc_detected = false;
    let mut doc_path = None;

    if solutions_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&solutions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("md") {
                    doc_detected = true;
                    doc_path = Some(path);
                    break;
                }
                if path.is_dir() {
                    // Check subdirectories (e.g. docs/solutions/architecture/)
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub in sub_entries.flatten() {
                            let sub_path = sub.path();
                            if sub_path.is_file()
                                && sub_path.extension().and_then(|e| e.to_str()) == Some("md")
                            {
                                doc_detected = true;
                                doc_path = Some(sub_path);
                                break;
                            }
                        }
                    }
                    if doc_detected {
                        break;
                    }
                }
            }
        }
    }

    KnowledgeObservation {
        required: has_core_changes,
        doc_detected,
        doc_path,
    }
}

/// Inspects `openspec/changes/` for optional active change packages.
pub fn observe_openspec(repo_root: &Path, branch: Option<&str>) -> Option<OpenSpecObservation> {
    let changes_dir = repo_root.join("openspec").join("changes");
    if !changes_dir.is_dir() {
        return None;
    }

    let entries = fs::read_dir(&changes_dir).ok()?;
    let mut change_dirs: Vec<(String, PathBuf, SystemTime)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) if n != "archive" && !n.starts_with('.') => n.to_string(),
                _ => continue,
            };
            let mtime = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            change_dirs.push((name, path, mtime));
        }
    }

    if change_dirs.is_empty() {
        return None;
    }

    // Try matching branch name to feature directory
    let selected = if let Some(b) = branch {
        let clean_branch = b
            .trim_start_matches("feat/")
            .trim_start_matches("fix/")
            .trim_start_matches("chore/")
            .trim_start_matches("refactor/");
        change_dirs
            .iter()
            .find(|(name, _, _)| name == clean_branch || clean_branch.contains(name))
            .cloned()
    } else {
        None
    };

    let (feature, path, _) = match selected {
        Some(s) => s,
        None => {
            change_dirs.sort_by_key(|b| std::cmp::Reverse(b.2));
            change_dirs[0].clone()
        }
    };

    // Check if tasks.md is fully completed
    let tasks_path = path.join("tasks.md");
    let is_sealed = if let Ok(tasks_content) = fs::read_to_string(&tasks_path) {
        let mut completed = 0;
        let mut incomplete = 0;
        for line in tasks_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
                completed += 1;
            } else if trimmed.starts_with("- [ ]") {
                incomplete += 1;
            }
        }
        completed > 0 && incomplete == 0
    } else {
        false
    };

    Some(OpenSpecObservation {
        feature,
        path,
        is_sealed,
    })
}
