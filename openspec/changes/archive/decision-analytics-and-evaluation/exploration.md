# Exploration: Decision Analytics & Evaluation

## Technical Investigation & Context

The Pluggable Decision Engine (System 1) has been integrated across four primary operational surfaces:
1. **Model Route Selection** (`src/decisions/routing.rs`): Classifies tasks into `fast`, `standard`, and `reasoning` capability classes.
2. **Dynamic Skill Selection** (`src/decisions/skill_routing.rs`): Classifies semantic intent across 7 categories (`architecture`, `security`, `testing`, `debugging`, `documentation`, `code_review`, `research`).
3. **Risk-Aware Tool Execution** (`src/decisions/risk.rs`): Evaluates execution policies (`allow`, `prompt`, `deny`) for tool invocations.
4. **Work Readiness Advisory** (`src/decisions/readiness.rs`): Evaluates semantic completeness for ODD tasks and CE stages (`ready`, `warning`, `not_ready`).

Before Issue #387, each component evaluated requests independently. `risk.rs` maintained an ad-hoc `risk-events.jsonl`, but no unified analytics pipeline existed to correlate decisions with downstream execution outcomes, measure provider latency distribution, track fallback frequencies, or compare static vs adaptive routing costs.

---

## Architectural Options Evaluated

### Option 1: Embedded SQLite Database for Analytics
- **Approach**: Introduce `rusqlite` or `duckdb` to store relational decision events, executions, and usage records.
- **Pros**: Complex relational queries, SQL aggregation, indexing.
- **Cons**: Substantial crate binary footprint increase, C dependency linking complexities on Linux musl / macOS / Windows cross-compilation, schema migration overhead. Disconnected from existing `.ce-ai/usage/*.jsonl` format.
- **Verdict**: Rejected.

### Option 2: Unified JSONL Ledger Reusing Usage Infrastructure (Adopted)
- **Approach**: Store structured `DecisionEvent` entries in `.ce-ai/usage/decisions.jsonl` utilizing atomic file appending (`append_line_atomic`), matching `src/capture/ledger.rs`. Provide in-memory aggregation algorithms for stats and counterfactual comparisons.
- **Pros**:
  - Zero new third-party dependencies (uses existing `serde_json` and standard library).
  - 100% cross-platform reliability without C library linking issues.
  - Aligns with existing usage analytics architecture (`.ce-ai/usage/`).
  - Seamless text-based debugging (`jq`, `cat`, grep).
  - Blazing fast append performance (< 1ms).
- **Verdict**: Adopted.

---

## Privacy & Redaction Strategy

A critical requirement of Issue #387 is:
> "Raw state should not be stored by default. Analytics should prefer: decision type, classification, confidence, latency, provider metadata, execution metadata, outcome rather than source code, prompts, credentials, full conversations, tool arguments."

- **Implementation**:
  - `task_summary`: Truncated, sanitized representation (max 60 characters, stripped of secret patterns and paths) or an alphanumeric hash.
  - Execution metadata: Only stores non-sensitive parameters (e.g. `complexity: moderate`, `risk: low`, `tool: run_command`).
  - Strict absence of raw prompt templates, file contents, or shell arguments.

---

## Shadow Mode & Counterfactual Modeling

In `shadow` mode:
- The decision engine evaluates requests and logs telemetry events marked with `shadow_mode: true`.
- Deterministic behavior remains authoritative: the harness uses the default/static model assignment.
- When computing `ce-ai decisions compare`:
  - **Static routing cost**: Observed or estimated cost of the model actually used by default.
  - **Adaptive routing cost**: Counterfactual cost had the suggested class been used.
  - Differentiates `[observed]` (ground-truth tokens from harness usage ledger) from `[estimated]` (standard benchmark tokens per complexity tier).
