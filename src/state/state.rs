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
mod tests {
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    use crate::state::state::{ModelAssignment, ReleaseProvenance, State};

    fn state_with(slot: &str) -> State {
        State {
            version: 1,
            installed_harnesses: vec![],
            managed_asset_digest: BTreeMap::new(),
            release_provenance: None,
            model_assignments: BTreeMap::from([(
                slot.to_string(),
                ModelAssignment {
                    provider_id: "opencode-go".into(),
                    model_id: "kimi-k2.6".into(),
                    effort: None,
                },
            )]),
            last_update_check: None,
            workflow: None,
            latest_release_tag: None,
            projects: vec![],
        }
    }

    #[test]
    fn round_trips_through_state_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let state = state_with("ce-brainstorm");
        state.save(&path).unwrap();
        assert_eq!(State::load(&path).unwrap(), state);
    }

    #[test]
    fn atomic_write_leaves_no_temp_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        state_with("ce-brainstorm").save(&path).unwrap();
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn persists_model_assignments_across_reloads() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut state = State::new();
        state.set_model_assignment("ce-brainstorm", "opencode-go", "kimi-k2.6");
        state.save(&path).unwrap();
        let assignment = &State::load(&path).unwrap().model_assignments["ce-brainstorm"];
        assert_eq!(assignment.provider_id, "opencode-go");
        assert_eq!(assignment.model_id, "kimi-k2.6");
    }

    #[test]
    fn load_missing_file_returns_default_state() {
        let dir = tempdir().unwrap();
        let loaded = State::load(&dir.path().join("absent.json")).unwrap();
        assert_eq!(loaded.version, 1);
        assert!(loaded.model_assignments.is_empty());
    }

    #[test]
    fn workspace_overrides_precedence() {
        let dir = tempdir().unwrap();
        let global_path = dir.path().join("global_state.json");
        let ws_dir = dir.path().join("workspace");
        std::fs::create_dir_all(&ws_dir).unwrap();
        let local_path = ws_dir.join(".ce-ai.json");

        let mut global = State::new();
        global.set_model_assignment("ce-brainstorm", "opencode-go", "kimi-k2.6");
        global.set_model_assignment("ce-work", "anthropic", "claude-3-5-sonnet");
        global.save(&global_path).unwrap();

        let mut local = State::new();
        local.set_model_assignment("ce-work", "openai", "gpt-4o");
        local.save(&local_path).unwrap();

        let loaded = State::load_with_workspace_overrides(&global_path, Some(&ws_dir)).unwrap();
        assert_eq!(
            loaded.model_assignments["ce-brainstorm"].provider_id,
            "opencode-go"
        );
        assert_eq!(loaded.model_assignments["ce-work"].provider_id, "openai");
        assert_eq!(loaded.model_assignments["ce-work"].model_id, "gpt-4o");
    }

    #[test]
    fn project_adoption_entry_roundtrip() {
        use crate::state::state::{AdoptionTier, ProjectAdoptionEntry};
        use std::path::PathBuf;

        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut state = State::new();
        state.projects.push(ProjectAdoptionEntry {
            path: PathBuf::from("/tmp/repo"),
            file: "AGENTS.md".into(),
            tier: AdoptionTier::Full,
            block_version: 1,
            block_sha256: "abc123sha".into(),
            created_file: true,
            adopted_at: "2026-08-22T00:00:00Z".into(),
        });
        state.save(&path).unwrap();

        let loaded = State::load(&path).unwrap();
        assert_eq!(loaded.projects.len(), 1);
        assert_eq!(loaded.projects[0].path, PathBuf::from("/tmp/repo"));
        assert_eq!(loaded.projects[0].tier, AdoptionTier::Full);
        assert!(loaded.projects[0].created_file);
    }

    #[test]
    fn release_provenance_round_trips_through_state_json() {
        use std::path::PathBuf;

        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let mut state = State::new();
        state
            .managed_asset_digest
            .insert("tarball".into(), "sha256:deadbeef".into());
        state.release_provenance = Some(ReleaseProvenance {
            tag: "v1.18.0".into(),
            url: "https://github.com/everyinc/compound-engineering-plugin/archive/refs/tags/v1.18.0.tar.gz".into(),
            archive_sha256: "deadbeef".into(),
            extraction_path: PathBuf::from("/tmp/ce-ai/cache/trees/v1.18.0"),
        });
        state.save(&path).unwrap();

        let loaded = State::load(&path).unwrap();
        let prov = loaded.release_provenance.expect("provenance persisted");
        assert_eq!(prov.tag, "v1.18.0");
        assert_eq!(prov.archive_sha256, "deadbeef");
        assert_eq!(
            loaded
                .managed_asset_digest
                .get("tarball")
                .map(String::as_str),
            Some("sha256:deadbeef")
        );
    }

    #[test]
    fn legacy_state_without_provenance_loads() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        let legacy = serde_json::json!({
            "version": 1,
            "installed_harnesses": [],
            "managed_asset_digest": { "tarball": "sha256:abc" }
        });
        std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();

        let loaded = State::load(&path).unwrap();
        assert!(loaded.release_provenance.is_none());
        assert_eq!(
            loaded
                .managed_asset_digest
                .get("tarball")
                .map(String::as_str),
            Some("sha256:abc")
        );
    }
}
