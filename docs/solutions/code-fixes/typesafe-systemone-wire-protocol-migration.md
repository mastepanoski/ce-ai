---
title: "TypeSafe AI System One Wire Protocol Migration and Parity for Jev Provider"
date: "2026-09-24"
category: "code-fixes"
module: "src/decisions/jev.rs"
component: "decision_engine"
severity: "high"
problem_type: "code_fix"
symptoms:
  - "JevProvider::evaluate sending requests to non-existent /decide endpoint returning HTTP 404"
  - "Inconsistent wire protocol schemas between cloud provider (jev) and local providers (kev, laya)"
  - "ce-ai tools init codegraph reporting false-positive already initialized on checkouts with only .gitignore"
root_cause: "Hallucinated endpoint (/decide) and custom wire structs in JevProvider instead of the canonical TypeSafe AI System One wire protocol (POST /v1/systemone) using noul, choice, score primitives. Additionally, init_codegraph only checked directory existence rather than index existence."
resolution_type: "code_fix"
tags: [code-fixes, decisions, jev, typesafe, systemone, wire-protocol, codegraph]
applies_when: "When encountering 404 errors with Jev decision provider or false-positive initialization skips with companion tools in newly created worktrees."
---

# TypeSafe AI System One Wire Protocol Migration and Parity for Jev Provider (Release v1.68.2)

## Problem

During a codebase audit of the Decision Engine, two structural issues were identified:
1. **Hallucinated Endpoint & Wire Schema in `JevProvider`**: `src/decisions/jev.rs` was calling `{endpoint}/decide` with a non-standard JSON payload (`JevWireRequest`), which returns HTTP 404 against `api.typesafe.ai`. In reality, TypeSafe AI exposes the canonical System One wire endpoint at `POST /v1/systemone` using `state`, `model`, and typed `questions` with `noul` (boolean), `choice` (categorical), and `score` (numeric) primitives.
2. **False-Positive Companion Tool Initialization in Worktrees**: In newly spawned git worktrees where `.codegraph/.gitignore` was checked out from source control, `ce-ai tools init codegraph` checked `target_path.join(".codegraph").exists()`, falsely reporting that the index was already initialized even though `codegraph.db` had not been created.

## Root Cause Analysis

1. `jev.rs` had been authored prior to the full TypeSafe System One specification adoption in `kev.rs` and `laya.rs`, resulting in duplicate and divergent types (`JevWireRequest` vs `SystemOneWireRequest`).
2. Directory existence was an insufficient indicator for an initialized CodeGraph database when git tracks a directory containing only `.gitignore`.

## Solution

1. **Centralized Wire Models in `src/decisions/types.rs`**:
   - Moved canonical `SystemOneWireQuestion`, `SystemOneWireRequest`, `SystemOneWireAnswer`, and `SystemOneWireResponse` to `types.rs`.
   - Implemented shared serialization helpers `build_systemone_questions` and `parse_systemone_answers`.
   - Re-exported wire types in `src/decisions/kev.rs` and `src/decisions/jev.rs` for backward compatibility.
2. **Migrated `JevProvider` in `src/decisions/jev.rs`**:
   - Updated endpoint routing to `{endpoint}/systemone` (or `{endpoint}/v1/systemone`).
   - Integrated shared System One request building and response decoding with usage cost extraction.
3. **Refined CodeGraph Initialization Guard**:
   - Added `is_codegraph_initialized` in `src/commands/tools.rs` to detect when `.codegraph/` contains only `.gitignore`, executing `codegraph init` rather than falsely skipping.
4. **Empirical Verification**:
   - Expanded unit tests in `src/decisions/tests/jev_tests.rs` for System One wire round-trip serialization and telemetry cost parsing.
   - 100% green unit and integration tests (`cargo test`), strict clippy (`cargo clippy --all-targets --all-features -- -D warnings`), and containerized Docker E2E gate (`make e2e`).
