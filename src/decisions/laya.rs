//! Laya MLX local decision provider for Apple Silicon.
//!
//! Connects to a locally running Laya daemon or socket delivering sub-15ms
//! typed decisions on Apple Silicon using native MLX.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::decisions::kev::{SystemOneWireQuestion, SystemOneWireResponse};
use crate::decisions::types::{
    DecisionAnswer, DecisionQuestion, DecisionRequest, DecisionResponse,
};
use crate::decisions::{DecisionProvider, HealthStatus};
use crate::error::CeError;

/// Configuration options for the Laya MLX local provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayaConfig {
    /// Laya daemon endpoint URL (e.g. "http://127.0.0.1:8080/v1").
    pub endpoint: String,
    /// Optional Unix domain socket path for sub-millisecond local IPC.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socket_path: Option<String>,
    /// Model checkpoint identifier (e.g. "aac6fef/laya-mlx", "convaiinnovations/laya").
    pub model: String,
    /// Request timeout in milliseconds (default 250ms).
    pub timeout_ms: u64,
}

impl Default for LayaConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:8080/v1".into(),
            socket_path: None,
            model: "aac6fef/laya-mlx".into(),
            timeout_ms: 250,
        }
    }
}

/// Provider client implementation for Laya MLX local decision engine.
pub struct LayaMlxProvider {
    config: LayaConfig,
    client: reqwest::blocking::Client,
}

impl LayaMlxProvider {
    /// Creates a new Laya MLX provider instance.
    pub fn new(config: LayaConfig) -> Self {
        let timeout = Duration::from_millis(config.timeout_ms);
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self { config, client }
    }

    pub fn config(&self) -> &LayaConfig {
        &self.config
    }

    /// Checks if current runtime platform is Apple Silicon macOS.
    pub fn is_apple_silicon() -> bool {
        cfg!(all(target_os = "macos", target_arch = "aarch64"))
    }

    /// Translates domain `DecisionRequest` into System One wire payload for Laya.
    pub fn build_wire_payload(&self, request: DecisionRequest) -> serde_json::Value {
        let mut questions = BTreeMap::new();

        for q in request.questions {
            match q {
                DecisionQuestion::Boolean { id, question } => {
                    questions.insert(
                        id,
                        SystemOneWireQuestion {
                            question_type: "noul".into(),
                            instructions: Some(question),
                            criteria: None,
                        },
                    );
                }
                DecisionQuestion::Choice {
                    id,
                    question,
                    options,
                } => {
                    let mut criteria_map = serde_json::Map::new();
                    for opt in options {
                        criteria_map.insert(opt, serde_json::Value::Null);
                    }
                    questions.insert(
                        id,
                        SystemOneWireQuestion {
                            question_type: "choice".into(),
                            instructions: Some(question),
                            criteria: Some(serde_json::Value::Object(criteria_map)),
                        },
                    );
                }
                DecisionQuestion::Score {
                    id,
                    question,
                    min: _,
                    max: _,
                } => {
                    questions.insert(
                        id,
                        SystemOneWireQuestion {
                            question_type: "score".into(),
                            instructions: Some(question),
                            criteria: Some(serde_json::json!(["low", "medium", "high"])),
                        },
                    );
                }
            }
        }

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

        serde_json::json!({
            "state": state_obj,
            "model": self.config.model,
            "questions": questions,
        })
    }

    /// Parses Laya wire response into domain `DecisionResponse`.
    pub fn parse_wire_response(
        &self,
        wire: SystemOneWireResponse,
        measured_latency_ms: u64,
    ) -> DecisionResponse {
        let mut answers = BTreeMap::new();

        for (id, ans) in wire.answers {
            if let Some(choice) = ans.choice {
                let conf = ans.confidence.unwrap_or(1.0);
                let probs = ans.probabilities.unwrap_or_default();
                answers.insert(
                    id,
                    DecisionAnswer::Choice {
                        selected: choice,
                        confidence: conf,
                        probabilities: probs,
                    },
                );
            } else if let Some(noul_val) = ans.noul {
                let value = noul_val >= 0.5;
                let conf = ans
                    .confidence
                    .unwrap_or_else(|| (noul_val - 0.5).abs() * 2.0);
                answers.insert(
                    id,
                    DecisionAnswer::Boolean {
                        value,
                        confidence: conf,
                    },
                );
            } else if let Some(score_val) = ans.score {
                let conf = ans.confidence.unwrap_or(1.0);
                answers.insert(
                    id,
                    DecisionAnswer::Score {
                        score: score_val,
                        confidence: conf,
                    },
                );
            }
        }

        DecisionResponse {
            answers,
            provider: "laya-mlx".into(),
            model: wire.model.unwrap_or_else(|| self.config.model.clone()),
            latency_ms: wire.latency_ms.unwrap_or(measured_latency_ms),
            estimated_cost_usd: Some(0.0), // Local MLX execution is free
            fallback_used: false,
            shadow_mode: false,
        }
    }
}

impl DecisionProvider for LayaMlxProvider {
    fn name(&self) -> &'static str {
        "laya-mlx"
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
            .map_err(|e| CeError::Runtime(format!("failed to serialize Laya request: {e}")))?;

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
                        "Laya MLX request timed out after {}ms: {e}",
                        self.config.timeout_ms
                    ))
                } else {
                    CeError::Network(format!(
                        "failed to connect to Laya MLX daemon at {url}: {e}. Ensure Laya MLX is running."
                    ))
                }
            })?;

        let latency_ms = start.elapsed().as_millis() as u64;

        if !http_resp.status().is_success() {
            let status = http_resp.status();
            let body = http_resp.text().unwrap_or_default();
            return Err(CeError::Network(format!(
                "Laya MLX daemon returned HTTP error {status}: {body}"
            )));
        }

        let resp_body = http_resp
            .text()
            .map_err(|e| CeError::Network(format!("failed to read Laya response body: {e}")))?;

        let wire_resp: SystemOneWireResponse = serde_json::from_str(&resp_body)
            .map_err(|e| CeError::Runtime(format!("failed to parse Laya response JSON: {e}")))?;

        Ok(self.parse_wire_response(wire_resp, latency_ms))
    }

    fn check_health(&self) -> Result<HealthStatus, CeError> {
        if !Self::is_apple_silicon() {
            return Ok(HealthStatus {
                available: false,
                latency_ms: 0,
                message: "laya-mlx requires macOS Apple Silicon (aarch64). Current platform is not supported.".into(),
            });
        }

        let base_url = self.config.endpoint.trim_end_matches('/');
        let health_url = if base_url.ends_with("/v1") {
            format!("{base_url}/models")
        } else {
            format!("{base_url}/v1/models")
        };

        let start = Instant::now();
        let resp = self.client.get(&health_url).send();

        match resp {
            Ok(r) if r.status().is_success() => {
                let latency_ms = start.elapsed().as_millis() as u64;
                Ok(HealthStatus {
                    available: true,
                    latency_ms,
                    message: format!("Laya MLX daemon available (model: {})", self.config.model),
                })
            }
            Ok(r) => Ok(HealthStatus {
                available: false,
                latency_ms: 0,
                message: format!("Laya daemon returned HTTP status {}", r.status()),
            }),
            Err(e) => Ok(HealthStatus {
                available: false,
                latency_ms: 0,
                message: format!("could not connect to Laya daemon at {health_url}: {e}"),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decisions::kev::SystemOneWireAnswer;
    use crate::decisions::types::DecisionContext;

    #[test]
    fn test_laya_config_default() {
        let cfg = LayaConfig::default();
        assert_eq!(cfg.endpoint, "http://127.0.0.1:8080/v1");
        assert_eq!(cfg.model, "aac6fef/laya-mlx");
        assert_eq!(cfg.timeout_ms, 250);
        assert!(cfg.socket_path.is_none());
    }

    #[test]
    fn test_build_wire_payload() {
        let provider = LayaMlxProvider::new(LayaConfig::default());
        let ctx = DecisionContext::new("Evaluate security risk")
            .with_stage("plan")
            .with_mode("active");

        let req = DecisionRequest::new(ctx)
            .with_question(DecisionQuestion::boolean("is_safe", "Is it read-only?"))
            .with_question(DecisionQuestion::choice(
                "tier",
                "Risk tier",
                vec!["low", "high"],
            ));

        let wire = provider.build_wire_payload(req);
        let questions = wire.get("questions").unwrap().as_object().unwrap();
        assert_eq!(questions.len(), 2);
        assert!(questions.contains_key("is_safe"));
        assert!(questions.contains_key("tier"));
    }

    #[test]
    fn test_parse_wire_response() {
        let provider = LayaMlxProvider::new(LayaConfig::default());
        let mut answers = BTreeMap::new();

        answers.insert(
            "decision".into(),
            SystemOneWireAnswer {
                answer_type: Some("choice".into()),
                choice: Some("allow".into()),
                confidence: Some(0.95),
                ..Default::default()
            },
        );

        let wire_resp = SystemOneWireResponse {
            model: Some("laya-decision-base".into()),
            answers,
            latency_ms: Some(11),
            usage: None,
        };

        let resp = provider.parse_wire_response(wire_resp, 12);
        assert_eq!(resp.provider, "laya-mlx");
        assert_eq!(resp.model, "laya-decision-base");
        assert_eq!(resp.latency_ms, 11);
        assert_eq!(resp.estimated_cost_usd, Some(0.0));

        let ans = resp.answers.get("decision").unwrap();
        match ans {
            DecisionAnswer::Choice {
                selected,
                confidence,
                ..
            } => {
                assert_eq!(selected, "allow");
                assert_eq!(*confidence, 0.95);
            }
            _ => panic!("expected choice answer"),
        }
    }
}
