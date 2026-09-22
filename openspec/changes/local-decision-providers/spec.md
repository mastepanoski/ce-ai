---
title: "Local Decision Engine Providers (Kev & Laya-MLX)"
domain: decisions
version: 1.0.0
last_updated: "2026-09-22"
---

# Specification: Local Decision Engine Providers (Kev & Laya-MLX)

## Functional Requirements

### 1. Provider Registration & Lifecycle
- **WHEN** `decisions.provider` is set to `"kev"`,
  - **THEN** `DecisionEngine::from_config` initializes a `KevProvider` pointing to `config.kev.endpoint`.
  - **AND** `provider.name()` returns `"kev"`.
- **WHEN** `decisions.provider` is set to `"laya"` or `"laya-mlx"`,
  - **THEN** `DecisionEngine::from_config` initializes a `LayaMlxProvider` pointing to `config.laya.endpoint` or socket.
  - **AND** `provider.name()` returns `"laya-mlx"`.
- **WHEN** an unconfigured or stopped local daemon fails to respond,
  - **THEN** `evaluate()` returns a `CeError::Network` or triggers non-blocking graceful fallback with `fallback_used = true`.
  - **AND** `ce-ai` never crashes, hangs, or emits non-zero exit codes during normal execution flows.

### 2. Wire Protocol Request & Response Mapping
- **WHEN** a `DecisionRequest` contains `DecisionQuestion::Boolean { id, question }`,
  - **THEN** the wire payload maps the question to System One `noul` with question text in `instructions`.
  - **AND** a response `noul: P` is mapped to `DecisionAnswer::Boolean` with `value: P >= 0.5`.
- **WHEN** a `DecisionRequest` contains `DecisionQuestion::Choice { id, question, options }`,
  - **THEN** the wire payload maps the question to System One `choice` with candidate labels as keys in `criteria`.
  - **AND** the returned choice, confidence, and option probabilities are stored in `DecisionAnswer::Choice`.
- **WHEN** a `DecisionRequest` contains `DecisionQuestion::Score { id, question, min, max }`,
  - **THEN** the wire payload maps the question to System One `score`.
  - **AND** the returned mean level index and confidence are stored in `DecisionAnswer::Score`.

### 3. CLI Ergonomics (`ce-ai decisions setup` & `ce-ai decisions test`)
- **WHEN** `ce-ai decisions setup --preset kev` or `--provider kev` is executed,
  - **THEN** `state.json` updates `provider = "kev"`, `mode = "active"`, and initial default `KevConfig`.
  - **AND** `spending_budget` is effectively free/unmetered ($0 cost).
- **WHEN** `ce-ai decisions setup --preset laya` or `--provider laya` is executed on macOS Apple Silicon,
  - **THEN** `state.json` updates `provider = "laya-mlx"`, `mode = "active"`, and initial default `LayaConfig`.
- **WHEN** `ce-ai decisions setup --provider laya` is executed on non-Apple-Silicon systems (Linux/Windows),
  - **THEN** the command warns the user that `laya-mlx` is Apple Silicon only and offers to configure `kev` instead.
- **WHEN** `ce-ai decisions test` is invoked,
  - **THEN** it executes a probe against the configured provider and prints health status and latency.

### 4. Diagnostics (`ce-ai doctor`)
- **WHEN** `ce-ai doctor` is executed with `provider = "kev"`,
  - **THEN** it probes `GET {kev.endpoint}/models`.
  - **AND** if the daemon is unreachable, outputs remediation commands to launch `kev.serve`.
- **WHEN** `ce-ai doctor` is executed with `provider = "laya"`,
  - **THEN** it validates host architecture (`aarch64-apple-darwin`).
  - **AND** probes the local socket/HTTP endpoint, warning if unreachable.
