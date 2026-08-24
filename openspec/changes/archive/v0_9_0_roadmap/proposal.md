# OpenSpec Proposal: Release v0.9.0 — Hardening, Performance & Security Audit

## Problem Statement
Following the successful completion of Release v0.8.0, `ce-ai` requires a dedicated hardening phase prior to the `v1.0.0` Production Stable Release. The goal of Release v0.9.0 is to perform a comprehensive ISO 27001 / ISO 27002 security threat matrix audit, establish high-performance benchmarks enforcing sub-50ms execution bounds for state diffing and hash calculation, and achieve 100% test coverage across core state management and harness adapters.

## In-Scope vs Out-of-Scope Boundaries

### In-Scope:
- **Security Audit & Threat Matrix Suite (`tests/security.rs`)**:
  - Path traversal payload rejection (`..`, absolute paths, symlink attacks).
  - Atomic write integrity verification under unexpected process termination.
  - Corrupted JSON recovery and graceful exit code mapping.
  - Updating `SECURITY.md` supported version table.
- **Performance Benchmarks (`benches/benchmarks.rs`)**:
  - SHA256 integrity calculation across managed skill trees.
  - Archive extraction benchmarking.
  - Workspace override precedence merging (`State::load_with_workspace_overrides`).
- **Core Edge-Case Test Coverage**:
  - 100% test coverage across `src/state/`, `src/harness/`, and `src/opencode/`.

### Out-of-Scope:
- API contract freeze (deferred to `v1.0.0`).

## Success Criteria
1. Dedicated security test suite `tests/security.rs` passes 100% green.
2. Performance benchmarks confirm state loading and SHA256 calculation execute under 50ms.
3. 100% test coverage across core modules.
4. All CI matrix checks pass green across Linux, macOS, and Windows.
