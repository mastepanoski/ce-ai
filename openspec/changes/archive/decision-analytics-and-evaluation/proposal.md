# Proposal: Decision Analytics & Evaluation

## Problem Statement

Adding probabilistic micro-decisions (System 1) to an agent system does not automatically guarantee improvement. Without empirical measurement, introducing a classifier layer can introduce hidden degradation:
- **Incorrect routing & unnecessary escalation**: Directing trivial tasks to expensive reasoning models or routing complex architecture tasks to fast models causing downstream retries.
- **Provider overhead & latency**: Additional network roundtrips and latency added to developer commands.
- **Unmeasured provider failures & silent fallbacks**: Decision providers may fail, timeout, or hit budget limits without observability into failure rates.
- **Uncalibrated confidence**: Decisions executed with marginal confidence without visibility into distribution.
- **Unverified cost benefits**: Claims that adaptive routing saves model spend cannot be verified without comparing counterfactual static policies against adaptive execution outcomes.

To determine whether the Pluggable Decision Engine actually improves `ce-ai`, developers and operators need **Decision Analytics & Evaluation** (Issue [#387](https://github.com/mastepanoski/ce-ai/issues/387)).

---

## In-Scope

1. **Telemetry & Decision Event Schema (`src/decisions/analytics.rs`)**:
   - Structured `DecisionEvent` record capturing decision ID, timestamp, workflow ID, stage, decision type (`model_routing`, `skill_routing`, `risk_classification`, `stage_readiness`), provider, model, latency, confidence, outcome, fallback indicator, shadow mode indicator, estimated decision cost, and execution metadata.
   - Privacy-by-design invariant: raw prompts, source code, credentials, full conversations, and raw tool arguments are **never** stored by default. Only sanitized metadata (e.g. task summary hash, complexity, selected model) is persisted.

2. **Integration with Existing Usage Analytics Infrastructure**:
   - Reuse existing usage ledger storage under `.ce-ai/usage/` (specifically `.ce-ai/usage/decisions.jsonl`) using atomic file append primitives (`append_line_atomic`).
   - Correlation with harness `UsageRecord`s via `workflow_id`, `session_id`, or timestamp ranges.

3. **Counterfactual & Shadow Evaluation**:
   - Robust `shadow` mode support where the Decision Engine evaluates decisions, logs telemetry, but preserves deterministic / static behavior as authoritative.
   - Counterfactual comparisons comparing suggested vs authoritative choices and estimating/observing cost differentials.

4. **CLI Observability Subcommands (`ce-ai decisions stats` & `ce-ai decisions compare`)**:
   - `ce-ai decisions stats [--workflow <id>] [--type <type>] [--json]`:
     - Aggregates decision volume, latency (median, mean, p95), fallback rates, confidence distributions, and decision-specific metrics.
   - `ce-ai decisions compare [--workflow <id>] [--json]`:
     - Compares static vs adaptive model routing, evaluating cost differences and latency overhead, explicitly distinguishing `[observed]` from `[estimated]` values.

5. **Diagnostic Probing in `ce-ai doctor`**:
   - Probes decision event ledger integrity, reports total event counts, provider fallback rate, and telemetry health.

6. **Configuration in `state.json`**:
   - `DecisionAnalyticsConfig` with default enabled status, preserving user configuration without clobbering.

---

## Out-of-Scope

1. **Remote Telemetry Exfiltration**:
   - All analytics ledgers are local to `.ce-ai/usage/`; no external metrics reporting to third-party cloud services is performed.
2. **Real-Time Streaming Dashboards**:
   - Analytics are consumed via CLI (`ce-ai decisions stats`, `ce-ai decisions compare`) and doctor probes, not a daemon web server.
3. **Automated Dynamic Prompt Tuning**:
   - The analytics system measures and surfaces performance; it does not automatically re-train or rewrite provider prompt templates.

---

## Risk Evaluation

| Risk | Likelihood | Impact | Mitigation Strategy |
| :--- | :---: | :---: | :--- |
| **Privacy Leakage of Sensitive Data** | Low | High | Strict sanitization invariant: store only classified attributes (complexity, risk tier, class), never raw task prompt text or source code. |
| **Disk Growth from Telemetry Events** | Low | Low | Compact JSONL formatting (~150-250 bytes per event), deduplication, and standard ledger management. |
| **Telemetry Write Latency Overhead** | Low | Low | Non-blocking file appending with graceful degradation (telemetry failure never aborts user workflows). |
| **Misleading Cost Comparisons** | Low | Medium | Explicitly label all cost metrics as either `[observed]` (from harness token counts) or `[estimated]` (from pricing benchmarks). |

---

## Success Criteria

1. Decision events correlate with workflow runs and stages.
2. Provider latency and fallback rates are tracked accurately across all decision types.
3. Shadow mode logs decisions without affecting deterministic execution.
4. `ce-ai decisions stats` and `ce-ai decisions compare` render human-readable tables and machine-readable JSON.
5. Observed vs estimated metrics are unambiguous.
6. 100% green verification across unit tests, CLI integration tests, and formatting/clippy checks.
