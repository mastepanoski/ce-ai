//! Deterministic Mock Decision Provider for offline unit and integration testing.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::decisions::types::{
    DecisionAnswer, DecisionQuestion, DecisionRequest, DecisionResponse,
};
use crate::decisions::DecisionProvider;
use crate::error::CeError;

/// Health status report returned by decision providers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    pub available: bool,
    pub latency_ms: u64,
    pub message: String,
}

/// A configurable mock decision provider for testing.
#[derive(Debug, Clone, Default)]
pub struct MockDecisionProvider {
    canned_answers: Arc<Mutex<BTreeMap<String, DecisionAnswer>>>,
    should_fail: Arc<Mutex<Option<String>>>,
    simulated_latency_ms: Arc<Mutex<u64>>,
}

impl MockDecisionProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_canned_answer(
        self,
        question_id: impl Into<String>,
        answer: DecisionAnswer,
    ) -> Self {
        if let Ok(mut map) = self.canned_answers.lock() {
            map.insert(question_id.into(), answer);
        }
        self
    }

    pub fn set_failure(&self, error_msg: Option<String>) {
        if let Ok(mut fail) = self.should_fail.lock() {
            *fail = error_msg;
        }
    }

    pub fn set_latency_ms(&self, latency_ms: u64) {
        if let Ok(mut lat) = self.simulated_latency_ms.lock() {
            *lat = latency_ms;
        }
    }
}

impl DecisionProvider for MockDecisionProvider {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn evaluate(&self, request: DecisionRequest) -> Result<DecisionResponse, CeError> {
        if let Ok(fail) = self.should_fail.lock() {
            if let Some(msg) = fail.as_ref() {
                return Err(CeError::Network(msg.clone()));
            }
        }

        let latency = match self.simulated_latency_ms.lock() {
            Ok(guard) => *guard,
            Err(_) => 1,
        };

        let mut resp = DecisionResponse::new("mock", "mock-v1", latency);

        let canned = match self.canned_answers.lock() {
            Ok(guard) => guard.clone(),
            Err(_) => BTreeMap::new(),
        };

        for question in request.questions {
            let qid = question.id().to_string();
            if let Some(ans) = canned.get(&qid) {
                resp.answers.insert(qid.to_string(), ans.clone());
            } else {
                let default_ans = match question {
                    DecisionQuestion::Boolean { .. } => DecisionAnswer::Boolean {
                        value: false,
                        confidence: 1.0,
                    },
                    DecisionQuestion::Choice { options, .. } => DecisionAnswer::Choice {
                        selected: options.first().cloned().unwrap_or_default(),
                        confidence: 1.0,
                        probabilities: BTreeMap::new(),
                    },
                    DecisionQuestion::Score { min, .. } => DecisionAnswer::Score {
                        score: min,
                        confidence: 1.0,
                    },
                };
                resp.answers.insert(qid.to_string(), default_ans);
            }
        }

        Ok(resp)
    }

    fn check_health(&self) -> Result<HealthStatus, CeError> {
        if let Ok(fail) = self.should_fail.lock() {
            if let Some(msg) = fail.as_ref() {
                return Ok(HealthStatus {
                    available: false,
                    latency_ms: 0,
                    message: format!("Mock failure: {msg}"),
                });
            }
        }

        let latency = match self.simulated_latency_ms.lock() {
            Ok(guard) => *guard,
            Err(_) => 1,
        };

        Ok(HealthStatus {
            available: true,
            latency_ms: latency,
            message: "Mock provider healthy".into(),
        })
    }
}

#[cfg(test)]
#[path = "tests/mock_tests.rs"]
mod tests;
