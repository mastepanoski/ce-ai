//! Adaptive Model Route Selection (System One Advisory Layer).
//!
//! Provides deterministic model class recommendation and dynamic routing based on
//! lightweight task classification questions (`complexity`, `needs_reasoning`,
//! `needs_large_context`, `risk`).

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::decisions::types::{
    DecisionAnswer, DecisionContext, DecisionQuestion, DecisionRequest, DecisionResponse,
};
use crate::decisions::DecisionEngine;
use crate::error::CeError;

/// Logical model capability class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelClass {
    Fast,
    Standard,
    Reasoning,
}

impl ModelClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelClass::Fast => "fast",
            ModelClass::Standard => "standard",
            ModelClass::Reasoning => "reasoning",
        }
    }

    pub fn parse(s: &str) -> Result<Self, CeError> {
        let clean = s.trim().to_lowercase();
        match clean.as_str() {
            "fast" => Ok(ModelClass::Fast),
            "standard" => Ok(ModelClass::Standard),
            "reasoning" => Ok(ModelClass::Reasoning),
            _ => Err(CeError::Usage(format!(
                "invalid model class '{s}'. Valid classes: fast, standard, reasoning"
            ))),
        }
    }
}

impl fmt::Display for ModelClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Catalog mapping logical model classes to concrete provider model identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModelClassCatalog {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fast: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

impl ModelClassCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, class: ModelClass) -> Option<&str> {
        match class {
            ModelClass::Fast => self.fast.as_deref(),
            ModelClass::Standard => self.standard.as_deref(),
            ModelClass::Reasoning => self.reasoning.as_deref(),
        }
    }

    pub fn set(&mut self, class: ModelClass, model: impl Into<String>) {
        let val = Some(model.into());
        match class {
            ModelClass::Fast => self.fast = val,
            ModelClass::Standard => self.standard = val,
            ModelClass::Reasoning => self.reasoning = val,
        }
    }

    /// Resolves a model identifier for the given class, following a capability fallback chain
    /// if the preferred class is unmapped in the catalog.
    /// Returns `(resolved_model, fallback_applied)`.
    pub fn resolve_with_fallback(&self, target: ModelClass, default_model: &str) -> (String, bool) {
        if let Some(m) = self.get(target) {
            return (m.to_string(), false);
        }

        // Degradation chain per target
        let chain = match target {
            ModelClass::Reasoning => [ModelClass::Standard, ModelClass::Fast],
            ModelClass::Fast => [ModelClass::Standard, ModelClass::Reasoning],
            ModelClass::Standard => [ModelClass::Fast, ModelClass::Reasoning],
        };

        for candidate in chain {
            if let Some(m) = self.get(candidate) {
                return (m.to_string(), true);
            }
        }

        (default_model.to_string(), true)
    }
}

/// Confidence and classification thresholds for model routing.
/// Stored as integer percentages (`0..=100`) to guarantee `Eq` derivation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingThresholds {
    #[serde(default = "default_reasoning_threshold")]
    pub reasoning_threshold_pct: u32,
    #[serde(default = "default_standard_threshold")]
    pub standard_threshold_pct: u32,
}

fn default_reasoning_threshold() -> u32 {
    75
}

fn default_standard_threshold() -> u32 {
    50
}

impl Default for RoutingThresholds {
    fn default() -> Self {
        Self {
            reasoning_threshold_pct: default_reasoning_threshold(),
            standard_threshold_pct: default_standard_threshold(),
        }
    }
}

/// Persistent configuration for adaptive model routing in `state.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModelRoutingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub models: ModelClassCatalog,
    #[serde(default)]
    pub thresholds: RoutingThresholds,
}

/// Structured resolution outcome returned by the routing engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingResolution {
    pub task: String,
    pub recommended_class: ModelClass,
    pub resolved_model: String,
    pub complexity: String,
    pub needs_reasoning: bool,
    pub needs_large_context: bool,
    pub risk: String,
    pub rationale: String,
    pub fallback_applied: bool,
    pub latency_ms: u64,
}

/// Adaptive model routing engine.
pub struct ModelRouter<'a> {
    config: &'a ModelRoutingConfig,
    engine: Option<&'a DecisionEngine>,
}

impl<'a> ModelRouter<'a> {
    pub fn new(config: &'a ModelRoutingConfig, engine: Option<&'a DecisionEngine>) -> Self {
        Self { config, engine }
    }

    /// Constructs the standard 4-dimension classification request.
    pub fn build_request(task: &str) -> DecisionRequest {
        let ctx = DecisionContext::new(task);
        DecisionRequest::new(ctx)
            .with_question(DecisionQuestion::choice(
                "complexity",
                "Assess the implementation complexity of this task",
                vec!["trivial", "moderate", "complex"],
            ))
            .with_question(DecisionQuestion::boolean(
                "needs_reasoning",
                "Does this task require deep multi-step architectural reasoning or intricate debugging?",
            ))
            .with_question(DecisionQuestion::boolean(
                "needs_large_context",
                "Does this task require reading and reasoning across a large codebase or multiple repositories?",
            ))
            .with_question(DecisionQuestion::choice(
                "risk",
                "What is the operational or security risk if this task is misimplemented?",
                vec!["low", "medium", "high"],
            ))
    }

    /// Evaluates routing for a given task description, enforcing strict override precedence:
    /// 1. Explicit override
    /// 2. Agent slot static assignment
    /// 3. Adaptive routing (if enabled and provider available)
    /// 4. Default harness model fallback
    pub fn route(
        &self,
        task: &str,
        default_model: &str,
        explicit_override: Option<&str>,
        slot_assignment: Option<&str>,
    ) -> RoutingResolution {
        // 1. Explicit override precedence
        if let Some(explicit) = explicit_override {
            return RoutingResolution {
                task: task.to_string(),
                recommended_class: ModelClass::Standard,
                resolved_model: explicit.to_string(),
                complexity: "unspecified".into(),
                needs_reasoning: false,
                needs_large_context: false,
                risk: "unspecified".into(),
                rationale: "Explicit model override applied.".into(),
                fallback_applied: false,
                latency_ms: 0,
            };
        }

        // 2. Agent slot static assignment precedence
        if let Some(slot_model) = slot_assignment {
            return RoutingResolution {
                task: task.to_string(),
                recommended_class: ModelClass::Standard,
                resolved_model: slot_model.to_string(),
                complexity: "unspecified".into(),
                needs_reasoning: false,
                needs_large_context: false,
                risk: "unspecified".into(),
                rationale: "Static agent slot assignment applied.".into(),
                fallback_applied: false,
                latency_ms: 0,
            };
        }

        // 3. Check if adaptive routing is enabled
        if !self.config.enabled {
            return RoutingResolution {
                task: task.to_string(),
                recommended_class: ModelClass::Standard,
                resolved_model: default_model.to_string(),
                complexity: "unspecified".into(),
                needs_reasoning: false,
                needs_large_context: false,
                risk: "unspecified".into(),
                rationale: "Adaptive routing is disabled; default model selected.".into(),
                fallback_applied: true,
                latency_ms: 0,
            };
        }

        let engine = match self.engine {
            Some(e) if e.is_enabled() => e,
            _ => {
                return RoutingResolution {
                    task: task.to_string(),
                    recommended_class: ModelClass::Standard,
                    resolved_model: default_model.to_string(),
                    complexity: "unspecified".into(),
                    needs_reasoning: false,
                    needs_large_context: false,
                    risk: "unspecified".into(),
                    rationale:
                        "Decision provider is unconfigured or disabled; default model selected."
                            .into(),
                    fallback_applied: true,
                    latency_ms: 0,
                };
            }
        };

        let request = Self::build_request(task);
        let resp = match engine.evaluate(request) {
            Ok(r) => r,
            Err(e) => {
                return RoutingResolution {
                    task: task.to_string(),
                    recommended_class: ModelClass::Standard,
                    resolved_model: default_model.to_string(),
                    complexity: "unspecified".into(),
                    needs_reasoning: false,
                    needs_large_context: false,
                    risk: "unspecified".into(),
                    rationale: format!("Decision provider error ({e}); default model selected."),
                    fallback_applied: true,
                    latency_ms: 0,
                };
            }
        };

        if resp.fallback_used {
            return RoutingResolution {
                task: task.to_string(),
                recommended_class: ModelClass::Standard,
                resolved_model: default_model.to_string(),
                complexity: "unspecified".into(),
                needs_reasoning: false,
                needs_large_context: false,
                risk: "unspecified".into(),
                rationale: "Decision provider returned fallback state; default model selected."
                    .into(),
                fallback_applied: true,
                latency_ms: resp.latency_ms,
            };
        }

        // Parse questions from response
        let (complexity, _comp_conf) = extract_choice(&resp, "complexity", "moderate");
        let (needs_reasoning, reasoning_conf) = extract_boolean(&resp, "needs_reasoning");
        let (needs_large_context, _ctx_conf) = extract_boolean(&resp, "needs_large_context");
        let (risk, _risk_conf) = extract_choice(&resp, "risk", "low");

        let reasoning_threshold = self.config.thresholds.reasoning_threshold_pct as f64 / 100.0;

        let (target_class, rationale) = if (needs_reasoning
            && reasoning_conf >= reasoning_threshold)
            || complexity == "complex"
            || risk == "high"
        {
            (
                ModelClass::Reasoning,
                format!(
                    "Task routed to reasoning class (complexity: {complexity}, reasoning: {needs_reasoning} [{:.0}%], risk: {risk})",
                    reasoning_conf * 100.0
                ),
            )
        } else if complexity == "trivial"
            && risk == "low"
            && !needs_reasoning
            && !needs_large_context
        {
            (
                ModelClass::Fast,
                "Task routed to fast class (trivial complexity, low risk, no deep reasoning or large context needed)".into(),
            )
        } else {
            (
                ModelClass::Standard,
                format!("Task routed to standard class (complexity: {complexity}, risk: {risk})",),
            )
        };

        let (resolved_model, fallback_applied) = self
            .config
            .models
            .resolve_with_fallback(target_class, default_model);

        RoutingResolution {
            task: task.to_string(),
            recommended_class: target_class,
            resolved_model,
            complexity,
            needs_reasoning,
            needs_large_context,
            risk,
            rationale,
            fallback_applied,
            latency_ms: resp.latency_ms,
        }
    }
}

fn extract_choice(resp: &DecisionResponse, id: &str, default: &str) -> (String, f64) {
    match resp.get_answer(id) {
        Some(DecisionAnswer::Choice {
            selected,
            confidence,
            ..
        }) => (selected.clone(), *confidence),
        _ => (default.to_string(), 0.0),
    }
}

fn extract_boolean(resp: &DecisionResponse, id: &str) -> (bool, f64) {
    match resp.get_answer(id) {
        Some(DecisionAnswer::Boolean { value, confidence }) => (*value, *confidence),
        _ => (false, 0.0),
    }
}

#[cfg(test)]
#[path = "tests/routing_tests.rs"]
mod tests;
