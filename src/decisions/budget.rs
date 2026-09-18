//! Budget ceilings, rate limits, and circuit breaker for the Decision Engine.
//!
//! Enforces the Graceful Fallback invariant:
//! - When spend reaches `max_monthly_usd` or requests exceed `max_session_requests`,
//!   the Decision Engine immediately bypasses the provider without crashing.
//! - Consecutive network failures trip the circuit breaker, preventing repeated stalls.

use std::fmt;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Configuration for decision budget and circuit breaker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Maximum estimated monthly spend in USD before falling back to deterministic defaults.
    pub max_monthly_usd: f64,
    /// Maximum number of decision requests allowed in a single session.
    pub max_session_requests: u32,
    /// Timeout in milliseconds for an individual decision evaluation.
    pub timeout_ms: u64,
    /// Consecutive failures required to trip the circuit breaker.
    pub max_consecutive_failures: u32,
    /// Cool-off period in seconds while the circuit is open.
    pub cooloff_secs: u64,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_monthly_usd: 5.0,
            max_session_requests: 100,
            timeout_ms: 1000,
            max_consecutive_failures: 3,
            cooloff_secs: 60,
        }
    }
}

/// Persistent monthly ledger tracking accumulated decision usage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MonthlyLedger {
    /// Year-Month format: "YYYY-MM"
    pub month: String,
    pub accumulated_spend_usd: f64,
    pub total_requests: u32,
}

/// Operational state of the circuit breaker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CircuitState {
    #[default]
    Closed,
    Open {
        until_epoch_secs: u64,
    },
    HalfOpen,
}

/// Advisory reason explaining why a decision evaluation fell back to deterministic defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FallbackReason {
    Disabled,
    BudgetExceeded { spent_usd: f64, limit_usd: f64 },
    SessionLimitExceeded { count: u32, limit: u32 },
    CircuitOpen { consecutive_failures: u32 },
    Timeout,
}

impl fmt::Display for FallbackReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FallbackReason::Disabled => write!(f, "decision engine disabled"),
            FallbackReason::BudgetExceeded {
                spent_usd,
                limit_usd,
            } => {
                write!(
                    f,
                    "monthly budget limit reached (${spent_usd:.2} / ${limit_usd:.2})"
                )
            }
            FallbackReason::SessionLimitExceeded { count, limit } => {
                write!(f, "session request limit reached ({count} / {limit})")
            }
            FallbackReason::CircuitOpen {
                consecutive_failures,
            } => {
                write!(
                    f,
                    "circuit breaker open ({consecutive_failures} consecutive failures)"
                )
            }
            FallbackReason::Timeout => write!(f, "evaluation timed out"),
        }
    }
}

/// Tracks budget and manages circuit breaker lifecycle.
pub struct BudgetTracker {
    config: BudgetConfig,
    state_path: Option<PathBuf>,
    ledger: MonthlyLedger,
    session_requests: u32,
    consecutive_failures: u32,
    circuit_state: CircuitState,
}

impl BudgetTracker {
    /// Creates a budget tracker using the default path (`~/.config/ce-ai/decision_budget.json`).
    pub fn new(config: BudgetConfig, custom_path: Option<&Path>) -> Self {
        let path = custom_path
            .map(PathBuf::from)
            .or_else(Self::default_ledger_path);

        let mut tracker = Self {
            config,
            state_path: path.clone(),
            ledger: MonthlyLedger::default(),
            session_requests: 0,
            consecutive_failures: 0,
            circuit_state: CircuitState::Closed,
        };

        if let Some(p) = &path {
            tracker.load_from_path(p);
        }

        tracker.check_monthly_rollover();
        tracker
    }

    /// Creates an in-memory budget tracker for testing.
    pub fn in_memory(config: BudgetConfig) -> Self {
        let mut tracker = Self {
            config,
            state_path: None,
            ledger: MonthlyLedger::default(),
            session_requests: 0,
            consecutive_failures: 0,
            circuit_state: CircuitState::Closed,
        };
        tracker.check_monthly_rollover();
        tracker
    }

    pub fn config(&self) -> &BudgetConfig {
        &self.config
    }

    pub fn session_requests(&self) -> u32 {
        self.session_requests
    }

    pub fn accumulated_spend_usd(&self) -> f64 {
        self.ledger.accumulated_spend_usd
    }

    pub fn total_monthly_requests(&self) -> u32 {
        self.ledger.total_requests
    }

    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    pub fn circuit_state(&self) -> &CircuitState {
        &self.circuit_state
    }

    /// Evaluates whether the decision engine is permitted to execute.
    pub fn can_execute(&mut self) -> Result<(), FallbackReason> {
        self.check_monthly_rollover();

        // 1. Check Circuit Breaker
        let now = Utc::now().timestamp() as u64;
        if let CircuitState::Open { until_epoch_secs } = self.circuit_state {
            if now >= until_epoch_secs {
                self.circuit_state = CircuitState::HalfOpen;
            } else {
                return Err(FallbackReason::CircuitOpen {
                    consecutive_failures: self.consecutive_failures,
                });
            }
        }

        // 2. Check Session Limit
        if self.session_requests >= self.config.max_session_requests {
            return Err(FallbackReason::SessionLimitExceeded {
                count: self.session_requests,
                limit: self.config.max_session_requests,
            });
        }

        // 3. Check Monthly Budget
        if self.ledger.accumulated_spend_usd >= self.config.max_monthly_usd {
            return Err(FallbackReason::BudgetExceeded {
                spent_usd: self.ledger.accumulated_spend_usd,
                limit_usd: self.config.max_monthly_usd,
            });
        }

        Ok(())
    }

    /// Records a successful decision call.
    pub fn record_success(&mut self, cost_usd: Option<f64>) {
        self.check_monthly_rollover();
        self.session_requests += 1;
        self.consecutive_failures = 0;
        self.circuit_state = CircuitState::Closed;

        self.ledger.total_requests += 1;
        if let Some(cost) = cost_usd {
            if cost > 0.0 {
                self.ledger.accumulated_spend_usd += cost;
            }
        }

        self.persist();
    }

    /// Records a failed decision call, potentially tripping the circuit breaker.
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= self.config.max_consecutive_failures {
            let now = Utc::now().timestamp() as u64;
            self.circuit_state = CircuitState::Open {
                until_epoch_secs: now + self.config.cooloff_secs,
            };
        }
    }

    fn check_monthly_rollover(&mut self) {
        let current_month = Utc::now().format("%Y-%m").to_string();
        if self.ledger.month != current_month {
            self.ledger.month = current_month;
            self.ledger.accumulated_spend_usd = 0.0;
            self.ledger.total_requests = 0;
            self.persist();
        }
    }

    fn default_ledger_path() -> Option<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()?;
        Some(
            PathBuf::from(home)
                .join(".config")
                .join("ce-ai")
                .join("decision_budget.json"),
        )
    }

    fn load_from_path(&mut self, path: &Path) {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(loaded) = serde_json::from_str::<MonthlyLedger>(&content) {
                    self.ledger = loaded;
                }
            }
        }
    }

    fn persist(&self) {
        if let Some(path) = &self.state_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(serialized) = serde_json::to_string_pretty(&self.ledger) {
                let _ = crate::state::write_atomic(path, serialized.as_bytes());
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/budget_tests.rs"]
mod tests;
