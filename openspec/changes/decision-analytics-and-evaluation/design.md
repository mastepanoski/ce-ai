# Design: Decision Analytics & Evaluation

## System Architecture

```text
       Decision Invocations
  (Routing, Skills, Risk, Readiness)
                 │
                 ▼
          Decision Engine
                 │
                 ├── logs structured event (append_line_atomic)
                 ▼
    .ce-ai/usage/decisions.jsonl ───────────────┐
                                                │
    .ce-ai/usage/<author>.jsonl (UsageRecords) ──┼──▶ DecisionAnalytics Aggregator
                                                │           │
                                                │           ├── ce-ai decisions stats
                                                │           ├── ce-ai decisions compare
                                                │           └── ce-ai doctor probe
```

---

## Data Structures & Schemas

### 1. `DecisionType` Enum (`src/decisions/analytics.rs`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    ModelRouting,
    SkillRouting,
    RiskClassification,
    StageReadiness,
}

impl DecisionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ModelRouting => "model_routing",
            Self::SkillRouting => "skill_routing",
            Self::RiskClassification => "risk_classification",
            Self::StageReadiness => "stage_readiness",
        }
    }

    pub fn parse(s: &str) -> Result<Self, CeError> {
        match s.trim().to_lowercase().as_str() {
            "model_routing" | "routing" | "model" => Ok(Self::ModelRouting),
            "skill_routing" | "skills" | "skill" => Ok(Self::SkillRouting),
            "risk_classification" | "risk" => Ok(Self::RiskClassification),
            "stage_readiness" | "readiness" => Ok(Self::StageReadiness),
            other => Err(CeError::Usage(format!(
                "invalid decision type '{other}'. Valid types: model_routing, skill_routing, risk_classification, stage_readiness"
            ))),
        }
    }
}
```

### 2. `DecisionEvent` Record (`src/decisions/analytics.rs`)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionEvent {
    pub id: String,
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    pub decision_type: DecisionType,
    pub provider: String,
    pub model: String,
    pub latency_ms: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    pub outcome: String,
    #[serde(default)]
    pub fallback_used: bool,
    #[serde(default)]
    pub shadow_mode: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_cost_usd: Option<f64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}
```

### 3. Aggregation & Metrics Structs (`src/decisions/analytics.rs`)
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatencyStats {
    pub min_ms: u64,
    pub median_ms: u64,
    pub p95_ms: u64,
    pub mean_ms: u64,
    pub max_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceStats {
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub low_confidence_count: usize, // < 0.70
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionStats {
    pub total_runs: usize,
    pub fallback_count: usize,
    pub fallback_rate_pct: f64,
    pub shadow_count: usize,
    pub latency: LatencyStats,
    pub confidence: ConfidenceStats,
    pub outcome_distribution: BTreeMap<String, usize>,
    pub provider_distribution: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingComparison {
    pub total_runs: usize,
    pub fast_pct: f64,
    pub standard_pct: f64,
    pub reasoning_pct: f64,
    pub fallback_pct: f64,
    pub median_latency_ms: u64,
    pub static_routing_cost: CostValue,
    pub adaptive_routing_cost: CostValue,
    pub estimated_savings_usd: f64,
    pub estimated_savings_pct: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostValue {
    pub amount_usd: f64,
    pub is_observed: bool, // true if computed from usage ledger tokens, false if benchmark estimated
}
```

---

## Configuration Schema (`src/state/state.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionAnalyticsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub log_events: bool,
}

impl Default for DecisionAnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_events: true,
        }
    }
}
```

---

## Storage & Ledger Contract

- **Ledger Path**: `.ce-ai/usage/decisions.jsonl`
- **Atomic Persistence**:
  ```rust
  pub fn log_decision_event(config_dir: &Path, event: &DecisionEvent) -> Result<(), CeError>
  ```
  Appends serialized JSON lines atomically via `append_line_atomic`. Non-critical logging errors fail gracefully without interrupting user operations.
- **Reading Events**:
  ```rust
  pub fn read_decision_events(config_dir: &Path) -> Result<Vec<DecisionEvent>, CeError>
  ```
  Tolerates malformed lines with warnings, returning parsed valid records.

---

## CLI Specifications

### 1. `ce-ai decisions stats`
- **Syntax**: `ce-ai decisions stats [--workflow <id>] [--type <type>] [--json]`
- **Human-Readable Output Example**:
```text
Decision Engine Analytics

Total Decisions:          142
Fallback Rate:            2.1% (3/142)
Shadow Mode Decisions:    12
Median Latency:           31 ms (p95: 84 ms, mean: 36 ms)
Average Confidence:       87.4% (3 low-confidence <70%)

Distribution by Type:
  • Model Routing:        68 (47.9%)
  • Skill Routing:        34 (23.9%)
  • Risk Classification:  22 (15.5%)
  • Stage Readiness:      18 (12.7%)
```

### 2. `ce-ai decisions compare`
- **Syntax**: `ce-ai decisions compare [--workflow <id>] [--json]`
- **Human-Readable Output Example**:
```text
Decision Engine — Model Routing Comparison

Runs                     142
Fast                     38.0%
Standard                 44.0%
Reasoning                18.0%

Fallback                  2.1%
Median decision latency   31 ms

Model Cost Comparison:
  Static routing (standard)   $18.42  [estimated]
  Adaptive routing            $11.73  [estimated]
  Estimated Savings:          $6.69  (36.3%)
```
All costs are annotated with `[observed]` or `[estimated]`.
