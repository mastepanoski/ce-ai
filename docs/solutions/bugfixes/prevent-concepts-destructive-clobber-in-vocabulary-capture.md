---
title: "Preventing Destructive Clobber of CONCEPTS.md in Vocabulary Capture"
category: "bugfixes"
date: "2026-09-25"
tags:
  - concepts
  - ce-compound
  - vocabulary-capture
  - monotonic-accretion
  - doc-lint
  - anti-clobber
components:
  - scripts/validate-concepts.py
  - commands::workflow
  - commands::doc
  - commands::doctor
applies_when: "Running /ce-compound vocabulary capture, updating CONCEPTS.md, or auditing glossary accretion integrity"
problem_type: "bugfix"
---

# Preventing Destructive Clobber of CONCEPTS.md in Vocabulary Capture

## Problem
During Stage 6 (`ce-compound` Phase 2.4, "Vocabulary Capture"), AI agents extracting qualifying domain vocabulary into `CONCEPTS.md` frequently invoked full-file write operations (`Write` / `write_to_file`) with only the newly extracted terms. This caused silent, catastrophic data loss: dozens of previously established concepts were wiped out in a single turn.

Because doc hygiene validators (`validate-frontmatter.py`, `validate-doc-claims.py`, and `ce-ai doc lint`) historically audited only `docs/solutions/**/*.md`, this destructive clobber went unnoticed by automated CI gates. Furthermore, terminal reports hallucinated success (`CONCEPTS.md: updated — N added`) without empirical inspection of the git diff.

## Root Cause
1. **Missing Pre-Read & Tool Constraint**: The skill specification described `CONCEPTS.md` as accreting over time, but lacked an explicit operative directive requiring a `Read` immediately before composing edits, allowing agents to issue blind `Write` calls instead of targeted in-place modifications (`Edit` / `replace_file_content`).
2. **Absence of Monotonic Accretion Validator**: No mechanical guard verified that `CONCEPTS.md` headings only grew monotonically against git `HEAD`.
3. **Unchecked Reporting**: Reports printed agent declarations rather than diff measurements.

## Solution Architecture: Defense-in-Depth

The solution establishes a multi-layered defense to enforce monotonic accretion:

### 1. Portable Python Validator (`scripts/validate-concepts.py`)
A standalone Python 3 script (zero external dependencies) that:
- Resolves real paths using `os.path.realpath` to handle symlinks (such as macOS `/var` -> `/private/var`).
- Compares working-tree `CONCEPTS.md` against `git show HEAD:CONCEPTS.md`.
- Parses concept headings (`## `, `### `) and bold definition lists (`- **Term**:`), skipping preamble headers (`Concepts`, `Overview`, `Introduction`).
- Detects deliberate scrubs via `<!-- scrub: <Term> -->` or `<!-- retired: <Term> -->`.
- Fails with exit code 1 if any unscrubbed heading from `HEAD` is missing, printing the exact deleted terms and empirical diff counts (`N entries (was M, +X added, -Y removed)`).

### 2. Native Rust Diagnostic Probe & Strict Gate (`src/commands/workflow.rs` & `src/commands/doc.rs`)
- `probe_concepts_drift(repo_root, git_available)`: Integrates into `DocDebtReport`, surfacing `[concepts-clobber]` findings.
- `ce-ai doc lint`: Audits `CONCEPTS.md` alongside `docs/solutions/`. In `--strict` mode, exits with code 6 (`CeError::Verification`) if destructive shrinkage is detected.
- `ce-ai doctor`: Emits `doctor-warn` with actionable remediation guidance when concepts drift occurs.

### 3. Agent Operative Invariants (`AGENTS.md` & `SKILL.md`)
- Hard Invariant #12 and Constraint #10 in `AGENTS.md` mandate pre-reading and surgical edits.
- Phase 2.4 in `ce-compound/SKILL.md` (and `assembly.md`) enforces running `validate-concepts.py` before completing the run.

## Key Learnings & Prevention
1. **Symlink Traps in Git Operations**: When invoking `git rev-parse --show-toplevel` alongside temporary paths, always use `os.path.realpath` on both sides before computing relative paths; otherwise paths like `/var/folders` vs `/private/var/folders` fail resolution in `git show HEAD:<rel_path>`.
2. **Strict Semantic Accretion**: Glossaries must be treated as append-only registries by default; deliberate term pruning must require explicit annotations (`<!-- scrub: ... -->`) to prevent silent agent clobbering.
