---
title: "Solution Library Drift Repair & Frontmatter Schema Normalization"
category: architecture
problem_type: architectural_refactor
tags: [solutions, frontmatter, schema, drift, doctor, documentation]
applies_when: "When maintaining, auditing, or adding solution documents to docs/solutions/ to ensure zero diagnostic drift."
---

# Solution Library Drift Repair & Frontmatter Schema Normalization

## Problem
As `ce-ai` evolved through rapid iterations and architectural refactorings (such as decomposing the monolithic TUI module into `src/tui/`), historical solution documents in `docs/solutions/` accumulated dead code references. When Increment 2 introduced the diagnostic engine with `probe_solution_drift`, 70 out of 74 solution documents failed frontmatter schema validation (missing `title`, `applies_when`, or `problem_type`), and 9 solution files referenced non-existent source files.

## Solution Architecture & Execution

1. **Frontmatter Schema Normalization**:
   - Every solution document in `docs/solutions/` conforms strictly to the standardized 5-field schema:
     - `title`: Extracted from `# <Title>` headings and cleaned of redundant prefixes.
     - `category` (or `module`): Derived from the directory hierarchy (`architecture`, `bugfixes`, `build`, etc.).
     - `problem_type`: Categorized by issue classification (`architectural_refactor`, `bugfix`, `build_error`, `config_error`, `distribution`, `security_hardening`, `test_failure`, `workflow_issue`).
     - `tags`: Bounded set of 3-7 searchable lowercase domain tags.
     - `applies_when`: One-sentence semantic trigger providing search agents with clear indexing context.

2. **Dead Path Remediation**:
   - Pointed all references to decomposed modules to their active locations (or historical tags `<src/tui/app.rs>` / `<src/tui/render.rs>`).
   - Updated references to superseded harness modules to active dispatchers (`src/harness/mod.rs`).
   - Clarified sub-module test paths to point to existing test files (`src/harness/tests/mod_tests.rs`).

3. **Empirical Verification**:
   - `ce-ai doctor` runs `probe_solution_drift` across all 74 files, returning `ProbeStatus::Clean`.
   - `workflow status` reports 0 dead links and 0 missing frontmatter errors.

## Key Principles & Guardrails
- **Keep Frontmatter Valid on Creation**: When agents create new solutions via `/ce-compound`, always populate all 5 required frontmatter keys (`title`, `category`, `problem_type`, `tags`, `applies_when`).
- **Live Path Verification**: Never hardcode dead source paths in backticks; if citing a former/historical file, qualify it or cite the current modular path.
