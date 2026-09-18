# Exploration: Adaptive Model Route Selection

## Technical Investigation & Options Evaluated

### 1. Where Should Routing Logic Live?

- **Option A: Routing Policy Inside the Decision Provider (Vendor Black Box)**
  - *Description:* The external API receives the task description and directly returns `"anthropic/claude-3-5-sonnet"`.
  - *Tradeoffs:* Tightly couples `ce-ai` to vendor-specific model availability; impossible for users to customize thresholds or map self-hosted or alternative models; violates provider-agnostic principles.
- **Option B: Separation of Concerns (Probabilistic Classification + Deterministic Policy) — SELECTED**
  - *Description:* The Decision Engine (`DecisionProvider`) classifies factual task attributes (`complexity`, `needs_reasoning`, `needs_large_context`, `risk`). `ce-ai`'s internal policy engine then applies user-configured thresholds and maps the classification into logical model classes (`fast`, `standard`, `reasoning`).
  - *Benefits:* Full explainability, vendor independence, customizable thresholds, and deterministic testability.

### 2. Logical Model Classes

Rather than specifying models for every conceivable scenario, three capability tiers capture > 95% of agent operational requirements:
- **`fast`**: Cost-efficient, high-throughput model (< $0.50 / 1M tokens, < 1s latency). Used for trivial edits, single-file typo fixes, test generation, and documentation.
- **`standard`**: Workhorse programming model ($3–$15 / 1M tokens, balanced latency). Used for typical feature implementation, standard bug fixes, and unit test refactoring.
- **`reasoning`**: Extended thinking / high-compute model ($15+ / 1M tokens, deep planning). Used for architectural design, multi-repository changes, complex state machine bugs, and critical security audits.

### 3. State Schema & Compatibility

In `src/state/state.rs`:
`DecisionsConfig` already exists from Phase 1. We can extend it with an optional or defaulted `routing: ModelRoutingConfig` field.
To uphold `State`'s `Eq` trait requirement, all threshold values will be represented as integer percentages:
- `reasoning_threshold_pct: u32` (default: 75, representing 0.75 / 75% confidence)
- `standard_threshold_pct: u32` (default: 50, representing 0.50 / 50% confidence)

### 4. Failure Modes & Graceful Degradation Matrix

| Scenario | Decision Engine Behavior | Routing Engine Behavior | Exit Code |
| :--- | :--- | :--- | :--- |
| Routing Disabled (`enabled = false`) | No request made | Returns static / default model | 0 |
| Decision Provider Error / Timeout | Circuit breaker records failure | Falls back to default model | 0 |
| Budget Ceilings Exceeded | Returns fallback reason | Falls back to default model | 0 |
| Target Model Class Unconfigured | N/A | Falls back along class chain (`reasoning` ➔ `standard` ➔ `fast` ➔ default) | 0 |
| Explicit Slot Override Present | N/A | Explicit override always wins | 0 |

### 5. Architectural Decision Records (ADRs)

- **ADR-1: Deterministic Policy Separation**: The external provider is strictly a classifier of task properties; policy determination is 100% owned by `ce-ai`.
- **ADR-2: Explicit Override Precedence**: An explicit user command or slot configuration always takes precedence over adaptive suggestions.
- **ADR-3: Integer Percentage Storage**: Avoid floating point in `state.json` to ensure strict derivation of `Eq` and eliminate precision serialization drift.
