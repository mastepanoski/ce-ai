---
title: "Decision Analytics & Counterfactual Evaluation Engine"
category: "architecture"
module: "src/decisions/analytics.rs"
date: "2026-09-22"
problem_type: "observability"
component: "decision_engine"
severity: "medium"
symptoms:
  - "Pluggable Decision Engine had no empirical observability into latency, confidence, or fallback frequencies"
  - "Operators could not compare static single-model routing costs against adaptive routing savings"
  - "Shadow mode lacked telemetry recording to evaluate counterfactual performance before live enforcement"
root_cause: "feature_gap"
resolution_type: "architectural_addition"
applies_when: "When observing decision engine operations, running shadow evaluation, or calculating routing ROI"
tags:
  - "decisions"
  - "analytics"
  - "telemetry"
  - "cost-comparison"
  - "shadow-mode"
  - "privacy"
---

# Decision Analytics & Counterfactual Evaluation Engine

## Problem Statement
The Pluggable Decision Engine (System 1) lacked a dedicated observability and evaluation mechanism. As users adopted model routing, skill routing, risk classification, and readiness evaluation, there was no empirical ledger to measure latency overhead, confidence trends, fallback frequency, or cost efficiency. Furthermore, testing new decision models in `shadow` mode required capturing counterfactual telemetry without influencing authoritative deterministic execution.

## Architectural Design & Key Decisions

1. **Lightweight JSONL Ledger (`.ce-ai/usage/decisions.jsonl`)**:
   Reused existing `.ce-ai/usage/` shard conventions. Atomic file appends via `std::fs::OpenOptions::append` provide resilient multi-process concurrency without requiring heavyweight relational database engines (e.g. SQLite).
2. **Strict Privacy Invariant & State Redaction**:
   Enforced strict sanitization preventing raw source code, prompts, credentials, full conversations, and raw shell/tool arguments from ever being persisted. Only categorized metadata (e.g., complexity label, sanitized task summary hash, tool name) is logged.
3. **Counterfactual Shadow Mode Evaluation**:
   In `shadow` mode, evaluators perform full recommendation requests against providers and record telemetry (`shadow_mode: true`), while returning deterministic authoritative baselines to ensure zero behavioral perturbation.
4. **Empirical vs Benchmark Cost Accounting**:
   Separated cost reporting into observed (`[observed]`, calculated when tokens exist in `.ce-ai/usage/*.jsonl`) and benchmark estimates (`[estimated]`, based on industry standard capability pricing).
5. **CLI & Health Integration**:
   - `ce-ai decisions stats`: Provides total decisions, fallback rates, shadow counts, latency percentiles (`min`, `median`, `p95`, `mean`, `max`), and type distributions with `--workflow` and `--type` filters.
   - `ce-ai decisions compare`: Compares model routing distribution and cost savings against static defaults.
   - `ce-ai doctor`: Checks ledger health and reports empirical fallback rates.
