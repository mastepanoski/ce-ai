# Proposal: Intelligent Upstream Bug Detection, Deduplication & Reporting with User Consent

## Problem Statement
When developers or autonomous AI agents encounter issues while working in an adopted codebase:
1. **Triage Ambiguity (Host Project vs. `ce-ai` Tool)**:
   - Agents often mistake host application bugs (e.g. failing unit tests, application logic defects) for tool bugs, or conversely silently work around critical internal `ce-ai` crashes.
   - There must be a strict architectural boundary: **host project bugs are solved strictly via Compound Engineering** (`ce-debug` ➔ `ce-plan` ➔ `ce-work` ➔ `ce-compound`), whereas **internal `ce-ai` defects** (crashes, panics, unhandled `CeError` exit codes, harness sync errors) qualify for upstream reporting.
2. **Missing Upstream Feedback Loop**: Internal tool bugs frequently go unreported, causing maintainers to miss critical defects in prompt injection guards, state synchronization, or harness adapters.
3. **Issue Duplicate Noise**: Unfiltered reporting creates duplicate issues in `mastepanoski/ce-ai`.
4. **Data Privacy & ISO/IEC 27001 Compliance**: Automated or semi-automated issue drafts must guarantee **zero host project data leakage** (no host code, repository names, customer data, git remotes, or secrets).
5. **User Sovereignty & Consent**: The system must never send reports or create issues silently in the background; user consent is mandatory, dismissal must be frictionless, and `gh` CLI absence must be handled gracefully with onboarding steps and web URL fallbacks.

## Scope Boundaries

### In Scope
1. **Strict Triage Boundary**: Explicitly document and enforce routing: host application bugs route to Compound Engineering (`ce-debug`); internal `ce-ai` tool errors trigger the reporting workflow.
2. **Dedicated CLI Subcommand (`ce-ai report-bug`)**: Allows manual or scripted bug reporting with diagnostic compilation, sanitization, deduplication, and submission options.
3. **Upstream Deduplication**: Queries `mastepanoski/ce-ai` via `gh` or GitHub API to check for existing matching issues by error fingerprint; links to existing issues or drafts comments if relevant.
4. **Zero-Data-Leakage Sanitization (ISO 27001 / NIST AI RMF)**: Redacts tokens, bearer headers, API keys, private keys, user home paths (`~/`), and project paths (`<project-root>`).
5. **Interactive User Consent Gate**: Friendly interactive prompt with options to submit, review/edit draft, open browser URL, or dismiss.
6. **GitHub CLI (`gh`) Readiness & Web Fallback**: Probes `gh auth status`, provides platform-specific install/login instructions if missing/unauthenticated, and generates pre-filled web submission URLs (`https://github.com/mastepanoski/ce-ai/issues/new?title=...&body=...`).
7. **Operational Directives**: Updated `AGENTS.md` and `ce-debug` guidelines clarifying the host project vs internal tool boundary.

### Out of Scope
- Reporting host project business logic or application test failures upstream.
- Silent background telemetry or automatic issue creation without user consent.
- Direct write access to GitHub without user authorization.

## Risk Evaluation & Mitigation
- **Privacy & Leakage Risk**: Leakage of proprietary code or API credentials.
  *Mitigation*: Deterministic multi-pass sanitization (paths, tokens, credentials, repo names); diagnostic bundle strictly restricted to `ce-ai` binary metadata, OS, arch, harness, and sanitized error string.
- **Agent Interruption / Loop Risk**: Automated agents getting stuck in interactive prompts.
  *Mitigation*: Non-interactive streams (`!is_terminal()`) and CI environments automatically suppress interactive prompts; `CE_DISABLE_BUG_PROMPT=1` and `CE_NO_BUG_REPORT=1` provide permanent opt-outs.
- **GitHub API Rate Limiting**: Searching issues unauthenticated.
  *Mitigation*: Leverages `gh` CLI credentials when available, authenticated tokens via `CE_AI_GITHUB_TOKEN`, and graceful failover to web submission links.

## Success Criteria
- Clear, unambiguous domain separation between host bugs and `ce-ai` internal tool defects.
- Subcommand `ce-ai report-bug` compiles a sanitized diagnostic bundle conforming to `.github/ISSUE_TEMPLATE/bug_report.yml`.
- All tokens, secrets, private keys, and user home paths are 100% redacted in diagnostic bundles.
- Zero transmissions occur without explicit consent.
- `gh` CLI status is verified and actionable install guidance or web fallback URLs are provided.
- 100% test pass across unit and integration suites with zero clippy warnings.
