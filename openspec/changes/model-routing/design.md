# Design: Adaptive Model Route Selection (System One Advisory Layer)

## Overview

The Adaptive Model Route Selection layer enables `ce-ai` to dynamically evaluate task descriptions and advisory requirements via the Pluggable Decision Engine (System 1) and recommend an optimal model capability class (`fast`, `standard`, or `reasoning`).

The system maintains a clean separation of concerns:
- **Decision Engine (Provider)**: Probabilistically classifies task dimensions (`complexity`, `needs_reasoning`, `needs_large_context`, `risk`) in < 100ms.
- **Routing Engine (`ce-ai`)**: Deterministically maps classified dimensions to logical model classes according to user-configured thresholds, enforces strict override precedence, and handles graceful capability degradation.

---

## Architecture & Precedence Flow

```text
Task Description (from CLI or Agent Workflow)
                     │
                     ▼
  ┌─────────────────────────────────────┐
  │   1. Explicit CLI / User Override   │ ──── (If present) ───► Model = Explicit
  └─────────────────────────────────────┘
                     │ (None)
                     ▼
  ┌─────────────────────────────────────┐
  │   2. Agent Slot Static Assignment   │ ──── (If assigned) ──► Model = Assigned
  └─────────────────────────────────────┘
                     │ (None or Dynamic Mode)
                     ▼
  ┌─────────────────────────────────────┐
  │   3. Adaptive Model Routing Engine  │ ──── (If enabled)
  │      - Query DecisionProvider       │
  │      - Classify Task Dimensions     │
  │      - Map to Logical Model Class   │
  │      - Resolve Class in Catalog     │ ──── (Resolved) ─────► Model = Catalog Target
  └─────────────────────────────────────┘
                     │ (Disabled, Provider Failure, or Unmapped)
                     ▼
  ┌─────────────────────────────────────┐
  │   4. Default Harness Model Fallback │ ─────────────────────► Model = Harness Default
  └─────────────────────────────────────┘
```

---

## Domain Types & Configuration Schema

### 1. Logical Model Class (`src/decisions/routing.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelClass {
    Fast,
    Standard,
    Reasoning,
}

impl ModelClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelClass::Fast => "fast",
            ModelClass::Standard => "standard",
            ModelClass::Reasoning => "reasoning",
        }
    }
}
```

### 2. Model Class Catalog (`src/decisions/routing.rs` & `src/state/state.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModelClassCatalog {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fast: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

impl ModelClassCatalog {
    pub fn get(&self, class: ModelClass) -> Option<&str> {
        match class {
            ModelClass::Fast => self.fast.as_deref(),
            ModelClass::Standard => self.standard.as_deref(),
            ModelClass::Reasoning => self.reasoning.as_deref(),
        }
    }
}
```

### 3. Routing Thresholds (`src/decisions/routing.rs`)

To ensure `State` satisfies `Eq` without floating-point precision issues, all thresholds are stored as integer percentages (`0..=100`):

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingThresholds {
    #[serde(default = "default_reasoning_threshold")]
    pub reasoning_threshold_pct: u32,
    #[serde(default = "default_standard_threshold")]
    pub standard_threshold_pct: u32,
}

fn default_reasoning_threshold() -> u32 {
    75 // 75% confidence
}

fn default_standard_threshold() -> u32 {
    50 // 50% confidence
}

impl Default for RoutingThresholds {
    fn default() -> Self {
        Self {
            reasoning_threshold_pct: default_reasoning_threshold(),
            standard_threshold_pct: default_standard_threshold(),
        }
    }
}
```

### 4. Model Routing Configuration (`src/state/state.rs`)

Nested inside `DecisionsConfig`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModelRoutingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub models: ModelClassCatalog,
    #[serde(default)]
    pub thresholds: RoutingThresholds,
}
```

---

## Policy Evaluation & Deterministic Mapping

### Standard Decision Questions Asked

The `ModelRouter` dispatches four typed questions to the `DecisionEngine`:
1. `complexity`: Choice `["trivial", "moderate", "complex"]`
2. `needs_reasoning`: Boolean (requires architectural planning, deep logic, or non-obvious debugging)
3. `needs_large_context`: Boolean (requires extensive codebase or multi-file context)
4. `risk`: Choice `["low", "medium", "high"]`

### Classification Decision Matrix

Given the answers:
1. **Target `Reasoning`**:
   - `needs_reasoning == true` with `confidence >= reasoning_threshold_pct`, OR
   - `complexity == "complex"`, OR
   - `risk == "high"`.
2. **Target `Fast`**:
   - `complexity == "trivial"`, AND
   - `risk == "low"`, AND
   - `needs_reasoning == false`, AND
   - `needs_large_context == false`.
3. **Target `Standard`**:
   - All other cases (balanced default for typical coding tasks).

### Graceful Capability Fallback Chain

If the catalog does not configure the target class, the router degrades gracefully along capability adjacency:
- `Reasoning` target missing ➔ try `Standard` ➔ try `Fast` ➔ default fallback.
- `Fast` target missing ➔ try `Standard` ➔ try `Reasoning` ➔ default fallback.
- `Standard` target missing ➔ try `Fast` ➔ try `Reasoning` ➔ default fallback.

If the decision provider times out, encounters an error, or the budget is exhausted, the router returns the standard/default model with a diagnostic reason (exit code 0).

---

## CLI Specifications

### 1. `ce-ai models route "<task description>" [--json] [--verbose]`

Evaluates the task description against the active decision provider and displays:
- Recommended Model Class (`fast`, `standard`, `reasoning`)
- Resolved Model Identifier (e.g. `anthropic/claude-3-5-sonnet`)
- Decision Dimensions (Complexity, Needs Reasoning, Large Context, Risk)
- Routing Rationale and confidence percentages.

JSON Output schema:
```json
{
  "task": "Extract regex matches from markdown",
  "recommended_class": "fast",
  "resolved_model": "anthropic/claude-3-5-haiku",
  "dimensions": {
    "complexity": "trivial",
    "needs_reasoning": false,
    "needs_large_context": false,
    "risk": "low"
  },
  "rationale": "Task classified as trivial complexity with low risk.",
  "fallback_applied": false
}
```

### 2. Diagnostics in `ce-ai doctor`

`ce-ai doctor` checks:
- Model routing status (`enabled` / `disabled`).
- Configured model classes: `fast`, `standard`, `reasoning`.
- Warns if routing is enabled but no model classes are mapped.
