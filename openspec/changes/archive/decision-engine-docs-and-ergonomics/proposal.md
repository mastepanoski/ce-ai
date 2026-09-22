# Proposal: Decision Engine Documentation & CLI Ergonomics

## Problem Statement

The **Pluggable Decision Engine** (System 1 micro-decision layer) in `ce-ai` provides rapid, structured, probabilistic classifications (via Jev/TypeSafe AI or an offline Mock provider) to inform model routing, dynamic skill selection, risk evaluation (`check-risk`), and work readiness verification (`check-readiness`).

However, two major gaps exist:
1. **Missing User Guide**: Although implemented and tested across multiple archived change packages (`pluggable-decision-engine`, `risk-aware-execution`, `readiness-advisory`), there is no dedicated documentation in `docs/user-guide/` explaining how the engine works, how it safely falls back to deterministic rules, or how to operate `ce-ai decisions`. Furthermore, `README.md` lacks a reference row for `ce-ai decisions`.
2. **Ergonomic Inconsistencies in the CLI**:
   - **Preset vs. Mode Conceptual Confusion**: `ce-ai decisions setup --preset` accepts `recommended`, `shadow`, or `local`, while runtime execution modes are `off`, `shadow`, and `active`. The name `shadow` is overloaded as both a preset and a mode, while `recommended` maps to `active` (with Jev provider) and `local` maps to `active` (with Mock provider).
   - **No Way to Disable via CLI**: There is no `--preset off` (or `disabled`) in `ce-ai decisions setup`. Users wanting to turn off the engine must manually edit `state.json` (`"enabled": false` or `"mode": "off"`).
   - **No Granular Mode Switcher**: Users cannot toggle between `active`, `shadow`, and `off` without either rerunning full setup (which overwrites budget and model configs) or modifying `state.json` directly.

## In-Scope

1. **CLI Ergonomic Improvements**:
   - Add `off` (and `disabled`) preset support to `ce-ai decisions setup --preset off`.
   - Add `ce-ai decisions mode [active|shadow|off]` subcommand:
     - When called without arguments (`ce-ai decisions mode`), it prints the current operational mode and whether the engine is enabled.
     - When called with an argument (`ce-ai decisions mode active`), it safely updates the mode in `state.json` using atomic writes (`write_atomic`), enabling the engine if it was off.
2. **Comprehensive User Documentation**:
   - Author `docs/user-guide/decision-engine-guide.md` adhering strictly to Diátaxis (How-to & Reference) and the project style guide (`docs/references/docs-styling.md`).
   - Clearly demystify Presets vs. Operational Modes, circuit-breaker fallbacks, deterministic security precedence, and subcommand usage.
   - Update `README.md` to link `decision-engine-guide.md` in both the Command Table and Documentation Map while strictly preserving the `<= 100` lines invariant.
3. **Automated Testing**:
   - Add CLI integration tests in `tests/cli.rs` validating `ce-ai decisions setup --preset off` and `ce-ai decisions mode [active|shadow|off]`.
   - Verify unit test suite, clippy `-D warnings`, and formatting.

## Out-of-Scope

- Modifying core provider logic (`JevProvider`, `MockDecisionProvider`).
- Changing the schema or serialization structure of `DecisionsConfig` in `state.json`.
- Modifying prompt strings sent to external decision APIs.

## Risk Evaluation & Mitigation

- **Risk: Breaking Existing Presets or State**:
  - *Mitigation*: The existing presets (`recommended`, `shadow`, `local`) remain unchanged. `off` is purely additive. Modifying mode via `ce-ai decisions mode` uses `write_atomic` and validates valid enum values (`DecisionMode::parse`).
- **Risk: README line count overflow**:
  - *Mitigation*: Consolidate adjacent command table rows to ensure `README.md` remains strictly `<= 100` lines as enforced by AGENTS.md.

## Success Criteria

1. `ce-ai decisions setup --preset off` cleanly disables evaluations and reports the disabled status.
2. `ce-ai decisions mode` queries and toggles `active`, `shadow`, and `off` reliably.
3. `docs/user-guide/decision-engine-guide.md` provides clear, runnable commands and distinct Diátaxis structure.
4. `README.md` is `<= 100` lines and all tests pass with zero clippy warnings.
