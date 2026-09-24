//! Jev (TypeSafe AI) HTTP client provider for the Decision Engine.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::decisions::types::{
    build_systemone_questions, parse_systemone_answers, DecisionRequest, DecisionResponse,
    SystemOneWireRequest, SystemOneWireResponse,
};
pub use crate::decisions::types::{
    SystemOneWireRequest as JevWireRequest, SystemOneWireResponse as JevWireResponse,
};
use crate::decisions::{DecisionProvider, HealthStatus};
use crate::error::CeError;

/// Configuration options for the Jev provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JevConfig {
    /// Jev API base endpoint URL (e.g. "https://api.typesafe.ai/v1").
    pub endpoint: String,
    /// Model identifier to evaluate (e.g. "jev-latest").
    pub model: String,
    /// Request timeout in milliseconds.
    pub timeout_ms: u64,
}

impl Default for JevConfig {
    fn default() -> Self {
        Self {
            endpoint: "https://api.typesafe.ai/v1".into(),
            model: "jev-latest".into(),
            timeout_ms: 1000,
        }
    }
}

/// Provider client implementation for Jev / TypeSafe AI.
pub struct JevProvider {
    config: JevConfig,
    api_key: Option<String>,
    client: reqwest::blocking::Client,
}

impl JevProvider {
    /// Creates a new Jev provider instance.
    pub fn new(config: JevConfig, api_key: Option<String>) -> Self {
        let timeout = Duration::from_millis(config.timeout_ms);
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_else(|_| reqwest::blocking::Client::new());

        Self {
            config,
            api_key,
            client,
        }
    }

    pub fn config(&self) -> &JevConfig {
        &self.config
    }

    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some() || crate::decisions::auth::resolve_api_key(None).is_some()
    }

    /// Serializes a domain `DecisionRequest` into the System One wire protocol.
    pub fn build_wire_payload(&self, request: DecisionRequest) -> SystemOneWireRequest {
        let questions = build_systemone_questions(request.questions);
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

    /// Parses a System One wire response into domain `DecisionResponse`.
    pub fn parse_wire_response(
        &self,
        wire: SystemOneWireResponse,
        latency_ms: u64,
    ) -> DecisionResponse {
        let answers = parse_systemone_answers(wire.answers);
        let estimated_cost_usd = wire.estimated_cost_usd.or_else(|| {
            wire.usage.as_ref().and_then(|u| {
                u.get("estimated_cost_usd")
                    .and_then(|c| c.as_f64())
                    .or_else(|| u.get("cost_usd").and_then(|c| c.as_f64()))
            })
        });

        DecisionResponse {
            answers,
            provider: "jev".into(),
            model: wire.model.unwrap_or_else(|| self.config.model.clone()),
            latency_ms: wire.latency_ms.unwrap_or(latency_ms),
            estimated_cost_usd,
            fallback_used: false,
            shadow_mode: false,
        }
    }
}

impl DecisionProvider for JevProvider {
    fn name(&self) -> &'static str {
        "jev"
    }

    fn evaluate(&self, request: DecisionRequest) -> Result<DecisionResponse, CeError> {
        let key_resolved;
        let key = match self.api_key.as_deref() {
            Some(k) => k,
            None => {
                key_resolved = crate::decisions::auth::resolve_api_key(None);
                key_resolved.as_deref().ok_or_else(|| {
                    CeError::Usage(
                        "Jev API key not configured. Set TYPESAFE_API_KEY environment variable or run 'ce-ai decisions auth'."
                            .into(),
                    )
                })?
            }
        };

        let trimmed = self.config.endpoint.trim_end_matches('/');
        let url = if trimmed.ends_with("/v1") {
            format!("{trimmed}/systemone")
        } else {
            format!("{trimmed}/v1/systemone")
        };
        let wire_req = self.build_wire_payload(request);

        let payload = serde_json::to_string(&wire_req)
            .map_err(|e| CeError::Runtime(format!("failed to serialize Jev request: {e}")))?;

        let start = Instant::now();
        let http_resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {key}"))
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    CeError::Network(format!(
                        "Jev API request timed out after {}ms: {e}",
                        self.config.timeout_ms
                    ))
                } else {
                    CeError::Network(format!("failed to connect to Jev API: {e}"))
                }
            })?;

        let latency_ms = start.elapsed().as_millis() as u64;
        let status = http_resp.status();
        let body = http_resp.text().unwrap_or_default();

        if !status.is_success() {
            return Err(CeError::Network(format!(
                "Jev API error (HTTP {}): {}",
                status.as_u16(),
                body
            )));
        }

        let wire_resp: SystemOneWireResponse = serde_json::from_str(&body)
            .map_err(|e| CeError::Network(format!("invalid JSON response from Jev API: {e}")))?;

        Ok(self.parse_wire_response(wire_resp, latency_ms))
    }

    fn check_health(&self) -> Result<HealthStatus, CeError> {
        let key_resolved;
        let key = match self.api_key.as_deref() {
            Some(k) => Some(k),
            None => {
                key_resolved = crate::decisions::auth::resolve_api_key(None);
                key_resolved.as_deref()
            }
        };

        let Some(key) = key else {
            return Ok(HealthStatus {
                available: false,
                latency_ms: 0,
                message: "API key not configured (missing TYPESAFE_API_KEY)".into(),
            });
        };

        let trimmed = self.config.endpoint.trim_end_matches('/');
        let primary_url = if trimmed.ends_with("/v1") {
            format!("{trimmed}/models")
        } else if trimmed.contains("api.typesafe.ai") {
            format!("{trimmed}/v1/models")
        } else {
            format!("{trimmed}/models")
        };
        let fallback_url = if trimmed.ends_with("/v1") {
            format!("{}/health", trimmed.trim_end_matches("/v1"))
        } else {
            format!("{trimmed}/health")
        };

        let start = Instant::now();

        // 1. Try primary models/auth probe
        let first_attempt = self
            .client
            .get(&primary_url)
            .header("Authorization", format!("Bearer {key}"))
            .send();

        match first_attempt {
            Ok(resp) if resp.status().is_success() => {
                let latency_ms = start.elapsed().as_millis() as u64;
                Ok(HealthStatus {
                    available: true,
                    latency_ms,
                    message: format!("Jev API healthy ({}ms)", latency_ms),
                })
            }
            Ok(resp) if resp.status() == reqwest::StatusCode::UNAUTHORIZED => {
                let latency_ms = start.elapsed().as_millis() as u64;
                Ok(HealthStatus {
                    available: false,
                    latency_ms,
                    message: "Jev authentication failed: HTTP 401 Unauthorized (check API key)"
                        .into(),
                })
            }
            Ok(resp) if resp.status() == reqwest::StatusCode::FORBIDDEN => {
                let latency_ms = start.elapsed().as_millis() as u64;
                Ok(HealthStatus {
                    available: false,
                    latency_ms,
                    message: "Jev authentication failed: HTTP 403 Forbidden".into(),
                })
            }
            Ok(resp) if resp.status() == reqwest::StatusCode::NOT_FOUND => {
                // Primary endpoint 404 (e.g. mock server) -> try fallback health endpoint
                match self
                    .client
                    .get(&fallback_url)
                    .header("Authorization", format!("Bearer {key}"))
                    .send()
                {
                    Ok(fb_resp) => {
                        let latency_ms = start.elapsed().as_millis() as u64;
                        if fb_resp.status().is_success() {
                            Ok(HealthStatus {
                                available: true,
                                latency_ms,
                                message: format!("Jev API healthy ({}ms)", latency_ms),
                            })
                        } else {
                            Ok(HealthStatus {
                                available: false,
                                latency_ms,
                                message: format!(
                                    "Jev health check returned HTTP {}",
                                    fb_resp.status()
                                ),
                            })
                        }
                    }
                    Err(e) => Ok(HealthStatus {
                        available: false,
                        latency_ms: start.elapsed().as_millis() as u64,
                        message: format!("Jev connection error: {e}"),
                    }),
                }
            }
            Ok(resp) => {
                let latency_ms = start.elapsed().as_millis() as u64;
                Ok(HealthStatus {
                    available: false,
                    latency_ms,
                    message: format!("Jev health check returned HTTP {}", resp.status()),
                })
            }
            Err(e) => {
                // If primary request failed, also try fallback URL before giving up
                match self
                    .client
                    .get(&fallback_url)
                    .header("Authorization", format!("Bearer {key}"))
                    .send()
                {
                    Ok(fb_resp) if fb_resp.status().is_success() => {
                        let latency_ms = start.elapsed().as_millis() as u64;
                        Ok(HealthStatus {
                            available: true,
                            latency_ms,
                            message: format!("Jev API healthy ({}ms)", latency_ms),
                        })
                    }
                    _ => Ok(HealthStatus {
                        available: false,
                        latency_ms: start.elapsed().as_millis() as u64,
                        message: format!("Jev connection error: {e}"),
                    }),
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/jev_tests.rs"]
mod tests;
