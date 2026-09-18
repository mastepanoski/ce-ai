# Design: Pluggable Decision Engine & Jev Provider Architecture

## Overview

The Pluggable Decision Engine provides `ce-ai` with a fast, structured, probabilistic decision layer (System 1). It is provider-agnostic, featuring Jev (TypeSafe AI) as its primary external provider, backed by deterministic mock implementations, credential resolution, budget monitoring, and developer onboarding ergonomics.

---

## Module Layout

```text
src/
├── decisions/
│   ├── mod.rs          # DecisionProvider trait, DecisionEngine dispatcher, ProviderKind
│   ├── types.rs        # DecisionRequest, DecisionResponse, DecisionQuestion, DecisionAnswer
│   ├── jev.rs          # Jev (TypeSafe AI) HTTP client implementation
│   ├── budget.rs       # BudgetTracker, CircuitBreaker, and monthly usage ledger
│   ├── auth.rs         # Credential resolution (env vars + credentials.toml)
│   └── mock.rs         # Deterministic mock provider for offline testing
├── commands/
│   ├── decisions.rs    # CLI subcommands (status, auth, setup, test)
│   └── doctor.rs       # Integration with probe_decision_engine
```

---

## Core Domain Types & Traits (`src/decisions/types.rs`)

```rust
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DecisionMode {
    #[default]
    Off,
    Shadow,
    Active,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub task_description: String,
    pub workflow_stage: Option<String>,
    pub execution_mode: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum DecisionQuestion {
    Boolean {
        id: String,
        question: String,
    },
    Choice {
        id: String,
        question: String,
        options: Vec<String>,
    },
    Score {
        id: String,
        question: String,
        min: f64,
        max: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRequest {
    pub context: DecisionContext,
    pub questions: Vec<DecisionQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum DecisionAnswer {
    Boolean {
        value: bool,
        confidence: f64,
    },
    Choice {
        selected: String,
        confidence: f64,
        probabilities: BTreeMap<String, f64>,
    },
    Score {
        score: f64,
        confidence: f64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResponse {
    pub answers: BTreeMap<String, DecisionAnswer>,
    pub provider: String,
    pub model: String,
    pub latency_ms: u64,
    pub estimated_cost_usd: Option<f64>,
    pub fallback_used: bool,
}
```

---

## Provider Abstraction (`src/decisions/mod.rs`)

```rust
use crate::error::CeError;
use crate::decisions::types::{DecisionRequest, DecisionResponse};

pub trait DecisionProvider: Send + Sync {
    /// Friendly name of the provider (e.g. "jev", "mock", "rules")
    fn name(&self) -> &'static str;

    /// Evaluates a structured request against the provider
    fn evaluate(&self, request: DecisionRequest) -> Result<DecisionResponse, CeError>;

    /// Health check to verify credentials and API availability
    fn check_health(&self) -> Result<HealthStatus, CeError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub available: bool,
    pub latency_ms: u64,
    pub message: String,
}
```

---

## Budget Ceilings & Circuit Breaker (`src/decisions/budget.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    pub max_monthly_usd: f64,
    pub max_session_requests: u32,
    pub timeout_ms: u64,
    pub max_consecutive_failures: u32,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_monthly_usd: 5.0,
            max_session_requests: 100,
            timeout_ms: 1000,
            max_consecutive_failures: 3,
        }
    }
}

pub struct BudgetTracker {
    config: BudgetConfig,
    state_path: PathBuf,
}

impl BudgetTracker {
    pub fn can_execute(&self) -> Result<(), FallbackReason>;
    pub fn record_success(&mut self, cost_usd: f64, latency_ms: u64);
    pub fn record_failure(&mut self, err: &CeError);
}

pub enum FallbackReason {
    Disabled,
    BudgetExceeded { spent_usd: f64, limit_usd: f64 },
    CircuitOpen { consecutive_failures: u32 },
    Timeout,
}
```

---

## Configuration Schema (`ce-ai.toml` or `state.json`)

```toml
[decisions]
enabled = true
provider = "jev"
mode = "active" # "active" | "shadow" | "off"

[decisions.budget]
max_monthly_usd = 5.00
max_session_requests = 100
timeout_ms = 1000
max_consecutive_failures = 3

[decisions.jev]
model = "jev-latest"
endpoint = "https://api.typesafe.ai/v1"
```

---

## CLI Contracts

### 1. `ce-ai decisions status`
Displays decision engine configuration, provider status, API key presence, latency, and budget consumption.
```text
Decision Engine Status
  Enabled:     yes
  Provider:    jev
  Model:       jev-latest
  Mode:        active
  Health:      OK (42ms latency)
  API Key:     Configured (via TYPESAFE_API_KEY)
  
Budget & Consumption (September 2026)
  Spend:       $0.42 / $5.00 (8.4%)
  Requests:    38 / 100 session limit
  Circuit:     Closed (0 failures)
```

### 2. `ce-ai decisions auth [--key <KEY>] [--check]`
Tests or sets the API key securely. If `--key` is omitted, prompts interactively for secret input.
Saves to `~/.config/ce-ai/credentials.toml` (0600 permissions) if requested, or verifies current environment variables.

### 3. `ce-ai decisions setup [--preset <recommended|shadow|local>]`
Configures `[decisions]` in `ce-ai.toml` with safe defaults:
- `recommended`: `active` mode, `max_monthly_usd = 5.0`, `timeout_ms = 1000`.
- `shadow`: `shadow` mode for observation without execution influence.
- `local`: offline mock/rules provider.

### 4. `ce-ai doctor` Integration
Adds `probe_decision_engine`:
- Verifies provider health and response latency.
- Warns if budget is near or over capacity.
- If not configured, offers actionable developer suggestions:
  `ℹ Decision Engine: Jev provider available. Run 'ce-ai decisions setup' or export TYPESAFE_API_KEY.`
