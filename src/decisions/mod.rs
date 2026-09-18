//! Pluggable Decision Engine: fast, structured, probabilistic micro-decisions (System 1)
//! decoupled from deterministic orchestration (System 2).

pub mod auth;
pub mod budget;
pub mod jev;
pub mod mock;
pub mod routing;
pub mod skill_routing;
pub mod types;

pub use auth::{
    default_credentials_path, mask_api_key, resolve_api_key, save_api_key, CredentialsFile,
};
pub use budget::{BudgetConfig, BudgetTracker, CircuitState, FallbackReason, MonthlyLedger};
pub use jev::{JevConfig, JevProvider, JevWireRequest, JevWireResponse};
pub use mock::{HealthStatus, MockDecisionProvider};
pub use routing::{
    ModelClass, ModelClassCatalog, ModelRouter, ModelRoutingConfig, RoutingResolution,
    RoutingThresholds,
};
pub use skill_routing::{
    SkillClassification, SkillRouter, SkillRoutingConfig, SkillRoutingResult, RECOGNIZED_CATEGORIES,
};
pub use types::{
    DecisionAnswer, DecisionContext, DecisionMode, DecisionQuestion, DecisionRequest,
    DecisionResponse,
};

use crate::error::CeError;

/// Trait implemented by all decision providers (e.g. Jev, Mock, Local Rules).
pub trait DecisionProvider: Send + Sync {
    /// Canonical name of the provider (e.g. "jev", "mock", "rules").
    fn name(&self) -> &'static str;

    /// Evaluates a structured request against the provider.
    fn evaluate(&self, request: DecisionRequest) -> Result<DecisionResponse, CeError>;

    /// Health check to verify credentials and provider availability.
    fn check_health(&self) -> Result<HealthStatus, CeError>;
}

/// Orchestrator / Dispatcher for the Decision Engine.
pub struct DecisionEngine {
    provider: Option<Box<dyn DecisionProvider>>,
    mode: DecisionMode,
}

impl DecisionEngine {
    pub fn new(provider: Option<Box<dyn DecisionProvider>>, mode: DecisionMode) -> Self {
        Self { provider, mode }
    }

    /// Creates an engine with a mock provider for hermetic testing.
    pub fn with_mock(mock: MockDecisionProvider, mode: DecisionMode) -> Self {
        Self {
            provider: Some(Box::new(mock)),
            mode,
        }
    }

    /// Creates an engine configured according to state DecisionsConfig.
    pub fn from_config(config: &crate::state::state::DecisionsConfig) -> Self {
        if !config.enabled || config.mode == DecisionMode::Off {
            return Self::new(None, DecisionMode::Off);
        }

        let provider: Option<Box<dyn DecisionProvider>> = if config.provider == "mock" {
            Some(Box::new(mock::MockDecisionProvider::new()))
        } else {
            Some(Box::new(jev::JevProvider::new(config.jev.clone(), None)))
        };

        Self::new(provider, config.mode)
    }

    pub fn mode(&self) -> DecisionMode {
        self.mode
    }

    pub fn is_enabled(&self) -> bool {
        self.mode != DecisionMode::Off && self.provider.is_some()
    }

    /// Evaluates a batch request according to the active execution mode.
    pub fn evaluate(&self, request: DecisionRequest) -> Result<DecisionResponse, CeError> {
        match self.mode {
            DecisionMode::Off => {
                let mut resp = DecisionResponse::new("none", "disabled", 0);
                resp.fallback_used = true;
                Ok(resp)
            }
            DecisionMode::Shadow => {
                match &self.provider {
                    Some(provider) => {
                        match provider.evaluate(request) {
                            Ok(mut resp) => {
                                // Shadow mode records decision telemetry but flags fallback_used = true
                                // so deterministic policies know not to rely on the answer for enforcement.
                                resp.fallback_used = true;
                                Ok(resp)
                            }
                            Err(_) => {
                                let mut resp =
                                    DecisionResponse::new(provider.name(), "shadow-error", 0);
                                resp.fallback_used = true;
                                Ok(resp)
                            }
                        }
                    }
                    None => {
                        let mut resp = DecisionResponse::new("none", "unconfigured", 0);
                        resp.fallback_used = true;
                        Ok(resp)
                    }
                }
            }
            DecisionMode::Active => match &self.provider {
                Some(provider) => provider.evaluate(request),
                None => {
                    let mut resp = DecisionResponse::new("none", "unconfigured", 0);
                    resp.fallback_used = true;
                    Ok(resp)
                }
            },
        }
    }

    /// Probes provider health.
    pub fn check_health(&self) -> Result<HealthStatus, CeError> {
        match &self.provider {
            Some(provider) => provider.check_health(),
            None => Ok(HealthStatus {
                available: false,
                latency_ms: 0,
                message: "Decision provider not configured".into(),
            }),
        }
    }
}

#[cfg(test)]
#[path = "tests/mod_tests.rs"]
mod tests;
