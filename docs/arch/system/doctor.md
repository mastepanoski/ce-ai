---
title: "Doctor Health Diagnostic & Compliance Engine"
domain: doctor
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: Doctor Health Diagnostic & Compliance Engine

## 1. Overview & Architectural Boundaries

`ce-ai doctor` is the centralized diagnostic and health auditing command. It evaluates environment readiness, companion tools, branch protection compliance, documentation technical debt, and harness integrity without performing disruptive writes.

## 2. Capabilities & Requirements

### R1. Non-Blocking Advisory Reporting
WHEN `ce-ai doctor` identifies non-fatal anomalies (e.g. unarchived completed changes, stale specs, marketplace divergence, solution path drift)  
THEN the command MUST emit structured `doctor-warn:` lines with actionable remediation commands and exit with code 0 (`Ok(())`), preventing unexpected CI or build breakage.

### R2. Substrate Tri-State Reporting
WHEN evaluating substrate-dependent checks (e.g. Git branch checks, commit logs)  
THEN probes MUST return `ProbeStatus::Clean`, `ProbeStatus::Debt(findings)`, or `ProbeStatus::Unknown` (`[git: n/a]`), never falsely asserting `Clean` when underlying tools are missing.

### R3. Documentation Technical Debt Probes
WHEN scanning repository documentation  
THEN `doctor` MUST evaluate:
1. Desynchronized/landed OpenSpec changes lingering in `openspec/changes/`.
2. Stale pending changes untouched past `stale_spec_days` threshold (default 21 days).
3. Solution library drift in `docs/solutions/` (dead code paths and missing frontmatter).
4. Uncompacted archive packages exceeding `archive_compaction_threshold` (default 30).

### R4. Security & Companion Tool Auditing
WHEN probing tools  
THEN `doctor` MUST verify status for `codegraph`, `context7`, `engram`, `rtk`, and GitHub token presence.

## 3. Data Models & CLI Contracts

- `DocDebtReport`: aggregated tri-state documentation debt report.
- Finding types: `OpenSpecDesyncFinding`, `StalePendingFinding`, `SolutionDriftFinding`, `ArchiveCompactionFinding`.
- CLI command: `ce-ai doctor [--json]`.

## 4. Invariants & Operational Boundaries

- Diagnostic probes MUST be read-only (zero writes to repository or state during standard `doctor` execution).
