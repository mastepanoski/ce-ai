---
title: System Security, Compliance and Governance Specification
domain: security
version: 1.0.0
last_updated: 2026-10-23
dependencies: []
---

# System Security, Compliance & Governance Specification

This canonical specification establishes the mandatory, auditable security and governance invariants for `ce-ai` and host projects adopted under Compound Engineering.

## 1. Governance & Standards Framework

All automated workflows and binary executions adhere strictly to:
1. **ISO/IEC 27001 & 27002**: Cryptographic integrity, asset classification, and least privilege.
2. **OWASP Top 10 for LLMs / Applications**: Prevention of prompt injection, sensitive data leakage, and supply-chain compromise.
3. **NIST SP 800-53 / NIST AI Risk Management Framework (AI RMF 1.0)**: Atomic mutation guarantees, process contention, and audit trail resilience.

## 2. Capabilities & Requirements

### R1: ISO/IEC 27001 & 27002 Cryptographic Integrity

The system MUST verify and preserve file integrity deterministically across all operations.

- **WHEN** any harness component, skill asset, or managed block is synchronized,
- **THEN** its SHA256 cryptographic digest MUST be calculated and compared against the baseline manifest (`manifest.json` / `state.json`).
- **WHEN** unauthorized drift or file tampering is detected,
- **THEN** the system MUST report drift and restore the authentic asset during sync or reject corrupted caches with exit code 6 (`Verification`).

### R2: OWASP LLM-02 Anti-Credential Exposure and Sanitization

The system MUST prevent accidental leakage of sensitive tokens, secrets, or API keys across all outputs, logs, telemetry, and constellation manifests.

- **WHEN** a declarative remote repository URL is declared in `docs/arch/system.yaml` or CLI arguments,
- **THEN** `validate_remote_repo_url` MUST reject URLs containing embedded credentials (e.g. `https://user:token@host/repo`) with exit code 2 (`Usage`).
- **WHEN** bug reports, diagnostics, or telemetry payloads are generated (`ce-ai report-bug`),
- **THEN** all tokens, bearer keys, and environment variables MUST be sanitized with zero-leak guarantees before transmission or serialization.

### R3: NIST SP 800-53 & NIST AI RMF Atomic Write Resilience

Configuration stores, state files, and instruction surfaces MUST never be left in a corrupted or half-written state due to crashes, power loss, or process termination.

- **WHEN** mutations target `state.json`, `opencode.json`, `AGENTS.md`, or managed harness configs,
- **THEN** writes MUST use atomic temporary file creation followed by rename (`crate::state::write_atomic`).
- **WHEN** a write operation fails at any step,
- **THEN** the target file MUST remain untouched in its previous valid state with zero residual temp files.

### R4: Least Privilege and Sandboxed Process Contention

Subprocess execution and agent tooling MUST run within bounded sandboxes and least-privilege configurations.

- **WHEN** CLI commands probe Git state or execute tooling,
- **THEN** hooks environment variables (`GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE`, `GIT_PREFIX`) MUST be stripped to prevent hook recursion and worktree contamination.
- **WHEN** external tools or harnesses are inspected,
- **THEN** read-only diagnostics MUST NOT mutate configuration or file system state.
