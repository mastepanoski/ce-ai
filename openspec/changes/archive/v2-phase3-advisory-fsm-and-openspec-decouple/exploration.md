# Exploration: Phase 3 — Advisory Workflow FSM & OpenSpec Decoupling (v1.77.0)

## Technical Investigation & Options Evaluated

### Option A: Retain Authoritative Stage Cursor with Dynamic Exemptions
- **Description:** Keep `current_stage` in `state.json` as the source of truth, adding heuristics and bypass flags (`--skip-openspec`, `--no-gate`) to allow non-linear execution.
- **Evaluation:**
  - *Pros:* Minimal changes to existing FSM code.
  - *Cons:* Fails to address the core problem diagnosed by the Compound Engineering maintainer. A stored cursor in `state.json` is fundamentally decoupled from git branches, commits, plans, and PRs. It continuously drifts and requires complex reconciliation. Moreover, keeping the gate authoritative still burdens workflows with semantic overreach.
  - *Verdict:* **Ruled out.**

### Option B: Complete Removal of Workflow FSM and Status Commands
- **Description:** Delete `WorkflowState`, `ce-ai workflow`, and `ce-ai gate` entirely, leaving all coordination to bash scripts or manual user steps.
- **Evaluation:**
  - *Pros:* Simplifies the binary codebase.
  - *Cons:* Completely removes environment awareness, handoff tracking, test verification receipts, and telemetry. Agents and developers lose a valuable diagnostic lens that quickly summarizes branch state, plan progress, and knowledge capture needs.
  - *Verdict:* **Ruled out.**

### Option C: Decoupled Observable Capabilities Matrix with Advisory Observation (Selected)
- **Description:** Shift `ce-ai` from an authoritative state gatekeeper to a read-only **Advisory Workflow Observation Engine** (`ObservableWorkflowState`). Repository artifacts (git branch, working tree changes, markdown plans, review receipts, solution docs) are the single source of truth. OpenSpec is decoupled from the core domain as an optional integration.
- **Evaluation:**
  - *Pros:*
    - Zero state drift: state is computed on-demand from real filesystem and git artifacts.
    - Zero artificial documentation debt: small fixes, refactors, and debug sessions do not require dummy or heavy OpenSpec packages.
    - Full alignment with upstream Compound Engineering: respects CE's non-linear execution model and native contracts (`ce-handoff`, `ce-work`, `ce-compound`).
    - Non-disruptive gate check: Claude Code and other agent hooks continue to log rich telemetry to `gate-events.jsonl` without blocking code writes.
    - Backwards-compatible: existing JSON consumers receive both the new observation matrix and advisory stage mappings.
  - *Verdict:* **Selected.**

## Architectural Trade-offs & Decisions

1. **Inspection Latency:**
   - Computing `ObservableWorkflowState` involves querying git branch, git status porcelain (if available), checking directory presence in `docs/plans/` and `docs/solutions/`, and parsing markdown task checkboxes. In Rust, these disk operations take < 12ms, well within interactive CLI thresholds.
2. **Backwards Compatibility in JSON Output:**
   - `ce-ai workflow status --json` will include the full `observable_state` object while maintaining existing top-level fields (`current_phase`, `active_subtask`, `feature_name`) populated from observable context or advisory checkpoints.
3. **Gate Check Default Mode:**
   - Shifting default `GateMode` from `Enforce` to `Observe` ensures that agent pre-tool write hooks (such as Claude Code PreToolUse) never return a blocking exit code 1 due to missing OpenSpec files. Users who specifically desire strict blocking can still opt-in via `--mode enforce` or `CE_AI_GATE_MODE=enforce`.
