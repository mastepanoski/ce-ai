# Specification: Adaptive Model Route Selection (System One Advisory Layer)

## Requirements

### R1: Logical Model Classes & Capability Mapping
- **WHEN** an agent slot or task requests dynamic model advice
- **THEN** the routing engine MUST resolve the target into one of three logical model classes: `fast`, `standard`, or `reasoning`.
- **WHEN** resolving a logical model class to an executable model identifier
- **THEN** the system MUST query the configured `models` catalog in `state.json` (`[decisions.routing.models]`).
- **WHEN** a logical model class is mapped in the catalog (e.g. `fast = "anthropic/claude-3-5-haiku"`)
- **THEN** that model identifier MUST be selected as the routed recommendation.

### R2: Deterministic Routing Policy Engine
- **WHEN** a task description is evaluated by the routing engine
- **THEN** it MUST construct a `DecisionRequest` querying four typed dimensions:
  1. `complexity`: Choice `["trivial", "moderate", "complex"]`
  2. `needs_reasoning`: Boolean
  3. `needs_large_context`: Boolean
  4. `risk`: Choice `["low", "medium", "high"]`
- **WHEN** `needs_reasoning == true` with `confidence >= reasoning_threshold_pct`, OR `complexity == "complex"`, OR `risk == "high"`
- **THEN** the routing engine MUST select the `reasoning` model class.
- **WHEN** `complexity == "trivial"`, `risk == "low"`, `needs_reasoning == false`, and `needs_large_context == false`
- **THEN** the routing engine MUST select the `fast` model class.
- **WHEN** neither `reasoning` nor `fast` conditions are met
- **THEN** the routing engine MUST select the `standard` model class.

### R3: Strict Override Precedence
- **WHEN** an explicit user override or `--model` flag is provided
- **THEN** the router MUST immediately return the explicit override, bypassing dynamic routing.
- **WHEN** a slot has a static model assignment in `state.model_assignments` and routing is not explicitly requested
- **THEN** the static model assignment MUST take precedence over adaptive routing.
- **WHEN** routing is evaluated
- **THEN** adaptive routing MUST NEVER silently overwrite explicit user slot configurations.

### R4: Graceful Degradation & Fallback Chain
- **WHEN** model routing is disabled (`enabled = false`), the provider encounters a network error, times out, or the budget is exhausted
- **THEN** the router MUST immediately fall back to the default configured model with exit code 0.
- **WHEN** the resolved model class is unconfigured in the catalog (e.g. `reasoning` is missing)
- **THEN** the router MUST degrade along the capability fallback chain:
  - Missing `reasoning` ➔ try `standard` ➔ try `fast` ➔ fallback default.
  - Missing `fast` ➔ try `standard` ➔ try `reasoning` ➔ fallback default.
  - Missing `standard` ➔ try `fast` ➔ try `reasoning` ➔ fallback default.

### R5: CLI Advisory Ergonomics (`ce-ai models route`)
- **WHEN** a user runs `ce-ai models route "<task description>"`
- **THEN** `ce-ai` MUST display human-readable output including recommended model class, resolved model identifier, and evaluation rationale.
- **WHEN** the `--json` flag is provided
- **THEN** `ce-ai` MUST output structured JSON containing `task`, `recommended_class`, `resolved_model`, `dimensions`, `rationale`, and `fallback_applied`.
- **WHEN** the `--verbose` flag is provided
- **THEN** `ce-ai` MUST display full decision scores, confidence percentages, and evaluated provider latency.

### R6: Doctor Diagnostics & Preset Configuration
- **WHEN** `ce-ai doctor` is executed
- **THEN** it MUST inspect `state.decisions.routing` and report whether adaptive routing is active, along with the mapped models for `fast`, `standard`, and `reasoning`.
- **WHEN** running `ce-ai decisions setup --preset recommended`
- **THEN** it MUST pre-seed sensible multi-provider model classes in the routing catalog (`fast`, `standard`, `reasoning`) and enable routing.

---

## Acceptance Criteria

1. **Deterministic Offline Tests**: Comprehensive unit tests verify all paths of the routing policy matrix using `MockDecisionProvider` without network calls.
2. **Override Precedence Invariant**: Unit tests prove that explicit overrides and static slot assignments always take precedence over adaptive routing.
3. **Graceful Degradation Invariant**: Simulated provider timeouts, network errors, or missing catalog entries gracefully return default models with exit code 0 and an explanatory diagnostic.
4. **CLI Ergonomics**: `ce-ai models route "fix typo in docs"` outputs `fast` recommendation, while `ce-ai models route "redesign distributed consensus"` outputs `reasoning`.
5. **Quality Gates**: Zero Clippy warnings (`cargo clippy --all-targets --all-features -- -D warnings`), formatted code (`cargo fmt --check`), 100% test pass (`cargo test`).
