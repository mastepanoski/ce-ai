# Design: Solution Library Drift Repair & Frontmatter Schema Normalization

## Schema Contract for `docs/solutions/**/*.md`

Every solution file in `docs/solutions/` MUST begin with a valid YAML frontmatter block adhering to:

```yaml
---
title: "<Concise Title Describing the Problem & Solution>"
category: "<Subsystem or Domain, e.g. architecture, harness, commands, distribution>"
problem_type: "<One of: bugfix, architectural_refactor, build_error, config_error, security_hardening, test_failure, workflow_issue>"
tags: [<list of 3-7 lowercase keywords>]
applies_when: "<One-sentence trigger condition specifying when an agent or developer should consult this solution>"
---
```

### Key Normalization Rules
1. **`title`**:
   - Extracted from the first `# <Title>` heading in the document, stripped of `# Solution:`, `# Bugfix:`, `# Issue #...` prefixes, and title-cased.
2. **`category` / `module`**:
   - Preserved if already present (`module` satisfies `probe_solution_drift`).
   - If missing, derived from the relative directory name (`architecture` -> `architecture`, `bugfixes` -> `bugfixes`, `distribution` -> `distribution`, etc.).
3. **`problem_type`**:
   - Preserved if present.
   - If missing, mapped from category:
     - `bugfixes/`, `code-fixes/`, `logic-errors/` -> `bugfix`
     - `architecture/` -> `architectural_refactor`
     - `build-errors/` -> `build_error`
     - `config-errors/` -> `config_error`
     - `distribution/` -> `distribution`
     - `security/` -> `security_hardening`
     - `test-failures/` -> `test_failure`
     - `workflow-issues/` -> `workflow_issue`
4. **`tags`**:
   - Preserved if present.
   - If missing, populated with category and key terms derived from filename.
5. **`applies_when`**:
   - Generated based on the document's problem description, providing a clear contextual trigger (e.g. `"Encountering Kimi Code CLI marketplace plugin divergence or managing orphan managed trees."`).

## Dead Path Resolution Matrix

| Solution File | Dead Reference | Root Cause | Remediation Target |
| :--- | :--- | :--- | :--- |
| `architecture/extract-inline-unit-tests-2026-08-28.md` | `tests/mod_tests.rs` | Refactored into domain-scoped integration tests | `tests/cli.rs` |
| `architecture/grok-native-harness-adapter.md` | `src/harness/generic_json.rs` | Superseded by native harness implementations | `src/harness/mod.rs` |
| `architecture/kimi-adapter-audit-refinements.md` | `src/harness/generic_json.rs` | Superseded by native harness implementations | `src/harness/mod.rs` |
| `architecture/proactive-workflow-observability-fsm-tui-sync-watcher.md` | `src/tui.rs` | Decomposed into `src/tui/` module | `src/tui/app.rs` / `src/tui/` |
| `architecture/refactor-transversal-maintainability-2026-08-27.md` | `src/tui.rs` | Historical narrative of monolith decomposition | `src/tui/` |
| `architecture/tui-paridad-y-estabilidad-2026-08-26.md` | `src/tui.rs:86` | Historical narrative of TUI refactor | `src/tui/app.rs` |
| `backup-restore-management-and-point-in-time-recovery.md` | `src/tui.rs` | Pointing to TUI backup views | `src/tui/app.rs` |
| `config-errors/model-assignment-drift-reconciliation-and-health-probe.md` | `src/tui.rs` | Pointing to TUI dashboard navigation | `src/tui/app.rs` |
| `multi-harness-propagation-and-sync-verification.md` | `src/tui.rs` | Pointing to multi-harness TUI status | `src/tui/app.rs` |

## Atomic Repair Workflow

1. Execute Python normalization script:
   - Reads each solution file.
   - Parses existing YAML frontmatter or creates new frontmatter block.
   - Enriches missing keys while preserving existing fields.
   - Replaces dead source paths in the markdown body.
   - Writes back atomically.
2. Run `ce-ai doctor` via `cargo run -- doctor` to verify zero `doctor-warn: solution` warnings.
3. Run `cargo test` to ensure all existing diagnostic and workflow unit/integration tests pass.
