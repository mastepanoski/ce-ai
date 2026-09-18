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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowSource {
    #[default]
    Manual,
    Inferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureResolution {
    Branch,
    MtimeFallback,
}

impl FeatureResolution {
    pub fn as_str(&self) -> &'static str {
        match self {
            FeatureResolution::Branch => "branch",
            FeatureResolution::MtimeFallback => "mtime_fallback",
        }
    }
}

impl std::fmt::Display for FeatureResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionMode {
    #[default]
    Auto,
    Organic,
    Compound,
}

impl ExecutionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionMode::Auto => "auto",
            ExecutionMode::Organic => "organic",
            ExecutionMode::Compound => "compound",
        }
    }

    pub fn parse(s: &str) -> Result<Self, CeError> {
        let clean = s.trim().to_lowercase();
        match clean.as_str() {
            "auto" => Ok(ExecutionMode::Auto),
            "organic" | "odd" => Ok(ExecutionMode::Organic),
            "compound" | "ce" | "openspec" => Ok(ExecutionMode::Compound),
            _ => Err(CeError::Usage(format!(
                "invalid execution mode '{s}'. Valid modes: auto, organic (odd), compound (ce)"
            ))),
        }
    }
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkflowState {
    pub stage: WorkflowStage,
    pub task: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_name: Option<String>,
    pub updated_at: String,
    #[serde(default)]
    pub source: WorkflowSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<FeatureResolution>,
    #[serde(default)]
    pub new_cycle: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<ExecutionMode>,
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

/// Observe-only ship-readiness receipt: records that a code review ran for a
/// branch head (issue #354). `override_reason` captures an explicit override
/// for audit when the receipt is recorded without a review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReviewReceipt {
    pub head_sha: String,
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_reason: Option<String>,
}

/// The primary decision outcome of a gate check (Issue #333, #334).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateDecision {
    /// Stage 4 (`ce-work`) write actively blocked due to missing OpenSpec contract artifacts in enforce mode.
    Blocked,
    /// Stage 4 (`ce-work`) write with missing OpenSpec contract artifacts (in observe mode).
    WouldBlock,
    /// Authorized write (complete OpenSpec contract, ce-debug, tier minimal, non-Stage 4, or non-target path).
    Pass,
    /// Ambiguous, corrupt, or unadopted checkpoint state.
    Undetermined,
    /// Environmental or lifecycle anomaly isolated from the happy path.
    EdgeCase,
}

impl GateDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            GateDecision::Blocked => "blocked",
            GateDecision::WouldBlock => "would_block",
            GateDecision::Pass => "pass",
            GateDecision::Undetermined => "undetermined",
            GateDecision::EdgeCase => "edge_case",
        }
    }
}

/// Evaluation mode for gate checks: blocking (enforce) vs advisory (observe).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GateMode {
    #[default]
    Enforce,
    Observe,
}

impl GateMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            GateMode::Enforce => "enforce",
            GateMode::Observe => "observe",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "enforce" | "blocking" => Some(GateMode::Enforce),
            "observe" | "advisory" => Some(GateMode::Observe),
            _ => None,
        }
    }
}

/// Structured record of a gate validation outcome (Issue #334).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateReceipt {
    pub timestamp: String,
    pub feature: String,
    pub target_path: String,
    pub decision: GateDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_point: Option<String>,
    pub tier: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_artifacts: Vec<String>,
    pub reason: String,
}

/// Workspace-level configuration for documentation technical debt hygiene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocHygieneConfig {
    #[serde(default = "default_stale_spec_days")]
    pub stale_spec_days: u32,
    #[serde(default = "default_true")]
    pub check_solution_paths: bool,
    #[serde(default = "default_true")]
    pub require_solution_frontmatter: bool,
    #[serde(default = "default_archive_compaction_threshold")]
    pub archive_compaction_threshold: u32,
}

fn default_stale_spec_days() -> u32 {
    21
}

fn default_true() -> bool {
    true
}

fn default_archive_compaction_threshold() -> u32 {
    30
}

impl Default for DocHygieneConfig {
    fn default() -> Self {
        Self {
            stale_spec_days: default_stale_spec_days(),
            check_solution_paths: default_true(),
            require_solution_frontmatter: default_true(),
            archive_compaction_threshold: default_archive_compaction_threshold(),
        }
    }
}

/// Configuration for the optional Decision Engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_provider_name")]
    pub provider: String,
    #[serde(default)]
    pub mode: crate::decisions::DecisionMode,
    #[serde(default)]
    pub budget: crate::decisions::BudgetConfig,
    #[serde(default)]
    pub jev: crate::decisions::JevConfig,
    #[serde(default)]
    pub routing: crate::decisions::ModelRoutingConfig,
    #[serde(default)]
    pub skills: crate::decisions::SkillRoutingConfig,
    #[serde(default)]
    pub risk: crate::decisions::RiskConfig,
}

fn default_provider_name() -> String {
    "jev".to_string()
}

impl Default for DecisionsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_provider_name(),
            mode: crate::decisions::DecisionMode::Active,
            budget: crate::decisions::BudgetConfig::default(),
            jev: crate::decisions::JevConfig::default(),
            routing: crate::decisions::ModelRoutingConfig::default(),
            skills: crate::decisions::SkillRoutingConfig::default(),
            risk: crate::decisions::RiskConfig::default(),
        }
    }
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub workflows: BTreeMap<String, WorkflowState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_release_tag: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projects: Vec<ProjectAdoptionEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_surfaces: Vec<SkillSurface>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardrail: Option<GuardrailState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_checkpoint: Option<bool>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub review_receipts: BTreeMap<String, ReviewReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate_mode: Option<GateMode>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub gate_receipts: BTreeMap<String, GateReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_hygiene: Option<DocHygieneConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decisions: Option<DecisionsConfig>,
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
        let canonical = Self::normalize_workspace_key(target_path);
        self.projects
            .iter()
            .find(|p| p.path == target_path || Self::normalize_workspace_key(&p.path) == canonical)
    }

    /// Checks whether the given workspace/repo root has been adopted.
    pub fn is_project_adopted(&self, repo_root: &Path) -> bool {
        self.project_for_path(repo_root).is_some()
    }

    pub fn is_auto_checkpoint_enabled(&self) -> bool {
        self.auto_checkpoint.unwrap_or(true)
    }

    /// Normalizes a workspace root path into a stable canonical string key.
    pub fn normalize_workspace_key(root: &Path) -> String {
        match std::fs::canonicalize(root) {
            Ok(canonical) => canonical.to_string_lossy().to_string(),
            Err(_) => root.to_string_lossy().to_string(),
        }
    }

    /// Constructs a composite key combining canonical root and git branch.
    /// Falls back to canonical root if branch is None or empty.
    pub fn workspace_branch_key(root: &Path, branch: Option<&str>) -> String {
        let canonical_root = Self::normalize_workspace_key(root);
        match branch.filter(|b| !b.trim().is_empty()) {
            Some(b) => format!("{canonical_root}::{b}"),
            None => canonical_root,
        }
    }

    /// Returns active `WorkflowState` for a specific workspace root and optional branch,
    /// falling back to the canonical root key, legacy global `workflow` field, or `last_update_check`.
    pub fn current_workflow_for_branch(
        &self,
        root: &Path,
        branch: Option<&str>,
    ) -> Option<WorkflowState> {
        let canonical_root = Self::normalize_workspace_key(root);
        if let Some(b) = branch.filter(|b| !b.trim().is_empty()) {
            let branch_key = format!("{canonical_root}::{b}");
            if let Some(wf) = self.workflows.get(&branch_key) {
                return Some(wf.clone());
            }
        }
        if let Some(wf) = self.workflows.get(&canonical_root) {
            return Some(wf.clone());
        }
        // If branch was not specified, search for the most recently updated branch entry for this workspace
        if branch.is_none() {
            let prefix = format!("{canonical_root}::");
            let mut matches: Vec<&WorkflowState> = self
                .workflows
                .iter()
                .filter(|(k, _)| k.starts_with(&prefix))
                .map(|(_, v)| v)
                .collect();
            if !matches.is_empty() {
                matches.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                return Some(matches[0].clone());
            }
        }
        if self.workflows.is_empty() {
            return self.current_workflow();
        }
        None
    }

    /// Returns active `WorkflowState` for a specific workspace root,
    /// falling back to the legacy global `workflow` field or `last_update_check`.
    pub fn current_workflow_for(&self, root: &Path) -> Option<WorkflowState> {
        self.current_workflow_for_branch(root, None)
    }

    /// Returns active `WorkflowState`, falling back to legacy `last_update_check` parsing if present.
    pub fn current_workflow(&self) -> Option<WorkflowState> {
        if let Ok(cwd) = std::env::current_dir() {
            let key = Self::normalize_workspace_key(&cwd);
            if let Some(wf) = self.workflows.get(&key) {
                return Some(wf.clone());
            }
        }
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
                    source: WorkflowSource::Manual,
                    resolution: None,
                    new_cycle: false,
                    execution_mode: None,
                });
            }
        }
        None
    }

    /// Validates stage transition and updates state.workflows for the specified workspace and branch.
    pub fn validate_and_set_workflow_for_branch(
        &mut self,
        root: &Path,
        branch: Option<&str>,
        target_stage: WorkflowStage,
        task: &str,
        feature: Option<String>,
        source: WorkflowSource,
    ) -> Result<(), CeError> {
        self.validate_and_set_workflow_for_branch_with_resolution(
            root,
            branch,
            target_stage,
            task,
            feature,
            source,
            None,
        )
    }

    /// Validates stage transition and updates state.workflows with optional feature resolution provenance.
    #[allow(clippy::too_many_arguments)]
    pub fn validate_and_set_workflow_for_branch_with_resolution(
        &mut self,
        root: &Path,
        branch: Option<&str>,
        target_stage: WorkflowStage,
        task: &str,
        feature: Option<String>,
        source: WorkflowSource,
        resolution: Option<FeatureResolution>,
    ) -> Result<(), CeError> {
        let current_wf = self.current_workflow_for_branch(root, branch);
        let current_stage = current_wf
            .as_ref()
            .map(|wf| wf.stage)
            .unwrap_or(WorkflowStage::Ideation);

        let is_new_cycle = match (&current_wf, &feature) {
            (Some(wf), feat) => {
                target_stage.number() == 1 && wf.feature_name.as_deref() != feat.as_deref()
            }
            _ => false,
        };

        // Monotonic provenance guard:
        // 1. Inferred checkpoints can NEVER equal or regress an existing Manual checkpoint.
        // 2. Inferred checkpoints can NEVER regress any existing checkpoint (Manual or Inferred).
        // Exception: New cycle detected (target_stage is Stage 1 and feature changed).
        if let Some(ref wf) = current_wf {
            if source == WorkflowSource::Inferred && !is_new_cycle {
                if wf.source == WorkflowSource::Manual
                    && target_stage.number() <= current_stage.number()
                {
                    return Ok(());
                }
                if target_stage.number() < current_stage.number() {
                    return Ok(());
                }
            }
        }

        if !current_stage.can_transition_to(target_stage) {
            if source == WorkflowSource::Inferred {
                return Ok(());
            }
            return Err(CeError::Usage(format!(
                "invalid workflow transition: cannot jump from Stage {} ({}) directly to Stage {} ({}). Legal transitions: rewind, reset to Stage 1, stay on current stage, or advance to Stage {}.",
                current_stage.number(),
                current_stage.as_str(),
                target_stage.number(),
                target_stage.as_str(),
                current_stage.number() + 1
            )));
        }
        let is_reset_to_stage_1 =
            target_stage == WorkflowStage::Ideation && current_stage != WorkflowStage::Ideation;

        let (feature_name, resolution, execution_mode) = match feature {
            Some(f) => {
                let trimmed = f.trim().to_string();
                let current = self.current_workflow_for_branch(root, branch);
                let mode = current.as_ref().and_then(|wf| wf.execution_mode);
                if trimmed.is_empty() {
                    (None, None, mode)
                } else {
                    (Some(trimmed), resolution, mode)
                }
            }
            None => {
                if is_reset_to_stage_1 {
                    (None, None, None)
                } else {
                    let current = self.current_workflow_for_branch(root, branch);
                    let feat = current.as_ref().and_then(|wf| wf.feature_name.clone());
                    let res = resolution.or_else(|| current.as_ref().and_then(|wf| wf.resolution));
                    let mode = current.as_ref().and_then(|wf| wf.execution_mode);
                    (feat, res, mode)
                }
            }
        };

        let new_wf = WorkflowState {
            stage: target_stage,
            task: task.to_string(),
            feature_name,
            updated_at: chrono::Utc::now().to_rfc3339(),
            source,
            resolution,
            new_cycle: is_new_cycle,
            execution_mode,
        };

        let key = Self::workspace_branch_key(root, branch);
        self.workflows.insert(key, new_wf.clone());
        self.workflow = Some(new_wf);
        Ok(())
    }

    /// Sets or clears the execution mode for the specified workspace and branch.
    pub fn set_execution_mode_for_branch(
        &mut self,
        root: &Path,
        branch: Option<&str>,
        mode: Option<ExecutionMode>,
    ) {
        let key = Self::workspace_branch_key(root, branch);
        if let Some(wf) = self.workflows.get_mut(&key) {
            wf.execution_mode = mode;
            wf.updated_at = chrono::Utc::now().to_rfc3339();
        }
        if let Some(ref mut wf) = self.workflow {
            wf.execution_mode = mode;
            wf.updated_at = chrono::Utc::now().to_rfc3339();
        }
    }

    /// Validates stage transition and updates state.workflows for the specified workspace.
    pub fn validate_and_set_workflow_for(
        &mut self,
        root: &Path,
        target_stage: WorkflowStage,
        task: &str,
        feature: Option<String>,
    ) -> Result<(), CeError> {
        self.validate_and_set_workflow_for_branch(
            root,
            None,
            target_stage,
            task,
            feature,
            WorkflowSource::Manual,
        )
    }

    /// Mutates workflow state using a reload-before-save check to prevent multi-turn read-modify-write clobbering.
    pub fn atomic_update_workflow<F>(
        state_path: &Path,
        root: &Path,
        branch: Option<&str>,
        mutator: F,
    ) -> Result<WorkflowState, CeError>
    where
        F: FnOnce(&mut State) -> Result<Option<WorkflowState>, CeError>,
    {
        let mut state = if state_path.exists() {
            State::load(state_path)?
        } else {
            State::default()
        };

        let updated_opt = mutator(&mut state)?;
        if let Some(new_wf) = updated_opt {
            let key = Self::workspace_branch_key(root, branch);
            state.workflows.insert(key, new_wf.clone());
            state.workflow = Some(new_wf.clone());
            state.save(state_path)?;
            Ok(new_wf)
        } else {
            Ok(state
                .current_workflow_for_branch(root, branch)
                .unwrap_or_else(|| WorkflowState {
                    stage: WorkflowStage::Ideation,
                    task: "No active task recorded".to_string(),
                    feature_name: None,
                    updated_at: chrono::Utc::now().to_rfc3339(),
                    source: WorkflowSource::Manual,
                    resolution: None,
                    new_cycle: false,
                    execution_mode: None,
                }))
        }
    }

    /// Returns the code-review receipt recorded for the given workspace+branch, if any.
    pub fn review_receipt_for_branch(
        &self,
        repo_root: &Path,
        branch: Option<&str>,
    ) -> Option<&ReviewReceipt> {
        let key = Self::workspace_branch_key(repo_root, branch);
        self.review_receipts.get(&key)
    }

    /// Records a code-review receipt for the given workspace+branch and returns it.
    pub fn record_review_receipt(
        &mut self,
        repo_root: &Path,
        branch: Option<&str>,
        head_sha: &str,
        override_reason: Option<String>,
    ) -> ReviewReceipt {
        let key = Self::workspace_branch_key(repo_root, branch);
        let receipt = ReviewReceipt {
            head_sha: head_sha.to_string(),
            recorded_at: chrono::Utc::now().to_rfc3339(),
            override_reason,
        };
        self.review_receipts.insert(key, receipt.clone());
        receipt
    }

    /// Validates stage transition and updates state for the current working directory.
    pub fn validate_and_set_workflow(
        &mut self,
        target_stage: WorkflowStage,
        task: &str,
        feature: Option<String>,
    ) -> Result<(), CeError> {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        self.validate_and_set_workflow_for(&cwd, target_stage, task, feature)
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
        if local_state.doc_hygiene.is_some() {
            self.doc_hygiene = local_state.doc_hygiene;
        }
    }

    /// Returns the effective documentation hygiene configuration, falling back to defaults if not set.
    pub fn doc_hygiene(&self) -> DocHygieneConfig {
        self.doc_hygiene.unwrap_or_default()
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
