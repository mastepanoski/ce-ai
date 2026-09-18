# Design: Risk-Aware Tool Execution (Intelligent Permission & Risk Engine)

## Overview

The Risk-Aware Tool Execution engine provides layered, defense-in-depth risk classification for tool invocations within `ce-ai`. It evaluates actions against deterministic security rules first, passing ambiguous operations to the System 1 Pluggable Decision Engine to produce an authoritative `ExecutionPolicy` (`Allow`, `RequireConfirmation`, `Deny`).

---

## Architectural Workflow & Invariants

```text
               Incoming Tool Invocation (tool, command, task)
                                  │
                                  ▼
               ┌──────────────────────────────────────┐
               │ 1. Deterministic Security Rules       │
               │    (Pre-filter hard rules)            │
               └──────────────────────────────────────┘
                   ├── Categorical Prohibited ───────► Deny (0ms, un-overridable)
                   ├── Known Safe Read-Only ──────────► Allow (0ms)
                   └── Ambiguous Operation
                               │
                               ▼
               ┌──────────────────────────────────────┐
               │ 2. System 1 Intent & Risk Evaluation  │
               │    (Pluggable Decision Engine)        │
               │    - destructive                      │
               │    - credential_sensitive             │
               │    - external_side_effect             │
               │    - privilege_escalation             │
               │    - irreversible                     │
               │    - scope_exceeds_task               │
               │    - overall risk choice              │
               └──────────────────────────────────────┘
                               │
                   ┌───────────┴───────────┐
                   ▼                       ▼
            Provider Succeeded      Provider Failed/Timeout
                   │                       │
                   ▼                       ▼
        ┌──────────────────────┐ ┌──────────────────────┐
        │ 3. Threshold Policy  │ │ 4. Conservative      │
        │    Evaluation        │ │    Fallback Policy   │
        │    score >= deny:    │ │    (Default:         │
        │      Deny            │ │     RequireConfirm)  │
        │    score >= confirm: │ └──────────────────────┘
        │      RequireConfirm  │           │
        │    else:             │           │
        │      Allow           │           │
        └──────────────────────┘           │
                   │                       │
                   └───────────┬───────────┘
                               ▼
               ┌──────────────────────────────────────┐
               │ 5. Audit Logging & Redaction          │
               │    (gate-events.jsonl / risk-events)  │
               └──────────────────────────────────────┘
                               │
                               ▼
                       ExecutionPolicy
               [Allow | RequireConfirmation | Deny]
```

---

## Core Data Structures & Schemas

### 1. Execution Policy Enum (`src/decisions/risk.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPolicy {
    /// Safe to execute automatically without user intervention.
    Allow,
    /// Ambiguous or potentially sensitive; requires explicit human confirmation.
    RequireConfirmation,
    /// Categorically dangerous or exceeding risk ceilings; blocked outright.
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskFallbackPolicy {
    RequireConfirmation,
    Deny,
}

impl Default for RiskFallbackPolicy {
    fn default() -> Self {
        Self::RequireConfirmation
    }
}
```

### 2. Risk Configuration Schema (`src/state/state.rs` & `src/decisions/risk.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskThresholds {
    #[serde(default = "default_confirmation_threshold")]
    pub confirmation_threshold_pct: u32,
    #[serde(default = "default_deny_threshold")]
    pub deny_threshold_pct: u32,
}

fn default_confirmation_threshold() -> u32 { 60 }
fn default_deny_threshold() -> u32 { 90 }

impl Default for RiskThresholds {
    fn default() -> Self {
        Self {
            confirmation_threshold_pct: default_confirmation_threshold(),
            deny_threshold_pct: default_deny_threshold(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub thresholds: RiskThresholds,
    #[serde(default)]
    pub fallback: RiskFallbackPolicy,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            thresholds: RiskThresholds::default(),
            fallback: RiskFallbackPolicy::default(),
        }
    }
}
```

### 3. Risk Evaluation Outcomes (`src/decisions/risk.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskDimensionScore {
    pub dimension: String,
    pub confidence: f64,
    pub elevated: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskEvaluationResult {
    pub tool: String,
    pub command: String,
    pub task: Option<String>,
    pub policy: ExecutionPolicy,
    pub reason: String,
    pub composite_risk_score: f64,
    pub dimensions: Vec<RiskDimensionScore>,
    pub fallback_applied: bool,
    pub latency_ms: u64,
}
```

---

## Evaluation Algorithm

1. **Step 1: Deterministic Pre-Filtering**:
   - Inspect command tokens and flags against static security rules:
     - Root filesystem escapes (`/`, `/etc`, `/usr`, `~/.ssh`, `~/.gnupg`, `state.json`).
     - Hard destructive patterns (`rm -rf /`, `mkfs`, `dd if=/dev/zero`, fork bombs).
     - Privilege escalations (`sudo`, `doas`, `su -`, `chmod 777`).
     - Branch protection bypasses (`git push --force origin main`).
   - If triggered: immediately return `ExecutionPolicy::Deny` with explanatory reason (`deterministic_rule: ...`). Zero network calls.

2. **Step 2: Safe Operations Filter**:
   - Read-only tools or commands (`ls`, `pwd`, `git status`, `git diff`, `cat`, `head`, `tail`) return `ExecutionPolicy::Allow` immediately.

3. **Step 3: Probabilistic Inquiry (Ambiguous Operations)**:
   - Construct `DecisionRequest` querying the 6 risk dimensions.
   - Dispatch to Decision Engine (Jev / local / mock).
   - If engine times out or fails:
     - Return `fallback` policy (`RequireConfirmation` or `Deny`) with `fallback_applied = true`.
   - Calculate composite risk score:
     - Highest risk dimension confidence or weighted aggregate.
   - If composite risk $\ge$ `deny_threshold_pct / 100.0`: return `ExecutionPolicy::Deny`.
   - If composite risk $\ge$ `confirmation_threshold_pct / 100.0`: return `ExecutionPolicy::RequireConfirmation`.
   - Otherwise: return `ExecutionPolicy::Allow`.

4. **Step 4: Audit Logging**:
   - Write sanitized and redacted event record to `risk-events.jsonl`.
