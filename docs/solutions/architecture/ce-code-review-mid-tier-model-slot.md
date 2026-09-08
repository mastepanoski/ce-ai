---
title: "Exposing Mid-Tier Model Slots for Skill Persona Dispatch in OpenCode"
category: "architecture"
date: "2026-09-08"
tags:
  - models
  - opencode
  - persona-tiering
  - ce-code-review
  - doctor
components:
  - harness::agents
  - commands::models
  - commands::doctor
applies_when: "Configuring multi-tier or persona-level sub-agent dispatch models in OpenCode, distinguishing internal tiering slots from workflow stage slots, or diagnosing cost-tiering model gaps"
---

# Exposing Mid-Tier Model Slots for Skill Persona Dispatch in OpenCode

## Problem & Context
Upstream skills like `ce-code-review` (in `compound-engineering-v3.24.0`) dispatch specialized reviewer personas at different model tiers: three high-stakes reviewers (`correctness-reviewer`, `security-reviewer`, `adversarial-reviewer`) inherit the session model, while ~15+ sub-agents are dispatched to a mid-tier model to reduce costs.

While Claude Code ("Sonnet class") and Codex have documented conventions, OpenCode requires explicit `provider/model` strings. Previously, `ce-ai` only supported monolithic per-skill slots in `CE_AGENT_SLOTS`, forcing OpenCode orchestrators to either guess a model or omit overrides and run all 15+ sub-agents on expensive session models.

## Architectural Decision
1. **Extend `CE_AGENT_SLOTS` with `CODE_REVIEW_MID_TIER_SLOT` (`ce-code-review-mid-tier`)**:
   - Rather than creating a fragmented separate array that callers might omit, `CE_AGENT_SLOTS` was extended to 7 elements, ensuring `sync`, `tui`, uninstall cleanup, and drift detection handle the slot uniformly.
   - `CE_AGENT_STAGE_SLOTS` was introduced to preserve the distinction of the 6 primary workflow stages, with `is_tier_slot` and `is_stage_slot` predicates.
2. **Distinguished Hierarchy in `models list`**:
   - `models list` renders tiering slots nested beneath their parent skill (`  └─ mid-tier (ce-code-review-mid-tier): <model>`) when the parent is present, and with explicit `(mid-tier sub-slot)` labeling when standalone.
3. **Non-Blocking Doctor Diagnostic**:
   - `ce-ai doctor` inspects OpenCode model configuration: if `ce-code-review` has an assigned model but `ce-code-review-mid-tier` does not, it emits a non-blocking `doctor-info:` notice detailing the cost risk and naming the exact `ce-ai models set --harness opencode ce-code-review-mid-tier <provider/model>` command.
4. **Strict Opt-In Invariant**:
   - Consistent with Issue #111, `ce-ai` never fabricates or silently writes default models.
