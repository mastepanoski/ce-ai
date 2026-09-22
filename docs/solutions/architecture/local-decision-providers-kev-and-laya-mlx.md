---
title: "Local Decision Engine Providers: Universal Kev & Apple Silicon Laya-MLX"
category: "architecture"
module: "src/decisions/kev.rs, src/decisions/laya.rs"
date: "2026-09-22"
problem_type: "capability_expansion"
component: "decision_engine"
severity: "medium"
symptoms:
  - "Pluggable Decision Engine relied solely on external cloud API (Jev / TypeSafe AI) or mock test provider"
  - "Operators in offline, air-gapped, or cost-sensitive environments could not run real System 1 evaluations"
  - "High-performance Apple Silicon hardware was underutilized for sub-20ms local decision heads"
root_cause: "feature_gap"
resolution_type: "architectural_addition"
applies_when: "When configuring local, private, or zero-cost System 1 decision engine evaluation in ce-ai"
tags:
  - "decisions"
  - "local-providers"
  - "kev"
  - "laya-mlx"
  - "apple-silicon"
  - "offline-first"
---

# Local Decision Engine Providers: Universal Kev & Apple Silicon Laya-MLX

## Problem Statement
The Pluggable Decision Engine (System 1) in `ce-ai` provided cloud-based probabilistic micro-decisions via the Jev (TypeSafe AI) API and an offline mock provider for testing. However, operators requiring completely local, unmetered, zero-cost, and private execution had no intermediate option between cloud API keys and non-functional mock data. Furthermore, Apple Silicon machines have dedicated neural engines and unified memory that can execute small decision heads in under 20 milliseconds.

## Architectural Design & Key Decisions

1. **Dual-Tiered Local Strategy**:
   - **Tier 1 (Universal Local)**: `KevProvider` interfacing with Jared Palmer's Kev server (`python -m kev.serve --run jaredpalmer/kev-4b --port 8009`). It runs across all platforms (macOS, Linux, Windows), requiring no API key and zero cloud spend.
   - **Tier 2 (High-Speed Apple Silicon)**: `LayaMlxProvider` interfacing with MLX local daemons on Apple Silicon Macs (`aarch64-apple-darwin`), executing decision heads in under 20ms.

2. **System One Primitive Mapping**:
   Both local engines natively evaluate TypeSafe System 1 question types:
   - `DecisionQuestion::Boolean`: Mapped to `noul` (confidence interval between 0.0 and 1.0; `>= 0.5` is `true`).
   - `DecisionQuestion::Choice`: Mapped to `choice` with candidate labels and criteria.
   - `DecisionQuestion::Score`: Mapped to `score` with normalized level index.

3. **Hardware Architecture Gating**:
   `LayaMlxProvider` implements `is_apple_silicon()` checking `cfg!(all(target_os = "macos", target_arch = "aarch64"))`. When initialized on non-Apple-Silicon platforms, it emits a clear warning and gracefully advises using `ce-ai decisions setup --provider kev`.

4. **Fault Tolerance & Zero-Breakage Invariant**:
   If a local daemon (`kev` on port 8009 or `laya` on port 8008) is not running or unreachable, `ce-ai` never panics, crashes, or hangs. Calls gracefully fall back with `fallback_used = true` and record failure telemetry.

5. **Diagnostic & CLI Integration**:
   - Added `--preset kev` and `--preset laya` to `ce-ai decisions setup`.
   - Added `--provider <kev|laya>` and `--endpoint <url>` flags for customized configurations.
   - Added `ce-ai decisions test` to verify active provider health and latency.
   - Integrated probes into `ce-ai doctor` providing actionable daemon start commands when local endpoints are unreachable.
