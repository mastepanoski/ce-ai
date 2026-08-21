# OpenSpec Tasks: Release v0.9.0 Implementation Plan

- [x] **Unit 1: Security Audit & Threat Matrix Test Suite (`tests/security.rs`)**
  - [x] Implement path traversal payload rejection test suite in `tests/security.rs`.
  - [x] Implement atomic write integrity & tempfile cleanup test suite in `tests/security.rs`.
  - [x] Update `SECURITY.md` supported versions table.

- [x] **Unit 2: Performance Benchmarks (`benches/benchmarks.rs`)**
  - [x] Implement benchmark for state resolution and workspace overrides (< 50ms).
  - [x] Implement benchmark for SHA256 integrity hash calculation (< 50ms).

- [x] **Unit 3: Core Hardening & Test Coverage**
  - [x] Add edge-case tests for invalid `.ce-ai.json` syntax and corrupted `state.json`.
  - [x] Verify 100% test coverage across core modules.
