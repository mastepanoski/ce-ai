//! Domain types, question primitives, and response models for the Decision Engine.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::CeError;

/// Operational execution mode for the Decision Engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DecisionMode {
    /// Decision Engine is completely disabled. Zero evaluations or network calls.
    #[default]
    Off,
    /// Shadow mode: evaluates decisions in background and logs telemetry, but results
    /// do not influence deterministic execution policies.
    Shadow,
    /// Active mode: probabilistic decisions actively inform deterministic policies.
    Active,
}

impl DecisionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionMode::Off => "off",
            DecisionMode::Shadow => "shadow",
            DecisionMode::Active => "active",
        }
    }

    pub fn parse(s: &str) -> Result<Self, CeError> {
        let clean = s.trim().to_lowercase();
        match clean.as_str() {
            "off" | "disabled" => Ok(DecisionMode::Off),
            "shadow" => Ok(DecisionMode::Shadow),
            "active" | "on" | "enabled" => Ok(DecisionMode::Active),
            _ => Err(CeError::Usage(format!(
                "invalid decision mode '{s}'. Valid modes: off, shadow, active"
            ))),
        }
    }
}

impl fmt::Display for DecisionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Context passed into a decision request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DecisionContext {
    pub task_description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_stage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

impl DecisionContext {
    pub fn new(task_description: impl Into<String>) -> Self {
        Self {
            task_description: task_description.into(),
            workflow_stage: None,
            execution_mode: None,
            metadata: BTreeMap::new(),
        }
    }

    pub fn with_stage(mut self, stage: impl Into<String>) -> Self {
        self.workflow_stage = Some(stage.into());
        self
    }

    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        self.execution_mode = Some(mode.into());
        self
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Typed question primitives supported by the Decision Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DecisionQuestion {
    /// Binary boolean evaluation.
    Boolean { id: String, question: String },
    /// Categorical choice among a predefined set of options.
    Choice {
        id: String,
        question: String,
        options: Vec<String>,
    },
    /// Continuous score within bounded range.
    Score {
        id: String,
        question: String,
        min: f64,
        max: f64,
    },
}

impl DecisionQuestion {
    pub fn id(&self) -> &str {
        match self {
            DecisionQuestion::Boolean { id, .. } => id,
            DecisionQuestion::Choice { id, .. } => id,
            DecisionQuestion::Score { id, .. } => id,
        }
    }

    pub fn question(&self) -> &str {
        match self {
            DecisionQuestion::Boolean { question, .. } => question,
            DecisionQuestion::Choice { question, .. } => question,
            DecisionQuestion::Score { question, .. } => question,
        }
    }

    pub fn boolean(id: impl Into<String>, question: impl Into<String>) -> Self {
        DecisionQuestion::Boolean {
            id: id.into(),
            question: question.into(),
        }
    }

    pub fn choice(
        id: impl Into<String>,
        question: impl Into<String>,
        options: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        DecisionQuestion::Choice {
            id: id.into(),
            question: question.into(),
            options: options.into_iter().map(Into::into).collect(),
        }
    }

    pub fn score(id: impl Into<String>, question: impl Into<String>, min: f64, max: f64) -> Self {
        DecisionQuestion::Score {
            id: id.into(),
            question: question.into(),
            min,
            max,
        }
    }
}

/// Batch request containing context and a list of questions to be evaluated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DecisionRequest {
    pub context: DecisionContext,
    pub questions: Vec<DecisionQuestion>,
}

impl DecisionRequest {
    pub fn new(context: DecisionContext) -> Self {
        Self {
            context,
            questions: Vec::new(),
        }
    }

    pub fn with_question(mut self, question: DecisionQuestion) -> Self {
        self.questions.push(question);
        self
    }
}

/// Typed answer primitives returned by the Decision Engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DecisionAnswer {
    Boolean {
        value: bool,
        confidence: f64,
    },
    Choice {
        selected: String,
        confidence: f64,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        probabilities: BTreeMap<String, f64>,
    },
    Score {
        score: f64,
        confidence: f64,
    },
}

impl DecisionAnswer {
    pub fn confidence(&self) -> f64 {
        match self {
            DecisionAnswer::Boolean { confidence, .. } => *confidence,
            DecisionAnswer::Choice { confidence, .. } => *confidence,
            DecisionAnswer::Score { confidence, .. } => *confidence,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            DecisionAnswer::Boolean { value, .. } => Some(*value),
            _ => None,
        }
    }

    pub fn as_choice(&self) -> Option<&str> {
        match self {
            DecisionAnswer::Choice { selected, .. } => Some(selected),
            _ => None,
        }
    }

    pub fn as_score(&self) -> Option<f64> {
        match self {
            DecisionAnswer::Score { score, .. } => Some(*score),
            _ => None,
        }
    }
}

/// Response returned from a decision provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionResponse {
    pub answers: BTreeMap<String, DecisionAnswer>,
    pub provider: String,
    pub model: String,
    pub latency_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cost_usd: Option<f64>,
    #[serde(default)]
    pub fallback_used: bool,
    #[serde(default)]
    pub shadow_mode: bool,
}

impl DecisionResponse {
    pub fn new(provider: impl Into<String>, model: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            answers: BTreeMap::new(),
            provider: provider.into(),
            model: model.into(),
            latency_ms,
            estimated_cost_usd: None,
            fallback_used: false,
            shadow_mode: false,
        }
    }

    pub fn with_answer(mut self, question_id: impl Into<String>, answer: DecisionAnswer) -> Self {
        self.answers.insert(question_id.into(), answer);
        self
    }

    pub fn get_answer(&self, question_id: &str) -> Option<&DecisionAnswer> {
        self.answers.get(question_id)
    }
}

#[cfg(test)]
#[path = "tests/types_tests.rs"]
mod tests;
