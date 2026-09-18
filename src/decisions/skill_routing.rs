//! Dynamic Skill Selection & Injection (Intelligent Skill Routing).
//!
//! Provides intent classification and semantic skill category advisory
//! using the System 1 Pluggable Decision Engine.

use serde::{Deserialize, Serialize};

use crate::decisions::types::{DecisionAnswer, DecisionContext, DecisionQuestion, DecisionRequest};
use crate::decisions::DecisionEngine;
use crate::harness::HarnessKind;
use crate::source::registry::{SkillEntry, SkillRegistry};

/// Recognized semantic categories with question prompts.
pub const RECOGNIZED_CATEGORIES: &[(&str, &str)] = &[
    (
        "architecture",
        "Does this task involve system design, architectural boundaries, or data modeling?",
    ),
    (
        "security",
        "Does this task involve security reviews, authentication, authorization, or vulnerabilities?",
    ),
    (
        "testing",
        "Does this task involve creating, updating, or analyzing unit, integration, or E2E tests?",
    ),
    (
        "debugging",
        "Does this task involve diagnosing bugs, investigating errors, stack traces, or regressions?",
    ),
    (
        "documentation",
        "Does this task involve writing or maintaining technical documentation, specifications, or guides?",
    ),
    (
        "code_review",
        "Does this task involve reviewing code changes, checking style standards, or PR feedback?",
    ),
    (
        "research",
        "Does this task require exploring scientific literature, codebase research, or deep algorithmic analysis?",
    ),
];

/// Configuration for dynamic skill routing in `state.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillRoutingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_skill_confidence")]
    pub minimum_confidence_pct: u32,
}

fn default_skill_confidence() -> u32 {
    70
}

impl Default for SkillRoutingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            minimum_confidence_pct: default_skill_confidence(),
        }
    }
}

/// Evaluation score and selection state for a single category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillClassification {
    pub category: String,
    pub confidence: f64,
    pub selected: bool,
}

/// Structured outcome of a skill routing query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillRoutingResult {
    pub task: String,
    pub candidate_categories: Vec<String>,
    pub classifications: Vec<SkillClassification>,
    pub resolved_skills: Vec<SkillEntry>,
    pub fallback_applied: bool,
    pub rationale: String,
    pub latency_ms: u64,
}

/// Semantic skill router using System 1 decision engine.
pub struct SkillRouter<'a> {
    config: &'a SkillRoutingConfig,
    engine: Option<&'a DecisionEngine>,
}

impl<'a> SkillRouter<'a> {
    pub fn new(config: &'a SkillRoutingConfig, engine: Option<&'a DecisionEngine>) -> Self {
        Self { config, engine }
    }

    /// Constructs the 7-dimension category inquiry request.
    pub fn build_request(task: &str) -> DecisionRequest {
        let ctx = DecisionContext::new(task);
        let mut req = DecisionRequest::new(ctx);
        for (cat, q) in RECOGNIZED_CATEGORIES {
            req = req.with_question(DecisionQuestion::boolean(*cat, *q));
        }
        req
    }

    /// Evaluates category intent for the task description.
    /// Returns: `(candidate_categories, classifications, fallback_applied, rationale, latency_ms)`
    pub fn classify_intent(
        &self,
        task: &str,
    ) -> (Vec<String>, Vec<SkillClassification>, bool, String, u64) {
        if !self.config.enabled {
            return (
                Vec::new(),
                Vec::new(),
                true,
                "Skill routing is disabled in configuration.".into(),
                0,
            );
        }

        let engine = match self.engine {
            Some(e) if e.is_enabled() => e,
            _ => {
                return (
                    Vec::new(),
                    Vec::new(),
                    true,
                    "Decision provider is unconfigured or disabled.".into(),
                    0,
                );
            }
        };

        let req = Self::build_request(task);
        let resp = match engine.evaluate(req) {
            Ok(r) => r,
            Err(e) => {
                return (
                    Vec::new(),
                    Vec::new(),
                    true,
                    format!("Decision provider error: {e}"),
                    0,
                );
            }
        };

        if resp.fallback_used {
            return (
                Vec::new(),
                Vec::new(),
                true,
                "Decision provider returned fallback response.".into(),
                resp.latency_ms,
            );
        }

        let threshold = self.config.minimum_confidence_pct as f64 / 100.0;
        let mut candidate_categories = Vec::new();
        let mut classifications = Vec::new();

        for (cat, _) in RECOGNIZED_CATEGORIES {
            let (val, conf) = match resp.get_answer(cat) {
                Some(DecisionAnswer::Boolean { value, confidence }) => (*value, *confidence),
                _ => (false, 0.0),
            };

            let selected = val && conf >= threshold;
            if selected {
                candidate_categories.push(cat.to_string());
            }

            classifications.push(SkillClassification {
                category: cat.to_string(),
                confidence: conf,
                selected,
            });
        }

        let rationale = if candidate_categories.is_empty() {
            "No categories exceeded minimum confidence threshold.".into()
        } else {
            format!(
                "Matched candidate categories: {}",
                candidate_categories.join(", ")
            )
        };

        (
            candidate_categories,
            classifications,
            false,
            rationale,
            resp.latency_ms,
        )
    }

    /// Evaluates semantic intent and resolves matching skills from the authoritative registry.
    pub fn route(
        &self,
        registry: &SkillRegistry,
        harness: HarnessKind,
        task: &str,
    ) -> (SkillRoutingResult, String, String) {
        let (candidate_categories, classifications, fallback, rationale, latency_ms) =
            self.classify_intent(task);

        let (status, resolved_skills, markdown) = if fallback || candidate_categories.is_empty() {
            registry.resolve(harness, task)
        } else {
            registry.resolve_with_routing(harness, task, &candidate_categories)
        };

        let result = SkillRoutingResult {
            task: task.to_string(),
            candidate_categories,
            classifications,
            resolved_skills,
            fallback_applied: fallback,
            rationale,
            latency_ms,
        };

        (result, status, markdown)
    }
}

#[cfg(test)]
#[path = "tests/skill_routing_tests.rs"]
mod tests;
