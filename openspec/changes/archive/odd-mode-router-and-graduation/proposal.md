# Proposal: Adaptive Turn-0 Mode Router & Graduation Bridge (ODD Fast-Path vs OpenSpec Compound)

## Problem Statement

`ce-ai`'s 7-stage Compound Engineering workflow (`Ideation ➔ OpenSpec ➔ Plan ➔ Work ➔ Verify ➔ Compound ➔ Ship`) and automated gate enforcement (`src/commands/gate.rs`) guarantee architectural integrity, compliance (ISO/IEC 27001, ISO 42001, NIST AI RMF), and cryptographic drift control across multi-agent harnesses.

However, requiring the full 5-file OpenSpec change package (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`) for every task creates substantial friction on everyday tactical changes:
1. **Ceremony Overkill on Minor Tasks:** A 15-line bugfix, typo fix, or routine chore requires generating multiple formal specification documents before writing a single line of code.
2. **Cognitive Overhead & Token Consumption:** Upfront spec-driven development (SDD) on small issues burns context tokens and risks specification drift when exploratory discovery during debugging invalidates premature prose.
3. **Evolution Towards Organic Driven Development (ODD):** Gentle AI demonstrated that everyday tactical work thrives on a lightweight triad: **Problem Statement**, **Guardrails & Invariants**, and **Definition of Done (DoD)**. While discarding OpenSpec entirely would destroy `ce-ai`'s living domain contracts and release auditability, `ce-ai` currently lacks a native fast-path to accommodate this lighter organic pattern.
4. **The Missing Graduation Bridge:** When an organic exploratory spike or bugfix unexpectedly grows into a structural architectural change (> 200 LOC), there is no automated mechanism to promote the lightweight notes into a formal OpenSpec change package without manual re-authoring or loss of progress.

This proposal introduces an adaptive, deterministic **Turn-0 Mode Router** and **Graduation Bridge** in `ce-ai`. Tactical tasks execute with zero ceremony under ODD (`odd/tasks/<feature>.md`), while architectural features follow the formal 7-stage Compound Engineering lifecycle. The graduation bridge (`ce-ai workflow graduate <feature>`) mechanically promotes organic tasks into formal OpenSpec packages without dual-tracking or loss of progress.

## In-Scope

1. **Domain Models & State Extensions:**
   - Define `ExecutionMode` enum (`Auto`, `Organic`, `Compound`) in `src/state/state.rs` with serde serialization and string parsing (`odd`, `organic`, `compound`, `ce`, `auto`).
   - Extend `WorkflowState` with `pub execution_mode: Option<ExecutionMode>` in a backward-compatible manner.
2. **Turn-0 Mode Router (`probe_execution_mode`):**
   - Implement deterministic in-binary classification in `src/commands/workflow.rs` (< 5ms latency, zero token cost, zero network).
   - Classify `fix/*`, `chore/*`, `spike/*`, `test/*` branches as `Organic Mode`.
   - Classify `feat/*`, `spec/*` branches as `Compound Mode`.
   - Enforce the **OpenSpec Precedence Rule (Anti-Dual-Tracking)**: if an unresolved directory exists in `openspec/changes/<feature>/`, `Compound Mode` strictly overrides branch prefixes and `odd/tasks/` presence.
   - Implement non-git workspace fallback evaluating directory existence (`openspec/changes/` vs `odd/tasks/`) or `AdoptionTier`.
   - Support manual overrides via `--mode [odd|organic|compound|ce]` across `workflow resume`, `workflow checkpoint`, and `workflow status`.
3. **Turn-0 Banner Delivery:**
   - Customize `resume_lines()` in `src/commands/workflow.rs`:
     - In `Organic Mode`, output the ODD Fast-Path guidance banner (Problem + Guardrails + DoD) and suppress 7-stage OpenSpec warnings.
     - In `Compound Mode`, emit the full 7-stage FSM progress table and OpenSpec status lines.
4. **Canonical ODD Brief Schema & Helpers:**
   - Define canonical schema for `odd/tasks/<feature>.md` (YAML frontmatter, `# Problem Statement`, `# Guardrails & Invariants`, `# Definition of Done (DoD)` with markdown checkboxes).
   - Implement markdown generator (`generate_odd_task_template`) and structured parser (`parse_odd_task_file`).
5. **Graduation Bridge (`ce-ai workflow graduate`):**
   - Implement `ce-ai workflow graduate [feature]` and register top-level alias `ce-ai graduate [feature]`.
   - Map Problem Statement ➔ `proposal.md`, Guardrails ➔ `spec.md`, and DoD checkboxes ➔ `tasks.md` (preserving checked state `[x]`).
   - Remove `odd/tasks/<feature>.md` upon successful migration to eliminate dual-ledger desynchronization.
   - Advance workflow state directly to Stage 4 (`WorkTdd`) in `state.json`.
6. **Dual-Track Write Gate Policy:**
   - Update `evaluate_gate_policy` in `src/commands/gate.rs`: permit tool writes without OpenSpec when `ExecutionMode::Organic` is active.
   - Implement observe-only tactical ceiling check: if uncommitted git diff exceeds 200 LOC, emit an advisory notice recommending graduation without blocking tool writes (exit code 0).
7. **Persistence & Verification:**
   - Align Engram persistent memory topic keys under `tasks/<feature>` for both modes.
   - Comprehensive unit tests and end-to-end CLI integration tests.

## Out-of-Scope

1. **Runtime LLM-Based Routing Agent:** Evaluated and explicitly rejected due to 1.5–3.5s latency and recurring token costs. Classification is strictly deterministic and in-binary.
2. **Deprecation or Weakening of OpenSpec:** `openspec/specs/` remains the mandatory system of record for living domain specifications, and `openspec/changes/` remains mandatory for architectural features (> 200 LOC).
3. **Automatic Git Branching:** The router classifies and adapts to the active branch; it does not invoke `git checkout -b` or force branch switching.
4. **Modifying External Companion Internals:** No modifications to upstream companion binaries (`codegraph`, `engram`, `context7`, `rtk`).

## Risk Evaluation & Mitigation

- **Risk 1: False Positive Organic Mode on Architectural Refactors.**
  - *Risk:* A developer starts a large architectural overhaul on a `fix/` branch, bypassing OpenSpec governance.
  - *Mitigation:*
    1. CLI flag `--mode compound` provides instant manual override.
    2. The OpenSpec Precedence Rule ensures any active directory in `openspec/changes/` forces Compound Mode.
    3. The gate policy inspects git diff: if changes exceed 200 LOC, it emits an advisory notice recommending `ce-ai workflow graduate`.
- **Risk 2: Data Loss or Desynchronization during Graduation.**
  - *Risk:* Partial failure during `workflow graduate` could delete the ODD brief without creating the OpenSpec change package.
  - *Mitigation:* Strict atomic sequencing: validate source existence, assert destination does not exist, write all three OpenSpec files and flush to disk, update `state.json` via `write_atomic`, and only then remove `odd/tasks/<feature>.md`.
- **Risk 3: Latency Regression in Turn-0 Hook.**
  - *Risk:* Calling `probe_execution_mode` on every harness invocation slows down agent interactions.
  - *Mitigation:* The probe performs in-memory string prefix matching on `repo_state.branch` (already resolved) and cached filesystem directory checks (`Path::is_dir()`), completing in < 5ms.

## Success Criteria

1. **Sub-5ms Turn-0 Evaluation:** `probe_execution_mode` resolves and returns in under 5ms.
2. **Zero-Ceremony Tactical Work:** Bugfixes and chores under 200 LOC can be implemented, verified, and shipped without creating an `openspec/changes/` directory.
3. **Lossless Graduation:** Running `ce-ai workflow graduate` on an ODD file produces a valid OpenSpec change package with 100% of Problem, Guardrails, and checked DoD checkboxes preserved, transitioning directly to Stage 4 (`WorkTdd`).
4. **100% Quality Gates:** Zero Clippy warnings (`-D warnings`), strict formatting (`cargo fmt --check`), and 100% pass rate across unit and CLI integration tests.
