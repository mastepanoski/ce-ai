# Proposal: Upstream Coupling Remediation & CE Contract Alignment

## Problem Statement
In Section 4 of the official feedback from the upstream Compound Engineering maintainer (`EveryInc/compound-engineering-plugin`), seven concrete coupling divergences and bugs were identified in `ce-ai`:
1. **Frontmatter Schema Mismatch:** `ce-ai` required `title`, `tags`, and `applies_when` for all solution documents. Upstream `schema.yaml` mandates `module`, `date`, `problem_type`, `component`, and `severity`, while `tags` and `applies_when` are optional (`applies_when` is knowledge-track only).
2. **Component Field Parsing:** `ce-ai` only parsed `components`, missing upstream's `component` and `related_components` fields.
3. **Hardcoded Docs Root:** `ce-ai` hardcoded `docs/`, ignoring the upstream configurable `docs_root` in `.compound-engineering/config.yaml`.
4. **Brainstorm Discovery Path:** Upstream writes new ideation artifacts as requirements-only plans in `docs/plans/`, treating `docs/brainstorms/` as legacy. `ce-ai` only checked `docs/brainstorms/` and `docs/ideation/`.
5. **Dead-Path Language Scoping:** `ce-ai`'s dead-path probe was restricted to `.rs` files under `src/` and `tests/`, failing to validate paths in multi-language codebases.
6. **Pi Extension Filename Collision:** `ce-ai` materialized its Pi hook at `.pi/extensions/compound-engineering.ts`, colliding with upstream CE's own extension entrypoint.
7. **Overly Broad `ce-debug` Gate Exemption:** Substring matching (`clean.contains("ce-debug")`) allowed false positives to bypass the write gate.

## In-Scope
- Implement dynamic `docs_root` discovery by parsing `.compound-engineering/config.yaml` with a default of `docs/`.
- Align solution frontmatter validation in `src/commands/workflow.rs` with upstream `schema.yaml` requirements.
- Parse both singular `component` and `related_components` in `src/commands/doc.rs`.
- Detect modern requirements-only plans (`*-requirements.md`, `*-requirements.html`) under `<docs_root>/plans/` as ideation artifacts in `src/commands/workflow.rs`.
- Expand dead-path verification to all common source extensions (`.rs`, `.ts`, `.js`, `.py`, `.go`, `.rb`, etc.).
- Rename the Pi companion extension to `ce-ai-companion.ts`, with migration/cleanup of legacy files.
- Restrict `ce-debug` gate exemption to exact structured command/task prefixes.
- Update tests to verify all 7 remediations.

## Out-of-Scope
- Full replacement of file synchronization with fleet native installers (deferred to Phase 2, `ce-ai fleet`).

## Success Criteria
- All 7 coupling bugs resolved cleanly without breaking existing tests.
- `cargo test` passes 100% across unit and integration tests.
- `ce-ai doc lint --strict` validates upstream-compatible solution documents without false rejections.
- OpenSpec lifecycle tail validated via `scripts/validate-tasks-tail.py`.
