# Proposal: Work Readiness & Verification Advisory

## Problem Statement

Across both Organic Driven Development (ODD) and Compound Engineering (7-Stage FSM) workflows in `ce-ai`, developers and AI agents need fast, objective feedback on whether a unit of work is truly ready for the next stage or ready to close:

1. **Semantic Gaps in Deterministic Quality Gates**:
   - Deterministic checks (`cargo test`, `cargo clippy`, file existence) can verify that code compiles and existing tests pass. However, they cannot assess semantic completeness:
     - Did the implementation actually satisfy all checklist items and stated goals, or only a subset?
     - Did the change respect specified task guardrails and constraints?
     - Has an informal ODD task accumulated enough scope, architectural footprint, or risk that it warrants graduating to a formal OpenSpec specification (`ce-ai graduate`)?
     - Are tests semantically sufficient for the newly introduced behavior?
2. **Cognitive Overhead and Premature Progression**:
   - Without advisory feedback, agents frequently jump from implementation to shipping prematurely or close tasks with incomplete documentation or unverified edge cases.
3. **Separation of Concerns**:
   - Evaluating readiness semantically must not weaken deterministic security guarantees or bypass FSM transition gates. The Decision Engine (System 1) provides structured advisory telemetry; the deterministic engine (`gate.rs`, `workflow.rs`) retains exclusive authorization over actual state mutations and transitions.

Issue [#386](https://github.com/mastepanoski/ce-ai/issues/386) introduces **Work Readiness & Verification Advisory** as Phase 5 of the Pluggable Decision Engine epic: a fast, structured advisory layer evaluating semantic readiness across both ODD tasks and CE stages with configurable integer-percentage thresholds and fail-safe non-blocking degradation.

---

## In-Scope

1. **Two Workflow Modes Support**:
   - **Organic Driven Development (ODD)**:
     - `dod_satisfied`: Estimates if Definition of Done items are satisfied by the current diff and working tree state.
     - `guardrails_respected`: Estimates if task constraints were honored without regression or boundary violation.
     - `graduation_recommended`: Recommends whether task complexity or scope creep warrants graduating from ODD to formal OpenSpec via `ce-ai graduate`.
     - `ready_to_close`: High-level advisory signal for closing the active task.
   - **Compound Engineering (CE 7-Stage FSM)**:
     - Planning (Stage 2/3): `requirements_clear`, `scope_defined`, `risks_identified`.
     - Implementation (Stage 4 / WorkTdd): `requirements_addressed`, `implementation_complete`, `tests_present`.
     - Verification / Review (Stage 5/6): `tests_sufficient`, `review_findings_resolved`, `documentation_complete`.
2. **Readiness Evaluation Primitives (`src/decisions/readiness.rs`)**:
   - `ReadinessStatus`: `Ready` (composite >= 80%), `Warning` (composite >= 60%), `NotReady` (composite < 60%).
   - `ReadinessDimensionScore`: dimension name, confidence, passed indicator.
   - `ReadinessEvaluationResult`: target feature/task, workflow mode, composite score, dimension breakdown, status, advisory notes, and latency.
3. **Configuration Schema in `state.json`**:
   - `ReadinessConfig` and `ReadinessThresholds` (`ready_pct: u32 = 80`, `warning_pct: u32 = 60`) with integer percentages ensuring `State` `Eq` trait compliance.
4. **Advisory Non-Blocking Invariant**:
   - Readiness signals are strictly advisory; they inform developers and agents in CLI reports (`ce-ai decisions check-readiness`, `ce-ai workflow status`) without returning non-zero exit codes or blocking progress.
5. **Fail-Open / Non-Disruptive Degradation**:
   - Provider outages, timeouts, or circuit breaker trips fall back cleanly to deterministic status reporting (exit code 0).
6. **CLI Diagnostics & Integration**:
   - `ce-ai decisions check-readiness [--feature <name>] [--task <text>] [--stage <n>] [--json] [--verbose]`.
   - Integration in `ce-ai decisions setup` presets (`recommended`, `shadow`, `local`) and `ce-ai doctor` diagnostic health probes.

---

## Out-of-Scope

1. **Mandatory Transition Blocking**:
   - Readiness advisory does NOT block git commits or fail builds; deterministic gates (`gate.rs`, CI matrix) handle blocking requirements.
2. **Automated Code Fixing**:
   - The engine assesses readiness and emits recommendations; it does not automatically generate patch code.
3. **External Issue Tracker Polling**:
   - Evaluation uses local working tree diffs, task markdown files (`odd/tasks/` or `openspec/changes/`), and state files; it does not make outbound requests to Jira, Linear, or GitHub Issues during evaluation.

---

## Risk Evaluation

| Risk | Likelihood | Impact | Mitigation Strategy |
| :--- | :---: | :---: | :--- |
| **Flaky or Noisy Readiness Advice** | Low | Low | Multi-dimensional scoring surfaces individual dimension confidences rather than an opaque pass/fail; strictly advisory mode. |
| **Provider Latency Overhead** | Low | Medium | Read-only evaluation caching, short timeouts (1000ms), and 0ms fallback when disabled. |
| **State Mutation or Accidental Transition** | Zero | High | Architectural invariant: `ReadinessEvaluator` receives immutable context references and has no write access to `state.json`. |

---

## Success Criteria

1. `cargo test decisions::readiness` covers ODD and CE readiness evaluations, threshold mapping, and fallback behavior with 100% pass rate.
2. `ce-ai decisions check-readiness` outputs clear human-readable indicators (`✓ Ready`, `△ Needs Attention`, `⚠ Incomplete`) and `--json` structured metrics.
3. `ce-ai doctor` probe surfaces readiness engine configuration status.
4. Presets (`recommended`, `shadow`, `local`) configure readiness defaults.
5. Zero warnings in `cargo clippy --all-targets --all-features -- -D warnings`.
