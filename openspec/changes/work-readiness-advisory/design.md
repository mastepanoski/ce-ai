# Design Specification: Work Readiness & Verification Advisory

## System Architecture

```
                  Active Task Brief / Stage Spec
                     (ODD / CE OpenSpec)
                              │
                              ▼
                     Git Diff / Stat State
                              │
                              ▼
                   ReadinessEvaluator (System 1)
                              │
                              ▼
                Decision Request (Structured Questions)
               - dod_satisfied / requirements_clear
               - guardrails_respected / implementation_complete
               - tests_sufficient
               - graduation_recommended
                              │
                              ▼
                      DecisionProvider
                (Jev / Mock / Offline Fallback)
                              │
                              ▼
                  Composite Scoring & Mapping
                   - >= 80%  -> Ready (✓)
                   - >= 60%  -> Warning (△)
                   - < 60%   -> NotReady (⚠)
                              │
                              ▼
                 ReadinessEvaluationResult
                (CLI & Workflow Status Advisory)
```

---

## 1. Data Structures & Types (`src/decisions/readiness.rs`)

### Readiness Status
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessStatus {
    /// Composite confidence meets or exceeds ready threshold (>= 80%).
    Ready,
    /// Marginal confidence (60%..79%); attention recommended on highlighted dimensions.
    Warning,
    /// Low confidence (< 60%); incomplete implementation, missing tests, or unverified DoD.
    NotReady,
}

impl ReadinessStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Warning => "warning",
            Self::NotReady => "not_ready",
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            Self::Ready => "✓ Ready",
            Self::Warning => "△ Needs Attention",
            Self::NotReady => "⚠ Incomplete",
        }
    }
}
```

### Configuration Schema (`src/state/state.rs` & `src/decisions/readiness.rs`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadinessThresholds {
    #[serde(default = "default_ready_threshold")]
    pub ready_pct: u32,
    #[serde(default = "default_warning_threshold")]
    pub warning_pct: u32,
}

fn default_ready_threshold() -> u32 { 80 }
fn default_warning_threshold() -> u32 { 60 }

impl Default for ReadinessThresholds {
    fn default() -> Self {
        Self {
            ready_pct: default_ready_threshold(),
            warning_pct: default_warning_threshold(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReadinessConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub thresholds: ReadinessThresholds,
}
```

### Result Structs
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessDimensionScore {
    pub dimension: String,
    pub confidence: f64,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadinessEvaluationResult {
    pub target: String,
    pub workflow_mode: String,
    pub status: ReadinessStatus,
    pub composite_score: f64,
    pub dimensions: Vec<ReadinessDimensionScore>,
    pub advisory_notes: Vec<String>,
    pub fallback_applied: bool,
    pub latency_ms: u64,
}
```

---

## 2. Standard Semantic Question Sets

### ODD Dimensions:
1. `dod_satisfied`: "Are the Definition of Done checklist items satisfied by the current changes and implementation?"
2. `guardrails_respected`: "Were all task boundaries, guardrails, and negative constraints strictly honored?"
3. `graduation_recommended`: "Does the task scope, architectural footprint, or risk warrant graduating to OpenSpec?"
4. `ready_to_close`: "Is this organic task ready to be closed?"

### Compound Engineering (CE) Dimensions:
1. `requirements_clear`: "Are the requirements, scope, and acceptance criteria unambiguous?"
2. `implementation_complete`: "Is the planned implementation code complete according to the task checklist?"
3. `tests_sufficient`: "Are the unit, integration, or quality gate tests sufficient to verify the behavior?"
4. `documentation_complete`: "Is documentation, changelog, or architecture notes properly updated?"

---

## 3. Evaluation Algorithm

1. **Context Assembly**:
   - For ODD: Read brief from `odd/tasks/<feature>.md` (or synthesize from task argument), checklist status, and git diff stat summary.
   - For CE: Read active stage from `state.workflow`, task checklist from `openspec/changes/<feature>/tasks.md`, and git diff stat.
2. **Disabled/Unconfigured Fast-Path**:
   - If `readiness.enabled = false` or engine unconfigured: return `ReadinessStatus::Ready` with `fallback_applied = true`, composite score `1.0`, zero latency.
3. **Structured Request Dispatch**:
   - Send relevant boolean dimensions to Decision Engine.
4. **Scoring & Threshold Mapping**:
   - Calculate average or minimum confidence across required dimensions.
   - If composite $\ge$ `ready_pct / 100.0`: `ReadinessStatus::Ready`.
   - Else if composite $\ge$ `warning_pct / 100.0`: `ReadinessStatus::Warning`.
   - Else: `ReadinessStatus::NotReady`.
5. **Advisory Notes Generation**:
   - If `graduation_recommended` has high confidence: note "Recommend graduating task to formal OpenSpec (`ce-ai graduate`)."
   - For any failing dimension: note "Dimension '<dim>' confidence ({conf}%) is below threshold."

---

## 4. CLI Subcommand Contract

```bash
ce-ai decisions check-readiness [--feature <name>] [--task <text>] [--stage <n>] [--json] [--verbose]
```
Outputs human-readable status, dimension scores, advisory guidance, or structured JSON.
