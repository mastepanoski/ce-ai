# Proposal: Expose Mid-Tier Model Slot for ce-code-review

## Problem Statement
In `compound-engineering-v3.24.0`, the `ce-code-review` skill introduced persona-level model tiering: three high-stakes reviewers (`correctness-reviewer`, `security-reviewer`, `adversarial-reviewer`) inherit the session model, while every other persona (~15+ sub-agent reviewers) is dispatched to "the platform's mid-tier model" to control cost and latency.

The upstream skill's documentation explicitly resolves this for Claude Code ("the Sonnet class") and Codex (conditional on explicit selector), but specifies no mechanism for OpenCode. In OpenCode, model identifiers require an explicit `provider/model` string; there is no implicit shorthand.

`ce-ai` manages OpenCode's per-agent model configuration in `opencode.json` via `CE_AGENT_SLOTS` and `ce-ai models set`, but currently only supports a single model per skill invocation (`ce-code-review` as a monolithic block). Consequently, OpenCode has no configured, discoverable slot for this internal dispatch stage. The orchestrating agent running `ce-code-review` under OpenCode is forced to either invent a model identifier or omit the override and fall back to the session model for all 15+ sub-agents, silently defeating the cost-tiering architecture.

## In-Scope
1. **Mid-Tier Slot Constant**: Define `ce-code-review-mid-tier` in `src/harness/agents.rs` alongside existing CE slots.
2. **Model Persistence**: Support `ce-ai models set --harness opencode ce-code-review-mid-tier <provider/model>` through the exact existing path (`apply_agent_model`, `state.set_model_assignment`, atomic write, append-only snapshot, no `variant` written).
3. **Distinguished List Output**: Display the `ce-code-review-mid-tier` slot in `ce-ai models list` clearly distinguished as a tiering sub-slot rather than an independent top-level workflow stage.
4. **Doctor Health Diagnostic**: Add a non-blocking informational diagnostic (`doctor-info:`) to `ce-ai doctor` when OpenCode is configured with a model for `ce-code-review` but lacks `ce-code-review-mid-tier`, naming the exact command to configure it.
5. **Strict Opt-In Semantics**: Never fabricate or write default model values when unset, preserving the invariant from Issue #111.
6. **Empirical Test Coverage**: Add unit tests in `src/commands/tests/models.rs` and `src/commands/tests/doctor.rs`.

## Out-of-Scope
- Modifying, intercepting, or executing the compound-engineering plugin's internal runtime dispatch logic.
- Editing files inside the compound-engineering plugin repository.
- Introducing multi-tier models for other skills (`ce-doc-review`, `ce-work`) at this time (deferred until needed upstream).

## Risk Evaluation & Mitigation
- **Risk (Fabricated / Guess Defaults)**: Setting a default mid-tier model automatically risks selecting a model the user lacks credentials or quota for.
  - *Mitigation*: Strictly opt-in; no default is ever written unless the user explicitly invokes `ce-ai models set`.
- **Risk (Breaking Callers of CE_AGENT_SLOTS)**: Extending `CE_AGENT_SLOTS` could affect existing callers (e.g., config assignments, sync, doctor drift, TUI).
  - *Mitigation*: Audit every caller with CodeGraph and unit tests to ensure extending the array to 7 items provides consistent discovery and cleanup across TUI, sync, and drift calculation.
- **Risk (Doctor False Positives / Blocked CI)**: Flagging missing mid-tier slots as errors would break developer workflows or strict CI gates.
  - *Mitigation*: Emit as an advisory `doctor-info:` diagnostic only, never as a blocking finding in `findings`.
