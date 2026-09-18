# Exploration: Pluggable Decision Engine & Jev Provider Architecture

## Context & Motivation

To support fast, structured, probabilistic micro-decisions without introducing full LLM invocation overhead, `ce-ai` requires an advisory **System One** decision layer.

Prior art in this space includes:
- **`jevwire` (TypeSafe AI's Jev integration)**: Defines an embeddable decision model library and MCP layer where deterministic prefilters precede probabilistic classification, external providers are strictly advisory, and mock/fake providers ensure testability.
- **`ce-ai`'s `HarnessAdapter` pattern**: Clean domain traits (`src/harness/mod.rs`) decoupling `ce-ai` from the idiosyncratic persistence and CLI interfaces of 12 distinct AI harnesses.
- **`ce-ai`'s Usage Ledger (`src/commands/usage.rs`)**: Deterministic sharded append-only JSONL accounting for token and cost tracking.

This exploration investigates the core architectural choices for:
1. Provider abstraction & question representation.
2. Jev HTTP integration and error handling.
3. API key resolution and credential safety.
4. Budget tracking, rate limits, and circuit breakers.

---

## Technical Investigation

### 1. Decision Provider Trait & Question Representation

We evaluated three approaches for modeling questions and answers:

#### Approach A: Dynamic JSON Payloads (Untyped)
```rust
pub trait DecisionProvider {
    fn evaluate(&self, prompt: &str, schema: serde_json::Value) -> Result<serde_json::Value, CeError>;
}
```
* **Pros**: Highly flexible; easily supports any future API shape.
* **Cons**: No compile-time guarantees; consumers must manually parse and validate JSON payloads; error-prone; breaks domain typing.

#### Approach B: Direct Vendor SDK Coupling
Bind directly to a third-party `typesafe-ai` crate.
* **Pros**: Minimal glue code for Jev.
* **Cons**: Direct compile-time dependency on external proprietary SDK; impossible to run offline mock tests cleanly without vendor dependencies; violates the provider-neutral mandate of `ce-ai`.

#### Approach C: Typed Domain Primitives (Selected)
```rust
pub enum DecisionQuestion {
    Boolean { id: String, question: String },
    Choice { id: String, question: String, options: Vec<String> },
    Score { id: String, question: String, min: f64, max: f64 },
}

pub enum DecisionAnswer {
    Boolean { value: bool, confidence: f64 },
    Choice { selected: String, confidence: f64, probabilities: BTreeMap<String, f64> },
    Score { score: f64, confidence: f64 },
}
```
* **Pros**: Clean domain separation; zero vendor leak; compile-time exhaustiveness checks; trivial to implement offline `MockDecisionProvider`.
* **Cons**: Requires translating domain enums into Jev JSON payloads inside `src/decisions/jev.rs`.

---

### 2. API Key Management & Credential Resolution

We evaluated three credential storage mechanisms:

#### Option 1: Git-tracked Project Config (`ce-ai.toml`)
* **Rejected**: Severe security risk. Committing API keys to repositories violates ISO 27001 / SOC 2 compliance.

#### Option 2: OS Keychain (`keyring-rs`)
* **Pros**: Secure encrypted OS store.
* **Cons**: Fails on headless CI runners (GitHub Actions Linux/Windows containers), adds heavyweight C dependencies (D-Bus, Secret Service), and frequently hangs or prompts on remote SSH sessions.

#### Option 3: Hierarchical Env Vars + Local Secured File (Selected)
* **Resolution Order**:
  1. Process Environment: `TYPESAFE_API_KEY`
  2. Legacy / Alias Environment: `JEV_API_KEY`
  3. User Global Config: `~/.config/ce-ai/credentials.toml` (created with `0600` permissions on Unix).
* **Pros**: Works seamlessly across CI/CD, local terminals, IDE extensions, and containers. Zero additional native dependencies.

---

### 3. Budget Ceilings & Circuit Breaker Architecture

If an automated agent makes decisions in a loop, an infinite loop or unexpected spike could rapidly consume budget.

We evaluated two tracking models:

#### Model 1: Ephemeral In-Memory Counter
* Tracks requests during the lifetime of a single CLI invocation.
* **Flaw**: `ce-ai` commands are ephemeral processes. Each CLI command exits after execution, meaning an in-memory counter resets to zero on every turn.

#### Model 2: Lightweight Persistent Budget Ledger (Selected)
* Persisted in `~/.config/ce-ai/budget_ledger.json` (or `.ce-ai/decisions/ledger.json` at workspace scope).
* Tracks:
  - `month`: Current billing month (e.g. `2026-09`).
  - `accumulated_cost_usd`: Running estimated dollar spend.
  - `accumulated_requests`: Total decision requests evaluated.
  - `consecutive_failures`: Count of sequential timeouts or errors.
* **Circuit Breaker Logic**:
  - If `accumulated_cost_usd >= max_monthly_usd` ➔ Provider state = `BudgetExceeded`.
  - If `consecutive_failures >= max_consecutive_failures` (e.g. 3) ➔ Provider state = `CircuitOpen` for `cooloff_period_secs`.
  - In both states, execution immediately bypasses the provider and uses deterministic fallbacks.

---

## Architectural Tradeoff Matrix

| Dimension | Selected Approach | Tradeoff / Justification |
|---|---|---|
| **Question Primitives** | Typed Enums (`Boolean`, `Choice`, `Score`) | Isolates core from vendor wire protocols; enables offline mocks. |
| **HTTP Client** | `reqwest::blocking` with Rustls | Reuses existing `ce-ai` dependencies (`Cargo.toml`) without pulling in heavy async runtimes for simple CLI commands. |
| **Fallback Policy** | Strict Non-Blocking Fallback | Decision Engine failure must NEVER crash developer commands. |
| **Credentials** | Env var precedence + `credentials.toml` | CI-friendly, container-friendly, zero secret leaks into git. |
| **Budget Enforcement** | Persistent monthly counter + circuit breaker | Protects developer from runaway cost loops across multiple CLI runs. |
