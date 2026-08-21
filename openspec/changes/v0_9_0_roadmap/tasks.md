# OpenSpec Tasks: Release v0.9.0 Implementation Plan

- [ ] **Unit 1: Security Audit & Threat Matrix Test Suite (`tests/security.rs`)**
  - [ ] Implement path traversal payload rejection test suite in `tests/security.rs`.
  - [ ] Implement atomic write integrity & tempfile cleanup test suite in `tests/security.rs`.
  - [ ] Update `SECURITY.md` supported versions table.

- [ ] **Unit 2: Performance Benchmarks (`benches/benchmarks.rs`)**
  - [ ] Implement benchmark for state resolution and workspace overrides (< 50ms).
  - [ ] Implement benchmark for SHA256 integrity hash calculation (< 50ms).

- [ ] **Unit 3: Core Hardening & Test Coverage**
  - [ ] Add edge-case tests for invalid `.ce-ai.json` syntax and corrupted `state.json`.
  - [ ] Verify 100% test coverage across core modules.
