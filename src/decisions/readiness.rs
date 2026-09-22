//! Work Readiness & Verification Advisory Engine (System 1).
//!
//! Provides fast, structured semantic readiness evaluation across:
//! 1. Organic Driven Development (ODD) task briefs (`odd/tasks/<feature>.md`).
//! 2. Compound Engineering 7-Stage FSM transitions.
//!
//! Enforces the core architectural invariant:
//! - Decision Engine -> evaluates semantic readiness (advisory only).
//! - Deterministic Engine (FSM / Gate / Bridge) -> authorizes transitions & enforcement.
//!
//! The Decision Engine never mutates workflow state or overrides deterministic validation.

use serde::{Deserialize, Serialize};

use crate::decisions::types::{DecisionAnswer, DecisionContext, DecisionQuestion, DecisionRequest};
use crate::decisions::DecisionEngine;

/// High-level advisory readiness status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessStatus {
    /// Composite confidence meets or exceeds ready threshold (>= 80%).
    Ready,
    /// Marginal confidence (60%..79%); attention recommended on highlighted dimensions.
    Warning,
    /// Low confidence (< 60%); incomplete implementation, missing tests, or unverified DoD.
    NotReady,
}

impl ReadinessStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Warning => "warning",
            Self::NotReady => "not_ready",
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Ready => "✓ Ready",
            Self::Warning => "△ Needs Attention",
            Self::NotReady => "⚠ Incomplete",
        }
    }
}

/// Configurable integer-percentage thresholds for readiness status mapping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessThresholds {
    #[serde(default = "default_ready_threshold")]
    pub ready_pct: u32,
    #[serde(default = "default_warning_threshold")]
    pub warning_pct: u32,
}

fn default_ready_threshold() -> u32 {
    80
}

fn default_warning_threshold() -> u32 {
    60
}

impl Default for ReadinessThresholds {
    fn default() -> Self {
        Self {
            ready_pct: default_ready_threshold(),
            warning_pct: default_warning_threshold(),
        }
    }
}

/// Configuration for readiness evaluation in `state.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReadinessConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub thresholds: ReadinessThresholds,
}

/// Standardized semantic dimension questions for Organic Driven Development (ODD).
pub const ODD_READINESS_DIMENSIONS: &[(&str, &str)] = &[
    (
        "dod_satisfied",
        "Are the Definition of Done checklist items satisfied by the current changes and implementation?",
    ),
    (
        "guardrails_respected",
        "Were all task boundaries, guardrails, and negative constraints strictly honored?",
    ),
    (
        "graduation_recommended",
        "Does the task scope, architectural footprint, or risk warrant graduating to formal OpenSpec?",
    ),
    (
        "ready_to_close",
        "Is this organic task complete and ready to be closed?",
    ),
];

/// Standardized semantic dimension questions for Compound Engineering (CE) stages.
pub const CE_READINESS_DIMENSIONS: &[(&str, &str)] = &[
    (
        "requirements_clear",
        "Are the requirements, scope, and acceptance criteria unambiguous?",
    ),
    (
        "implementation_complete",
        "Is the planned implementation code complete according to the task checklist?",
    ),
    (
        "tests_sufficient",
        "Are the unit, integration, or quality gate tests sufficient to verify the behavior?",
    ),
    (
        "docs_complete",
        "Is documentation, changelog, or architecture notes properly updated?",
    ),
];

/// Evaluation confidence for an individual readiness dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessDimensionScore {
    pub dimension: String,
    pub confidence: f64,
    pub passed: bool,
}

/// Structured outcome of a readiness evaluation query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessEvaluationResult {
    pub target: String,
    pub workflow_mode: String,
    pub status: ReadinessStatus,
    pub composite_score: f64,
    pub dimensions: Vec<ReadinessDimensionScore>,
    pub advisory_notes: Vec<String>,
    pub graduation_suggested: bool,
    pub fallback_applied: bool,
    pub latency_ms: u64,
}

/// Evaluator orchestrating semantic readiness queries against the Decision Engine.
pub struct ReadinessEvaluator<'a> {
    config: &'a ReadinessConfig,
    engine: Option<&'a DecisionEngine>,
    config_dir: Option<&'a std::path::Path>,
    workflow_id: Option<String>,
}

impl<'a> ReadinessEvaluator<'a> {
    pub fn new(config: &'a ReadinessConfig, engine: Option<&'a DecisionEngine>) -> Self {
        Self {
            config,
            engine,
            config_dir: None,
            workflow_id: None,
        }
    }

    pub fn with_config_dir(mut self, config_dir: &'a std::path::Path) -> Self {
        self.config_dir = Some(config_dir);
        self
    }

    pub fn with_workflow(mut self, workflow_id: Option<impl Into<String>>) -> Self {
        self.workflow_id = workflow_id.map(Into::into);
        self
    }

    /// Evaluates readiness for an Organic Driven Development (ODD) task brief.
    pub fn evaluate_odd(
        &self,
        feature: &str,
        brief_summary: &str,
        checklist_status: &str,
        diff_stat: Option<&str>,
    ) -> ReadinessEvaluationResult {
        if !self.config.enabled {
            return ReadinessEvaluationResult {
                target: feature.to_string(),
                workflow_mode: "organic".into(),
                status: ReadinessStatus::Ready,
                composite_score: 1.0,
                dimensions: Vec::new(),
                advisory_notes: vec!["Readiness advisory is disabled in configuration.".into()],
                graduation_suggested: false,
                fallback_applied: true,
                latency_ms: 0,
            };
        }

        let engine = match self.engine {
            Some(e) if e.is_enabled() => e,
            _ => {
                return ReadinessEvaluationResult {
                    target: feature.to_string(),
                    workflow_mode: "organic".into(),
                    status: ReadinessStatus::Ready,
                    composite_score: 1.0,
                    dimensions: Vec::new(),
                    advisory_notes: vec![
                        "Decision engine unconfigured; fallback applied (deterministic workflow active)."
                            .into(),
                    ],
                    graduation_suggested: false,
                    fallback_applied: true,
                    latency_ms: 0,
                };
            }
        };

        let mut prompt = format!(
            "Workflow: Organic Driven Development (ODD)\nFeature: {feature}\nBrief Summary:\n{brief_summary}\nChecklist:\n{checklist_status}"
        );
        if let Some(diff) = diff_stat {
            prompt.push_str(&format!("\nGit Diff Stat:\n{diff}"));
        }

        let mut req = DecisionRequest::new(DecisionContext::new(&prompt));
        for (dim, q) in ODD_READINESS_DIMENSIONS {
            req = req.with_question(DecisionQuestion::boolean(*dim, *q));
        }

        let resp = match engine.evaluate(req) {
            Ok(r) if !r.fallback_used || r.shadow_mode => r,
            _ => {
                return ReadinessEvaluationResult {
                    target: feature.to_string(),
                    workflow_mode: "organic".into(),
                    status: ReadinessStatus::Ready,
                    composite_score: 1.0,
                    dimensions: Vec::new(),
                    advisory_notes: vec![
                        "Decision provider unavailable or timed out; fallback applied.".into(),
                    ],
                    graduation_suggested: false,
                    fallback_applied: true,
                    latency_ms: 0,
                };
            }
        };

        self.process_odd_response(feature, resp)
    }

    fn process_odd_response(
        &self,
        feature: &str,
        resp: crate::decisions::types::DecisionResponse,
    ) -> ReadinessEvaluationResult {
        let ready_thresh = self.config.thresholds.ready_pct as f64 / 100.0;
        let warning_thresh = self.config.thresholds.warning_pct as f64 / 100.0;

        let mut dimensions = Vec::new();
        let mut sum_score = 0.0;
        let mut count = 0;
        let mut advisory_notes = Vec::new();
        let mut graduation_suggested = false;

        for (dim, _) in ODD_READINESS_DIMENSIONS {
            let (val, conf) = match resp.get_answer(dim) {
                Some(DecisionAnswer::Boolean { value, confidence }) => (*value, *confidence),
                _ => (false, 0.0),
            };

            let score = if val { conf } else { 1.0 - conf };

            if *dim == "graduation_recommended" {
                // Graduation is an advisory recommendation rather than a task incompleteness penalty
                if val && conf >= warning_thresh {
                    graduation_suggested = true;
                    advisory_notes.push(format!(
                        "Graduation recommended ({:.0}% confidence): Scope or complexity suggests promoting to formal OpenSpec via 'ce-ai graduate'.",
                        conf * 100.0
                    ));
                }
            } else {
                sum_score += score;
                count += 1;

                if !val || conf < warning_thresh {
                    advisory_notes.push(format!(
                        "Dimension '{dim}' needs attention (confidence: {:.0}%).",
                        conf * 100.0
                    ));
                }
            }

            dimensions.push(ReadinessDimensionScore {
                dimension: dim.to_string(),
                confidence: conf,
                passed: if *dim == "graduation_recommended" {
                    !val || conf < warning_thresh
                } else {
                    val && conf >= warning_thresh
                },
            });
        }

        let composite_score = if count > 0 {
            sum_score / count as f64
        } else {
            1.0
        };

        let status = if composite_score >= ready_thresh {
            ReadinessStatus::Ready
        } else if composite_score >= warning_thresh {
            ReadinessStatus::Warning
        } else {
            ReadinessStatus::NotReady
        };

        if resp.shadow_mode {
            advisory_notes.insert(
                0,
                "Shadow mode active: semantic readiness evaluated without workflow enforcement."
                    .into(),
            );
        }

        if let Some(cd) = self.config_dir {
            let event = crate::decisions::analytics::DecisionEvent::new(
                crate::decisions::analytics::DecisionType::StageReadiness,
                &resp.provider,
                &resp.model,
                resp.latency_ms,
                status.as_str(),
            )
            .with_workflow(
                self.workflow_id
                    .clone()
                    .or_else(|| Some(feature.to_string())),
            )
            .with_confidence(Some(composite_score))
            .with_shadow(resp.shadow_mode)
            .with_fallback(false)
            .with_estimated_cost(resp.estimated_cost_usd)
            .with_metadata("target", feature.to_string())
            .with_metadata("workflow_mode", "organic");
            let _ = crate::decisions::analytics::log_decision_event(cd, &event);
        }

        ReadinessEvaluationResult {
            target: feature.to_string(),
            workflow_mode: "organic".into(),
            status,
            composite_score,
            dimensions,
            advisory_notes,
            graduation_suggested,
            fallback_applied: false,
            latency_ms: resp.latency_ms,
        }
    }

    /// Evaluates readiness for a Compound Engineering stage transition.
    pub fn evaluate_stage(
        &self,
        feature: &str,
        stage_num: u32,
        stage_name: &str,
        context_summary: &str,
        diff_stat: Option<&str>,
    ) -> ReadinessEvaluationResult {
        if !self.config.enabled {
            return ReadinessEvaluationResult {
                target: format!("{feature} (Stage {stage_num}: {stage_name})"),
                workflow_mode: "compound".into(),
                status: ReadinessStatus::Ready,
                composite_score: 1.0,
                dimensions: Vec::new(),
                advisory_notes: vec!["Readiness advisory is disabled in configuration.".into()],
                graduation_suggested: false,
                fallback_applied: true,
                latency_ms: 0,
            };
        }

        let engine = match self.engine {
            Some(e) if e.is_enabled() => e,
            _ => {
                return ReadinessEvaluationResult {
                    target: format!("{feature} (Stage {stage_num}: {stage_name})"),
                    workflow_mode: "compound".into(),
                    status: ReadinessStatus::Ready,
                    composite_score: 1.0,
                    dimensions: Vec::new(),
                    advisory_notes: vec![
                        "Decision engine unconfigured; fallback applied (deterministic workflow active)."
                            .into(),
                    ],
                    graduation_suggested: false,
                    fallback_applied: true,
                    latency_ms: 0,
                };
            }
        };

        let mut prompt = format!(
            "Workflow: Compound Engineering FSM\nFeature: {feature}\nActive Stage: {stage_num} ({stage_name})\nContext:\n{context_summary}"
        );
        if let Some(diff) = diff_stat {
            prompt.push_str(&format!("\nGit Diff Stat:\n{diff}"));
        }

        let mut req = DecisionRequest::new(DecisionContext::new(&prompt));
        for (dim, q) in CE_READINESS_DIMENSIONS {
            req = req.with_question(DecisionQuestion::boolean(*dim, *q));
        }

        let resp = match engine.evaluate(req) {
            Ok(r) if !r.fallback_used || r.shadow_mode => r,
            _ => {
                return ReadinessEvaluationResult {
                    target: format!("{feature} (Stage {stage_num}: {stage_name})"),
                    workflow_mode: "compound".into(),
                    status: ReadinessStatus::Ready,
                    composite_score: 1.0,
                    dimensions: Vec::new(),
                    advisory_notes: vec![
                        "Decision provider unavailable or timed out; fallback applied.".into(),
                    ],
                    graduation_suggested: false,
                    fallback_applied: true,
                    latency_ms: 0,
                };
            }
        };

        self.process_stage_response(feature, stage_num, stage_name, resp)
    }

    fn process_stage_response(
        &self,
        feature: &str,
        stage_num: u32,
        stage_name: &str,
        resp: crate::decisions::types::DecisionResponse,
    ) -> ReadinessEvaluationResult {
        let ready_thresh = self.config.thresholds.ready_pct as f64 / 100.0;
        let warning_thresh = self.config.thresholds.warning_pct as f64 / 100.0;

        let mut dimensions = Vec::new();
        let mut sum_score = 0.0;
        let mut count = 0;
        let mut advisory_notes = Vec::new();

        for (dim, _) in CE_READINESS_DIMENSIONS {
            let (val, conf) = match resp.get_answer(dim) {
                Some(DecisionAnswer::Boolean { value, confidence }) => (*value, *confidence),
                _ => (false, 0.0),
            };

            let score = if val { conf } else { 1.0 - conf };
            sum_score += score;
            count += 1;

            if !val || conf < warning_thresh {
                advisory_notes.push(format!(
                    "Dimension '{dim}' needs attention (confidence: {:.0}%).",
                    conf * 100.0
                ));
            }

            dimensions.push(ReadinessDimensionScore {
                dimension: dim.to_string(),
                confidence: conf,
                passed: val && conf >= warning_thresh,
            });
        }

        let composite_score = if count > 0 {
            sum_score / count as f64
        } else {
            1.0
        };

        let status = if composite_score >= ready_thresh {
            ReadinessStatus::Ready
        } else if composite_score >= warning_thresh {
            ReadinessStatus::Warning
        } else {
            ReadinessStatus::NotReady
        };

        if resp.shadow_mode {
            advisory_notes.insert(
                0,
                "Shadow mode active: semantic readiness evaluated without workflow enforcement."
                    .into(),
            );
        }

        if let Some(cd) = self.config_dir {
            let event = crate::decisions::analytics::DecisionEvent::new(
                crate::decisions::analytics::DecisionType::StageReadiness,
                &resp.provider,
                &resp.model,
                resp.latency_ms,
                status.as_str(),
            )
            .with_workflow(
                self.workflow_id
                    .clone()
                    .or_else(|| Some(feature.to_string())),
            )
            .with_stage(Some(stage_num.to_string()))
            .with_confidence(Some(composite_score))
            .with_shadow(resp.shadow_mode)
            .with_fallback(false)
            .with_estimated_cost(resp.estimated_cost_usd)
            .with_metadata(
                "target",
                format!("{feature} (Stage {stage_num}: {stage_name})"),
            )
            .with_metadata("workflow_mode", "compound");
            let _ = crate::decisions::analytics::log_decision_event(cd, &event);
        }

        ReadinessEvaluationResult {
            target: format!("{feature} (Stage {stage_num}: {stage_name})"),
            workflow_mode: "compound".into(),
            status,
            composite_score,
            dimensions,
            advisory_notes,
            graduation_suggested: false,
            fallback_applied: false,
            latency_ms: resp.latency_ms,
        }
    }
}

#[cfg(test)]
#[path = "tests/readiness_tests.rs"]
mod tests;
