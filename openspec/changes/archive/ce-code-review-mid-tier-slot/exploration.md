# Exploration: Mid-Tier Model Slot Architecture and Tradeoffs

## Technical Context
The `ce-ai` codebase manages model configurations across harnesses. For OpenCode, configurations are written to `opencode.json` under `agent.<slot>.model`.

Currently:
- `src/harness/agents.rs` defines `CE_AGENT_SLOTS: [&str; 6]` containing `["ce-ai", "ce-brainstorm", "ce-plan", "ce-work", "ce-code-review", "ce-doc-review"]`.
- `apply_agent_model` modifies `agent.<slot>.model` atomically, preserving custom keys and avoiding writing `variant`.
- `config_assignments` in `src/commands/models.rs` iterates over `CE_AGENT_SLOTS` to read live assignments.
- `model_drift_findings` in `src/commands/models.rs` checks drift for all slots in `CE_AGENT_SLOTS`.
- `clean_managed_config` in `src/opencode/plugins.rs` cleans up all `CE_AGENT_SLOTS` on uninstall.
- `tui/app.rs` populates available model slots from `CE_AGENT_SLOTS`.

## Evaluated Options

### Option 1: Extend `CE_AGENT_SLOTS` to 7 Items & Introduce Stage/Tier Predicates (Recommended)
- Define `CODE_REVIEW_MID_TIER_SLOT: &str = "ce-code-review-mid-tier"`.
- Extend `CE_AGENT_SLOTS: [&str; 7]` to include `CODE_REVIEW_MID_TIER_SLOT`.
- Define `CE_AGENT_STAGE_SLOTS: [&str; 6]` for the 6 primary workflow stages.
- Provide helper predicates `is_tier_slot(slot: &str) -> bool` and `is_stage_slot(slot: &str) -> bool`.

**Impact on existing callers:**
1. `config_assignments`: Reads live assignments for both stage and tier slots. OpenCode users can view their active tier assignment.
2. `model_drift_findings`: Recognizes `ce-code-review-mid-tier` as a known CE slot. Drift between `state.json` and `opencode.json` for this slot will be detected and reconcilable via `ce-ai sync`.
3. `tui/app.rs`: The TUI model picker automatically lists `ce-code-review-mid-tier`, allowing visual configuration.
4. `clean_managed_config`: The mid-tier slot is cleanly removed when uninstalling ce-ai.
5. `models list`: Can format stage slots and tier slots with visual hierarchy instead of treating mid-tier as a top-level stage.

### Option 2: Keep `CE_AGENT_SLOTS` at 6 Items and Create a Separate `CE_TIER_SLOTS` Array
- Maintain `CE_AGENT_SLOTS` as only the 6 stage slots.
- Introduce `pub const CE_TIER_SLOTS: [&str; 1] = [CODE_REVIEW_MID_TIER_SLOT]`.

**Trade-offs:**
- Requires auditing and updating every subsystem (TUI, doctor, drift calculation, sync, uninstall cleanup) to explicitly merge both arrays.
- High likelihood of future bugs where a caller checks `CE_AGENT_SLOTS` and neglects `CE_TIER_SLOTS`.
- Rejected in favor of Option 1.

## UX Design for `ce-ai models list`
`models list` should clearly show the mid-tier slot in association with `ce-code-review`.

1. If `ce-code-review` is configured, render the mid-tier slot nested directly beneath it:
   ```text
   ce-ai: opencode-go/kimi-k2.6
   ce-code-review: anthropic/claude-sonnet-4-5
     └─ mid-tier (ce-code-review-mid-tier): anthropic/claude-haiku-3-5
   ce-work: anthropic/claude-sonnet-4-5
   ```
2. If `ce-code-review-mid-tier` is configured but `ce-code-review` is unset:
   ```text
   ce-code-review-mid-tier (mid-tier sub-slot): anthropic/claude-haiku-3-5
   ```

## Diagnostic Design for `ce-ai doctor`
In `src/commands/doctor.rs`:
- When OpenCode is installed / config can be read:
  - If `ce-code-review` has a model assigned (in `config` or `state`), but `ce-code-review-mid-tier` has no model assigned:
    - Emit an informational diagnostic:
      ```text
      doctor-info: ce-code-review has a model assigned but 'ce-code-review-mid-tier' is not configured; persona-level cost tiering has no explicit target on opencode and may silently fall back to the session model (run 'ce-ai models set --harness opencode ce-code-review-mid-tier <provider/model>' to configure)
      ```
  - If `ce-code-review` does not have a model assigned, or `ce-code-review-mid-tier` is already assigned, emit nothing.
  - Crucially: this is emitted via `println!("doctor-info: ...")` and is NOT pushed to `findings`, so doctor exits 0.
