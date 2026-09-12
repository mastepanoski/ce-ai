---
title: "Documentation Technical Debt Diagnostic Engine & Probes"
category: "architecture"
date: "2026-09-12"
problem_type: "architecture"
tags:
  - doc-debt
  - openspec
  - workflow
  - doctor
  - solutions
  - diagnostics
  - tri-state
components:
  - commands::workflow
  - commands::doctor
  - state::state
applies_when: "Auditing or diagnosing repository documentation debt, stranded OpenSpecs, or solution library drift"
---

# Documentation Technical Debt Diagnostic Engine & Probes

## Context

In AI-augmented software development workflows, documentation is not merely human reference material—it forms the primary cognitive substrate for autonomous coding agents. When documentation drifts from code reality, agents suffer from compounding hallucination loops:
1. **Stranded OpenSpecs**: Completed features whose parent tasks were checked off but subtasks remained open, or whose commits already landed on `main`/`HEAD` with released versions, deceive agents into believing active work remains pending or attempting redundant reimplementations.
2. **Stale Pending Specs**: Abandoned change proposals linger indefinitely in `openspec/changes/`, polluting agent context and stalling workflow automation.
3. **Solution Library Drift**: Historical guides in `docs/solutions/` that reference deleted or renamed source files teach agents obsolete patterns and non-existent APIs.

To solve this, `ce-ai` incorporates a **Documentation Technical Debt Diagnostic Engine** (`probe_doc_debt`) that audits repository documentation integrity and provides actionable remediation guidance.

## Architecture & Core Design

### 1. Substrate Tri-State (`ProbeStatus<T>`)
Probes that depend on git cannot simply return a boolean or binary state (`Clean` vs `Debt`). In constrained environments (such as Docker containers or minimal CI runners where `.git` is stripped or `git` is absent), falsely reporting `Clean` creates silent false negatives.

The engine introduces a generic tri-state enum:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbeStatus<T> {
    Clean,
    Debt(T),
    Unknown,
}
```
When `git` is unavailable, git-dependent probes return `ProbeStatus::Unknown`. The summary formatter then reflects `[git: n/a]` instead of falsely certifying the repository as clean.

### 2. Probing Mechanisms

- **Probe 1: OpenSpec Git-Aware Desync (`probe_openspec_desync`)**:
  - **Parent Checkbox Inconsistency**: Detects parent tasks marked `[x]` while subtasks remain open `[ ]`.
  - **Git History Reconciliation**: Inspects `git log -n 50 --format=%s` to identify whether commits referencing the feature have already landed on `HEAD` or the base branch.
  - **Surpassed Version Verification**: Compares cited versions in `tasks.md` (e.g., `v1.22.1`) against the active version in `Cargo.toml`.

- **Probe 2: Stale Pending OpenSpecs Watchdog (`probe_stale_pending_openspecs`)**:
  - Evaluates days elapsed since the last git commit touching each change folder.
  - Falls back gracefully to directory file `mtime` when git history is inaccessible.
  - Compares against configurable staleness thresholds (default: 30 days), explicitly ignoring `archive/` and hidden directories.

- **Probe 5: Solution Library Drift & Linter (`probe_solution_drift`)**:
  - Traverses `docs/solutions/**/*.md`.
  - Validates required YAML frontmatter fields (`title`, `category` or `module`, `problem_type`, `tags`, and `applies_when`).
  - Extracts backticked source paths (`src/**/*.rs` and `tests/**/*.rs`), strips line anchors (`#L10-L20` or `:42`), and verifies that the referenced files exist on disk.

### 3. Turn-0 Delivery & Doctor Integration

- **Turn-0 Resumption**: `ce-ai workflow resume`, `status`, and `checkpoint` display a concise summary line:
  ```text
  doc debt: clean
  # or:
  doc debt: 1 stranded, 2 stale, 1 solution drift [git: n/a]
  ```
- **Doctor Diagnostic Advisories**: `ce-ai doctor` prints non-blocking `doctor-warn:` entries containing copy-pasteable remediation commands (`ce-ai archive <feature>`, `ce-ai archive <feature> --status "..."`, or `rm -rf openspec/changes/<feature>`). Warnings are advisory to prevent breaking routine developer flows.

### 4. Repository Overrides (`.ce-ai.json`)

Teams can tune thresholds per workspace via `.ce-ai.json`:
```json
{
  "doc_hygiene": {
    "stale_spec_days": 45,
    "check_solution_paths": true,
    "require_solution_frontmatter": true
  }
}
```

## Key Files & Implementation
- `src/commands/workflow.rs`: Core probe implementations (`probe_doc_debt`, `probe_openspec_desync`, `probe_stale_pending_openspecs`, `probe_solution_drift`) and tri-state models.
- `src/commands/doctor.rs`: Doctor health check integration and advisory warning emission.
- `src/state/state.rs`: `DocHygieneConfig` struct and workspace override merging.
- `docs/user-guide/doc-hygiene-and-debt-explained.md`: Newbie-friendly conceptual guide.
