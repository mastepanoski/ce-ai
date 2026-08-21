# Requirements Document: Release v0.9.0 — Hardening, Performance & Security Audit

## Problem & Context
As `ce-ai` reaches Release v0.8.0 with cross-platform native installers and multi-harness support, preparing for the `v1.0.0` Production Stable Release requires rigorous hardening, performance benchmarking, and ISO 27001 / ISO 27002 security threat matrix validation.

---

## Key Requirements & Boundaries

### 1. ISO 27001 & ISO 27002 Security Threat Matrix Audit (R1)
- **R1.1 Cryptographic Path Traversal Guard**: Verify `source::archive` rejects all directory traversal payloads (`..`, absolute paths, symlink tricks) prior to file mutation.
- **R1.2 Atomic Write Integrity**: Verify `crate::state::write_atomic` enforces zero-byte corruption protection across power loss or process termination (NIST SP 800-53 CP-9/10 compliance).
- **R1.3 Security Test Suite (`tests/security.rs`)**: Create dedicated security test suite testing path traversal rejection, corrupted JSON gracefully handling, and permission boundaries.
- **R1.4 Security Policy Update (`SECURITY.md`)**: Update supported versions matrix to maintain active support for `0.8.x`.

### 2. High-Performance Benchmarks (< 50ms Target) (R2)
- **R2.1 Benchmark Infrastructure (`benches/benchmarks.rs`)**: Establish benchmark suite measuring execution time for:
  - Archive extraction (`source::archive::safe_extract`).
  - SHA256 integrity calculation across managed skill trees (`opencode::manifest::InstallManifest`).
  - Workspace override precedence merging (`State::load_with_workspace_overrides`).
- **R2.2 Sub-50ms Response Guarantee**: All state loading and SHA256 drift calculation operations MUST execute under 50 milliseconds.

### 3. Core State & Harness Adapter Hardening (R3)
- **R3.1 100% Edge-Case Coverage**: Add unit test coverage for invalid `.ce-ai.json` syntax, corrupted `state.json`, and missing harness paths.
- **R3.2 Robust Fallback Error Handling**: Graceful error reporting via `CeError` mapping to standard exit codes (Usage=2, State=3, IO=4, Network=5, Verification=6).

---

## Out-of-Scope Boundaries (Non-Goals)
- Frozen 1.0 API contract freeze (deferred to `v1.0.0`).
- GUI desktop application (out of scope).

---

## Success Criteria
1. Dedicated security test suite `tests/security.rs` passes 100% clean.
2. Criterion / integration benchmarks in `benches/benchmarks.rs` confirm sub-50ms performance for state diffing and hash calculation.
3. 100% test coverage across core `src/state/`, `src/harness/`, and `src/opencode/` modules.
4. `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `make e2e` pass green across Linux, macOS, and Windows.
