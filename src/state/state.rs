//! `state.json` I/O with atomic temp-file+rename writes (MM-1).

use crate::error::CeError;
use crate::state::write_atomic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AdoptionTier {
    #[default]
    Full,
    Minimal,
    Orchestrator,
}

impl AdoptionTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdoptionTier::Full => "full",
            AdoptionTier::Minimal => "minimal",
            AdoptionTier::Orchestrator => "orchestrator",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectAdoptionEntry {
    pub path: PathBuf,
    pub file: String,
    pub tier: AdoptionTier,
    pub block_version: u32,
    pub block_sha256: String,
    pub created_file: bool,
    pub adopted_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelAssignment {
    pub provider_id: String,
    pub model_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
}

/// Supply-chain provenance of the cached release tarball (Issue #161):
/// binds the archive digest to the release tag it actually came from so a
/// cached artifact can never be relabelled as a different requested version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseProvenance {
    /// GitHub release tag (e.g. `v1.2.3`); ce-ai only ever records immutable
    /// release tags — never a mutable branch.
    pub tag: String,
    /// Archive download URL that produced the cached artifact.
    pub url: String,
    /// Lowercase hex SHA256 of the cached tarball bytes.
    pub archive_sha256: String,
    /// Extracted source tree root used for the sync
    /// (`<config>/cache/trees/<tag>`; a temp dir under `--dry-run`).
    pub extraction_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStage {
    #[default]
    Ideation = 1,
    OpenSpec = 2,
    ExecutionPlan = 3,
    WorkTdd = 4,
    Verification = 5,
    KnowledgeCapture = 6,
    GitShipping = 7,
}

impl WorkflowStage {
    pub fn number(&self) -> u32 {
        *self as u32
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            WorkflowStage::Ideation => "ideation",
            WorkflowStage::OpenSpec => "openspec",
            WorkflowStage::ExecutionPlan => "plan",
            WorkflowStage::WorkTdd => "work",
            WorkflowStage::Verification => "verify",
            WorkflowStage::KnowledgeCapture => "compound",
            WorkflowStage::GitShipping => "ship",
        }
    }

    pub fn parse(s: &str) -> Result<Self, CeError> {
        let clean = s.trim().to_lowercase();
        match clean.as_str() {
            "1" | "ideation" | "brainstorm" => Ok(WorkflowStage::Ideation),
            "2" | "openspec" | "spec" => Ok(WorkflowStage::OpenSpec),
            "3" | "plan" | "executionplan" => Ok(WorkflowStage::ExecutionPlan),
            "4" | "work" | "tdd" | "worktdd" => Ok(WorkflowStage::WorkTdd),
            "5" | "verify" | "verification" => Ok(WorkflowStage::Verification),
            "6" | "compound" | "knowledgecapture" => Ok(WorkflowStage::KnowledgeCapture),
            "7" | "ship" | "gitshipping" => Ok(WorkflowStage::GitShipping),
            _ => Err(CeError::Usage(format!(
                "invalid workflow stage '{s}'. Valid stages: 1 (ideation), 2 (openspec), 3 (plan), 4 (work), 5 (verify), 6 (compound), 7 (ship)"
            ))),
        }
    }

    pub fn can_transition_to(&self, target: WorkflowStage) -> bool {
        let current_num = self.number();
        let target_num = target.number();
        target_num == 1
            || target_num == current_num
            || target_num == current_num + 1
            || (current_num > 1 && target_num == current_num - 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkflowState {
    pub stage: WorkflowStage,
    pub task: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_name: Option<String>,
    pub updated_at: String,
}

/// One tracked file of an adopted skills surface (path relative to the
/// surface root + expected sha256).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillSurfaceFile {
    pub path: String,
    pub sha256: String,
}

/// Adoption ledger entry: a harness skills root put under ce-ai management
/// (`adopted`), explicitly declined by the user (`declined`), or whose root
/// vanished (`orphaned`). Canonical-skills-adoption R2/R3/R19.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillSurface {
    pub harness: String,
    pub root: PathBuf,
    pub status: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<SkillSurfaceFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adopted_at: Option<String>,
}

/// Intensity level of pedagogical oversight (Issue #114).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GuardLevel {
    #[default]
    Junior,
    Strict,
}

impl GuardLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            GuardLevel::Junior => "junior",
            GuardLevel::Strict => "strict",
        }
    }

    pub fn parse(s: &str) -> Result<Self, CeError> {
        match s.trim().to_lowercase().as_str() {
            "junior" => Ok(GuardLevel::Junior),
            "strict" => Ok(GuardLevel::Strict),
            other => Err(CeError::Usage(format!(
                "invalid guard level '{other}'. Valid levels: junior, strict"
            ))),
        }
    }
}

impl std::fmt::Display for GuardLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Pedagogical guardrail configuration in `state.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardrailState {
    pub enabled: bool,
    pub level: GuardLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<String>,
    pub updated_at: String,
}

/// Canonical state file at `~/.ce-ai/state.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub installed_harnesses: Vec<serde_json::Value>,
    #[serde(default)]
    pub managed_asset_digest: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_provenance: Option<ReleaseProvenance>,
    #[serde(default)]
    pub model_assignments: BTreeMap<String, ModelAssignment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_update_check: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<WorkflowState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_release_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<ProjectAdoptionEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_surfaces: Vec<SkillSurface>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardrail: Option<GuardrailState>,
}
fn default_version() -> u32 {
    1
}

impl State {
    pub fn new() -> Self {
        Self {
            version: 1,
            ..Self::default()
        }
    }

    pub fn load(path: &Path) -> Result<Self, CeError> {
        match std::fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| {
                CeError::State(format!("state.json at {} is corrupt: {e}", path.display()))
            }),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Self::new()),
            Err(err) => Err(CeError::State(format!(
                "cannot read state.json at {}: {err}",
                path.display()
            ))),
        }
    }

    /// Persists state.json atomically (temp file + rename).
    pub fn save(&self, path: &Path) -> Result<(), CeError> {
        write_atomic(path, &serde_json::to_vec(self)?).map_err(|e| {
            CeError::State(format!(
                "cannot persist state.json at {}: {e}",
                path.display()
            ))
        })
    }

    /// Finds a registered project adoption entry matching the given path.
    pub fn project_for_path(&self, target_path: &Path) -> Option<&ProjectAdoptionEntry> {
        self.projects.iter().find(|p| p.path == target_path)
    }

    /// Returns active `WorkflowState`, falling back to legacy `last_update_check` parsing if present.
    pub fn current_workflow(&self) -> Option<WorkflowState> {
        if let Some(wf) = &self.workflow {
            return Some(wf.clone());
        }
        if let Some(entry) = &self.last_update_check {
            let mut parts = entry.splitn(3, " | ");
            let phase = parts.next()?.trim();
            let task = parts.next()?.trim();
            let ts = parts.next().unwrap_or("").trim().to_string();
            if let Ok(stage) = WorkflowStage::parse(phase) {
                return Some(WorkflowState {
                    stage,
                    task: task.to_string(),
                    feature_name: None,
                    updated_at: if ts.is_empty() {
                        chrono::Utc::now().to_rfc3339()
                    } else {
                        ts
                    },
                });
            }
        }
        None
    }

    /// Validates stage transition and updates state.workflow.
    pub fn validate_and_set_workflow(
        &mut self,
        target_stage: WorkflowStage,
        task: &str,
        feature: Option<String>,
    ) -> Result<(), CeError> {
        let current_stage = self
            .current_workflow()
            .map(|wf| wf.stage)
            .unwrap_or(WorkflowStage::Ideation);
        if !current_stage.can_transition_to(target_stage) {
            return Err(CeError::Usage(format!(
                "invalid workflow transition: cannot jump from Stage {} ({}) directly to Stage {} ({}). Legal transitions: rewind, reset to Stage 1, stay on current stage, or advance to Stage {}.",
                current_stage.number(),
                current_stage.as_str(),
                target_stage.number(),
                target_stage.as_str(),
                current_stage.number() + 1
            )));
        }
        let feature_name =
            feature.or_else(|| self.current_workflow().and_then(|wf| wf.feature_name));
        self.workflow = Some(WorkflowState {
            stage: target_stage,
            task: task.to_string(),
            feature_name,
            updated_at: chrono::Utc::now().to_rfc3339(),
        });
        Ok(())
    }

    pub fn set_model_assignment(&mut self, slot: &str, provider_id: &str, model_id: &str) {
        let assignment = ModelAssignment {
            provider_id: provider_id.into(),
            model_id: model_id.into(),
            effort: None,
        };
        self.model_assignments.insert(slot.into(), assignment);
    }

    /// Merges local workspace overrides on top of global state.
    pub fn merge_overrides(&mut self, local_state: State) {
        for (slot, assignment) in local_state.model_assignments {
            self.model_assignments.insert(slot, assignment);
        }
        if local_state.last_update_check.is_some() {
            self.last_update_check = local_state.last_update_check;
        }
        if local_state.latest_release_tag.is_some() {
            self.latest_release_tag = local_state.latest_release_tag;
        }
    }

    /// Loads global state and applies local `.ce-ai.json` overrides if present.
    pub fn load_with_workspace_overrides(
        global_path: &Path,
        workspace_root: Option<&Path>,
    ) -> Result<Self, CeError> {
        let mut state = Self::load(global_path)?;
        if let Some(ws_root) = workspace_root {
            let local_config = ws_root.join(".ce-ai.json");
            if local_config.exists() {
                let local_state = Self::load(&local_config)?;
                state.merge_overrides(local_state);
            }
        }
        Ok(state)
    }
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
