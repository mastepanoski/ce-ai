//! Kev local decision provider implementing TypeSafe System One wire protocol.
//!
//! Connects to a locally running Kev server (e.g. `python -m kev.serve --run jaredpalmer/kev-4b --port 8009`)
//! or remote self-hosted instance.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::decisions::types::{
    build_systemone_questions, parse_systemone_answers, DecisionRequest, DecisionResponse,
};
pub use crate::decisions::types::{
    SystemOneWireAnswer, SystemOneWireQuestion, SystemOneWireRequest, SystemOneWireResponse,
};
use crate::decisions::{DecisionProvider, HealthStatus};
use crate::error::CeError;

/// Configuration options for the Kev local provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KevConfig {
    /// Kev server base endpoint URL (e.g. "http://127.0.0.1:8009/v1").
    pub endpoint: String,
    /// Model identifier to evaluate (e.g. "kev-latest", "jaredpalmer/kev-4b").
    pub model: String,
    /// Request timeout in milliseconds.
    pub timeout_ms: u64,
}

impl Default for KevConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:8009/v1".into(),
            model: "kev-latest".into(),
            timeout_ms: 2000,
        }
    }
}

/// Provider client implementation for Kev local decision engine.
pub struct KevProvider {
    config: KevConfig,
    client: reqwest::blocking::Client,
}

impl KevProvider {
    /// Creates a new Kev provider instance.
    pub fn new(config: KevConfig) -> Self {
        let timeout = Duration::from_millis(config.timeout_ms);
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self { config, client }
    }

    pub fn config(&self) -> &KevConfig {
        &self.config
    }

    /// Translates domain `DecisionRequest` into System One wire payload.
    pub fn build_wire_payload(&self, request: DecisionRequest) -> SystemOneWireRequest {
        let questions = build_systemone_questions(request.questions);

        // Build state: structured metadata or plain task description
        let mut state_obj = serde_json::Map::new();
        state_obj.insert(
            "task".into(),
            serde_json::Value::String(request.context.task_description),
        );
        if let Some(stage) = request.context.workflow_stage {
            state_obj.insert("stage".into(), serde_json::Value::String(stage));
        }
        if let Some(mode) = request.context.execution_mode {
            state_obj.insert("mode".into(), serde_json::Value::String(mode));
        }
        if !request.context.metadata.is_empty() {
            let mut meta_map = serde_json::Map::new();
            for (k, v) in request.context.metadata {
                meta_map.insert(k, serde_json::Value::String(v));
            }
            state_obj.insert("metadata".into(), serde_json::Value::Object(meta_map));
        }

        SystemOneWireRequest {
            state: serde_json::Value::Object(state_obj),
            model: self.config.model.clone(),
            questions,
        }
    }

    /// Parses a System One response into domain `DecisionResponse`.
    pub fn parse_wire_response(
        &self,
        wire: SystemOneWireResponse,
        measured_latency_ms: u64,
    ) -> DecisionResponse {
        let answers = parse_systemone_answers(wire.answers);

        DecisionResponse {
            answers,
            provider: "kev".into(),
            model: wire.model.unwrap_or_else(|| self.config.model.clone()),
            latency_ms: wire.latency_ms.unwrap_or(measured_latency_ms),
            estimated_cost_usd: Some(0.0), // Local execution has zero token cost
            fallback_used: false,
            shadow_mode: false,
        }
    }
}

impl DecisionProvider for KevProvider {
    fn name(&self) -> &'static str {
        "kev"
    }

    fn evaluate(&self, request: DecisionRequest) -> Result<DecisionResponse, CeError> {
        let base_url = self.config.endpoint.trim_end_matches('/');
        let url = if base_url.ends_with("/v1") {
            format!("{base_url}/systemone")
        } else {
            format!("{base_url}/v1/systemone")
        };

        let wire_req = self.build_wire_payload(request);
        let payload = serde_json::to_string(&wire_req)
            .map_err(|e| CeError::Runtime(format!("failed to serialize Kev request: {e}")))?;

        let start = Instant::now();
        let http_resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    CeError::Network(format!(
                        "Kev server request timed out after {}ms: {e}",
                        self.config.timeout_ms
                    ))
                } else {
                    CeError::Network(format!(
                        "failed to connect to Kev server at {url}: {e}. Ensure Kev server is running with 'python -m kev.serve'."
                    ))
                }
            })?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !http_resp.status().is_success() {
            let status = http_resp.status();
            let body = http_resp.text().unwrap_or_default();
            return Err(CeError::Network(format!(
                "Kev server returned HTTP error {status}: {body}"
            )));
        }

        let resp_body = http_resp
            .text()
            .map_err(|e| CeError::Network(format!("failed to read Kev response body: {e}")))?;

        let wire_resp: SystemOneWireResponse = serde_json::from_str(&resp_body)
            .map_err(|e| CeError::Runtime(format!("failed to parse Kev response JSON: {e}")))?;

        Ok(self.parse_wire_response(wire_resp, latency_ms))
    }

    fn check_health(&self) -> Result<HealthStatus, CeError> {
        let base_url = self.config.endpoint.trim_end_matches('/');
        let models_url = if base_url.ends_with("/v1") {
            format!("{base_url}/models")
        } else {
            format!("{base_url}/v1/models")
        };

        let start = Instant::now();
        let resp = self.client.get(&models_url).send();

        match resp {
            Ok(r) if r.status().is_success() => {
                let latency_ms = start.elapsed().as_millis() as u64;
                Ok(HealthStatus {
                    available: true,
                    latency_ms,
                    message: format!("Kev server available (model: {})", self.config.model),
                })
            }
            Ok(r) => Ok(HealthStatus {
                available: false,
                latency_ms: 0,
                message: format!("Kev server returned HTTP status {}", r.status()),
            }),
            Err(e) => Ok(HealthStatus {
                available: false,
                latency_ms: 0,
                message: format!("could not connect to Kev server at {models_url}: {e}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decisions::types::{DecisionAnswer, DecisionContext, DecisionQuestion};
    use std::collections::BTreeMap;

    #[test]
    fn test_kev_config_default() {
        let cfg = KevConfig::default();
        assert_eq!(cfg.endpoint, "http://127.0.0.1:8009/v1");
        assert_eq!(cfg.model, "kev-latest");
        assert_eq!(cfg.timeout_ms, 2000);
    }

    #[test]
    fn test_build_wire_payload_questions() {
        let provider = KevProvider::new(KevConfig::default());
        let ctx = DecisionContext::new("Test prompt")
            .with_stage("work")
            .with_mode("active");

        let req = DecisionRequest::new(ctx)
            .with_question(DecisionQuestion::boolean("q_bool", "Is this safe?"))
            .with_question(DecisionQuestion::choice(
                "q_choice",
                "Select category",
                vec!["cat_a", "cat_b"],
            ))
            .with_question(DecisionQuestion::score("q_score", "Rate quality", 0.0, 1.0));

        let wire = provider.build_wire_payload(req);
        assert_eq!(wire.model, "kev-latest");
        assert_eq!(wire.questions.len(), 3);

        let q_bool = wire.questions.get("q_bool").unwrap();
        assert_eq!(q_bool.question_type, "noul");
        assert_eq!(q_bool.instructions.as_deref(), Some("Is this safe?"));

        let q_choice = wire.questions.get("q_choice").unwrap();
        assert_eq!(q_choice.question_type, "choice");
        let criteria = q_choice.criteria.as_ref().unwrap().as_object().unwrap();
        assert!(criteria.contains_key("cat_a"));
        assert!(criteria.contains_key("cat_b"));

        let q_score = wire.questions.get("q_score").unwrap();
        assert_eq!(q_score.question_type, "score");
    }

    #[test]
    fn test_parse_wire_response() {
        let provider = KevProvider::new(KevConfig::default());
        let mut answers = BTreeMap::new();

        let mut probs = BTreeMap::new();
        probs.insert("fast".into(), 0.85);
        probs.insert("slow".into(), 0.15);

        answers.insert(
            "route".into(),
            SystemOneWireAnswer {
                answer_type: Some("choice".into()),
                choice: Some("fast".into()),
                confidence: Some(0.85),
                probabilities: Some(probs),
                ..Default::default()
            },
        );

        answers.insert(
            "safe".into(),
            SystemOneWireAnswer {
                answer_type: Some("noul".into()),
                noul: Some(0.92),
                confidence: Some(0.84),
                ..Default::default()
            },
        );

        answers.insert(
            "urgency".into(),
            SystemOneWireAnswer {
                answer_type: Some("score".into()),
                score: Some(2.0),
                confidence: Some(0.75),
                ..Default::default()
            },
        );

        let wire_resp = SystemOneWireResponse {
            model: Some("kev-4b".into()),
            answers,
            latency_ms: Some(112),
            usage: None,
            ..Default::default()
        };

        let resp = provider.parse_wire_response(wire_resp, 115);
        assert_eq!(resp.provider, "kev");
        assert_eq!(resp.model, "kev-4b");
        assert_eq!(resp.latency_ms, 112);
        assert_eq!(resp.estimated_cost_usd, Some(0.0));

        let ans_route = resp.answers.get("route").unwrap();
        match ans_route {
            DecisionAnswer::Choice {
                selected,
                confidence,
                probabilities,
            } => {
                assert_eq!(selected, "fast");
                assert_eq!(*confidence, 0.85);
                assert_eq!(probabilities.get("fast"), Some(&0.85));
            }
            _ => panic!("expected Choice answer"),
        }

        let ans_safe = resp.answers.get("safe").unwrap();
        match ans_safe {
            DecisionAnswer::Boolean { value, confidence } => {
                assert!(*value);
                assert_eq!(*confidence, 0.84);
            }
            _ => panic!("expected Boolean answer"),
        }

        let ans_urgency = resp.answers.get("urgency").unwrap();
        match ans_urgency {
            DecisionAnswer::Score { score, confidence } => {
                assert_eq!(*score, 2.0);
                assert_eq!(*confidence, 0.75);
            }
            _ => panic!("expected Score answer"),
        }
    }
}
