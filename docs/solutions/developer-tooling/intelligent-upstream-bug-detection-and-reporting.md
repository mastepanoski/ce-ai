---
title: "Intelligent Upstream Bug Detection, Deduplication & Reporting with User Consent"
category: "developer-tooling"
date: "2026-09-26"
tags:
  - bug-reporter
  - deduplication
  - privacy-sanitization
  - user-consent
  - gh-cli
  - github-issues
components:
  - source::bug_reporter
  - commands::report_bug
  - commands::registry
applies_when: "Reporting internal ce-ai runtime crashes, unexpected command failures, or triage boundary enforcement"
problem_type: "best-practice"
---

# Intelligent Upstream Bug Detection, Deduplication & Reporting with User Consent

## Problem
In agentic coding workflows, developers and AI agents frequently encounter two distinct categories of issues:
1. **Host Project Domain Issues**: Business logic errors, failing unit tests, or framework configuration bugs in consumer repositories.
2. **Internal Tool Issues**: Panics, unhandled errors, harness synchronization failures, or CLI regressions inside `ce-ai` itself.

Previously, there was no structured mechanism for reporting internal `ce-ai` tool errors upstream:
- Developers and agents had to manually navigate to GitHub, locate the template, and fill in diagnostic fields.
- Submissions risked exposing proprietary secrets, local absolute paths, authentication tokens (`ghp_`, `sk-`, `Bearer`), or internal environment structures.
- Repetitive bug reports caused issue tracker noise when duplicate issues already existed in `mastepanoski/ce-ai`.
- Agents could confuse host application bugs with tool bugs, erroneously filing upstream issues for project-level domain test failures.

## Solution
Implemented the `ce-ai report-bug` command and underlying sanitization and deduplication engine:

1. **Strict Bug Triage Boundary**:
   - Codified in `AGENTS.md` (Invariant 13 and Rule 11) and `CONCEPTS.md`.
   - Host project bugs must be diagnosed and resolved locally using Compound Engineering workflows (`ce-debug` / `ce-work`).
   - Upstream bug reporting is strictly reserved for internal `ce-ai` binary errors, crashes, and harness sync failures.

2. **ISO/IEC 27001 Multi-Pass Privacy Sanitization Engine (`source::bug_reporter::sanitize_text`)**:
   - Replaces user home directories (`/Users/<user>` or `/home/<user>`) with `~` (supporting both direct and macOS `/private/` symlinks).
   - Replaces workspace roots with `<project-root>`.
   - Redacts authentication tokens (`ghp_*`, `github_pat_*`, `sk-*`, `Bearer`), private key PEM blocks, and `key=`/`secret=` key-value pairs with contextual markers like `[REDACTED_GH_TOKEN]`, `[REDACTED_OPENAI_TOKEN]`, `[REDACTED_PRIVATE_KEY]`.
   - Sanitizes `command_invoked`, `error_message`, and `title` to ensure zero token leakage across all fields.

3. **Intelligent Deduplication Search (`source::bug_reporter::search_upstream_issues`)**:
   - Extracts the first line/signature of the error message.
   - Queries `mastepanoski/ce-ai` via `gh issue list --search` to detect matching open or closed issues.
   - Displays matching issue numbers, titles, and links to prevent duplicate reports.

4. **Sovereignty & User Consent Gate**:
   - Never submits silently.
   - Interactive terminal prompt provides 4 explicit choices: `[1] Submit via gh CLI`, `[2] Browser URL fallback`, `[3] View sanitized draft`, `[4] Dismiss`.
   - Auto-confirmation `-y`/`--yes` supported for headless/scripted pipelines.
   - Formats pre-filled web submission URLs conforming to `.github/ISSUE_TEMPLATE/bug_report.yml`.
   - Non-interactive streams safely output the pre-filled web URL without blocking.

5. **Diagnostic Machine-Readable Output**:
   - `--dry-run` displays formatted draft markdown without network submission.
   - `--json` outputs the serialized `BugReportBundle` containing OS, architecture, version, target harness, and sanitized error logs.

## Verification
- Unit tests in `src/source/tests/bug_reporter.rs` test URL encoding, token/secret redaction, private key scrubbing, path anonymization, issue body formatting, web URL generation, and install instructions.
- Integration tests in `tests/cli.rs` test `--dry-run`, `--json`, path and token sanitization in real workspaces, and missing `gh` fallback with prefilled web URL.
- Monotonic accretion verified on `CONCEPTS.md` and `ce-ai doc lint --strict` passed.
