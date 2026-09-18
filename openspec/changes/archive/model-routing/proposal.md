# Proposal: Adaptive Model Route Selection (System One Advisory Layer)

## Problem Statement

In AI coding assistant workflows, different tasks demand vastly different degrees of reasoning and context depth:
- **Trivial extraction and formatting tasks** (e.g. regex extraction, JSON linting, simple doc comments) can be solved faster and for a fraction of a cent by lightweight models (`fast` class, e.g. Gemini 2.0 Flash, Claude 3.5 Haiku).
- **Standard feature work and localized bugfixes** require solid coding acumen and standard context windows (`standard` class, e.g. Claude 3.5 Sonnet, GPT-4o).
- **High-complexity architecture refactors and multi-file debugging** demand deep chain-of-thought and large context reasoning (`reasoning` class, e.g. Claude 3.7 Sonnet Thought, o3-mini, DeepSeek-R1).

Today, `ce-ai` relies primarily on static model assignments configured per agent slot (`ce-brainstorm`, `ce-plan`, `ce-work`). While static assignments are predictable, they are rigid:
1. Running heavy reasoning models for simple tactical edits incurs unnecessary latency (10s+) and burns budget.
2. Running lightweight models for complex tasks risks hallucination or plan failure, requiring human rework.
3. Users and harnesses lack a fast (< 100ms), low-cost (< $0.001) mechanism to dynamically evaluate task complexity and recommend an optimal model class.

With the Pluggable Decision Engine foundation now in place ([#382](https://github.com/mastepanoski/ce-ai/issues/382)), `ce-ai` can leverage fast System 1 evaluations (via Jev or offline mocks) to classify task requirements and recommend model routing dynamically without adding runtime LLM overhead.

## In-Scope

1. **Logical Model Classes (`fast`, `standard`, `reasoning`)**:
   - Decouple routing from specific vendor model identifiers through abstract capability tiers.
   - Configurable mapping in `state.json` (`[decisions.routing.models]`): `fast`, `standard`, `reasoning`.

2. **Deterministic Routing Policy Engine**:
   - Structured `DecisionRequest` questions:
     - `complexity`: `choice` (`trivial`, `moderate`, `complex`)
     - `needs_reasoning`: `boolean` (deep architectural planning or non-obvious debugging)
     - `needs_large_context`: `boolean` (massive codebase context required)
     - `risk`: `choice` (`low`, `medium`, `high`)
   - Transparent, deterministic mapping from decision answers to target model class.
   - Configurable confidence thresholds (e.g. `reasoning_threshold_pct = 75`).

3. **Strict Precedence Hierarchy**:
   - Enforce non-negotiable override order:
     ```text
     explicit CLI / user override
             ↓
     agent-specific static assignment (e.g. ce-ai models set)
             ↓
     adaptive model routing (if enabled)
             ↓
     existing harness default model
     ```
   - Invariant: Adaptive routing must **never** silently override an explicit model specification.

4. **Graceful Degradation & Fallback Chain**:
   - If routing is disabled, unconfigured, or the decision provider times out or trips the circuit breaker, immediately fall back to the default configured model (exit code 0).
   - If a selected model class is unconfigured (e.g. `reasoning` is empty), fall back along the capability chain (`reasoning` ➔ `standard` ➔ `fast` ➔ default).

5. **CLI & Diagnostic Ergonomics**:
   - `ce-ai models route "<task description>" [--json] [--verbose]`: Interactive CLI for testing and querying routing recommendations.
   - `ce-ai doctor`: Inspects routing configuration and reports mapped model classes.
   - Presets updated in `ce-ai decisions setup --preset recommended` to pre-seed sensible multi-provider model classes.

## Out-of-Scope

1. **Real-Time Token Usage Metering & Live Cost Dashboard**: Tracked separately under Issue [#387](https://github.com/mastepanoski/ce-ai/issues/387).
2. **Dynamic Skill Injection**: Tracked under Issue [#384](https://github.com/mastepanoski/ce-ai/issues/384).
3. **Automatic Live Harness Execution Interception**: Routing provides the advisory resolution API; interception hooks are layered per harness.

## Risk Evaluation & Mitigation

- **Risk 1: Flaky network or slow provider latency stalls model dispatch.**
  - *Mitigation:* Uses the Decision Engine's strict timeout (default: 1,000ms) and circuit breaker. If evaluation takes longer than the timeout, immediate fallback to the standard/default model.
- **Risk 2: Misclassification assigns an underpowered model to a subtle task.**
  - *Mitigation:* Conservative thresholding (bias towards `standard` on moderate ambiguity) and strict respect for explicit agent slot overrides.
- **Risk 3: Model identifiers become obsolete across vendor releases.**
  - *Mitigation:* Logical model classes allow users to swap underlying model strings in configuration without altering routing logic.

## Success Criteria

1. 100% offline testability: `MockDecisionProvider` can simulate any complexity/reasoning classification deterministically.
2. `ce-ai models route "<task>"` responds in < 100ms when using Jev and < 5ms when using Mock.
3. Zero regressions in existing `ce-ai models set`, `list`, or `profile` workflows.
4. Full adherence to `AGENTS.md` DoD: 0 Clippy warnings, 100% test pass, strict exit codes.
