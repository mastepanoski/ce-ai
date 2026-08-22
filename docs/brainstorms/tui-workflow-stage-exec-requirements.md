---
date: 2026-08-22
topic: tui-workflow-stage-exec
---

# TUI Workflow Panel — Native Action Execution

## Summary

The Workflow (FSM) panel becomes an actionable surface for ce-ai's native workflow subcommands: `[Enter]` runs status and renders its real output, `[1-7]` save stage checkpoints (existing behavior), and stage rows become guide text that distinguishes executable native actions from agent-harness skills. The `workflow resume` keybinding is deliberately excluded from this iteration (see Key Decisions). A newbie-friendly documentation section explains the reasoning behind this split.

---

## Problem Frame

A developer managing Compound Engineering across projects opens the `ce-ai` dashboard to check workflow state. The Workflow panel advertises a 7-stage cycle with mapped commands (`ce-brainstorm`, `ce-plan`, `ce-work`, ...), yet every action requires quitting the dashboard and typing in a terminal. The panel tells the user *what* each stage needs but offers no way to act on it.

There is a hidden trap behind "just run them from the panel": most of the listed commands are not ce-ai binaries at all — they are agent-harness skills (OpenCode/Claude commands), and each harness would need its own delegation path (`opencode run` vs a different Claude invocation across the twelve supported harnesses). The TUI could technically delegate non-interactively, but that choice was rejected: it breaks the single-dashboard experience and output capture. Meanwhile, some stage suggestions like test runners are project-specific (`cargo test` is meaningless from a Next.js project), so hardcoding them would make the panel wrong for most users.

---

## Requirements

**Panel execution surface**
- R1. `[Enter]` on the Workflow tab executes `ce-ai workflow status` and renders its actual output in the result modal. Today's handler shows only a canned success message; this change must surface real command output.
- R2. Keys `[1-7]` save stage-transition checkpoints exactly as they work today.
- R3. The always-visible `workflow resume` keybinding is excluded from this iteration: `resume` is a print-only stub that adds nothing over `[1-7]`, and binding it would violate honest execution. Real checkpoint-based recovery is recorded as a candidate follow-up change requiring its own defined user delta.
- R4. When a native command fails (e.g., unreadable or corrupted `state.json`), the result modal shows the command-failure copy class from R5 with an actionable message instead of failing silently; dashboard state remains unchanged.
- R5. All command outputs reuse the existing result-modal pattern; any key closes it and dashboard state reloads. Modal-close takes precedence over action keybindings. The modal renders two content classes with distinct copy: successful command output and command failure (`CeError`), so runtime failures surface as a defined modal state instead of falling back to canned text.

**Stage guide content**
- R6. Stage rows visually distinguish what is executable natively (status via `[Enter]`, checkpoints via `[1-7]`) from what belongs to an agent session, naming the corresponding agent-harness skill per stage (e.g., Stage 1 → `/ce-brainstorm`). The distinction must be perceivable without relying on color alone (e.g., a distinct prefix label such as `[run]` vs `skill:`); the exact treatment is deferred to planning subject to that constraint.
- R7. Guide copy is tech-neutral: the Verify stage references "the project's test/e2e commands" rather than hardcoded tool names, so the panel stays correct from any stack.
- R8. Panel hints/footer text list every available action so nothing is undiscoverable.

**Documentation**
- R9. User-facing documentation explains, in teacher-style prose aimed at newcomers, *why* the panel only executes native subcommands and how agent stages connect to harness skills — the reasoning, not just the keybindings. Skill naming follows the primary documented harness (opencode); other-harness equivalents are out of scope for this section.
- R10. Documentation follows the repo style guide: single Diátaxis intent per document, README stays within its line budget with deep content routed to `docs/`.

---

## Acceptance Examples

- AE1. **Covers R1, R5.** Given a readable `state.json`, when the user presses `[Enter]` on the Workflow tab, the modal shows the actual status output and closing it returns the dashboard to interactive state.
- AE2. **Covers R4, R5.** Given a corrupted or unreadable `state.json`, when any native action runs, the modal shows command-failure copy with an actionable message; dashboard state is unchanged.
- AE3. **Covers R5.** Given any modal is open, when any key is pressed, the modal closes and the panel reflects freshly loaded state.
- AE4. **Covers R7.** Given the user runs the TUI from a non-Rust project, when viewing the Workflow tab, the Verify stage text names no project-specific toolchain.
- AE5. **Covers R9.** Review checklist (static, no reader testing required): the docs section contains an explicit passage that classifies every Workflow-panel action as native-vs-guide, states the chosen-not-capable rationale (delegation would be harness-specific and break the single-dashboard experience), lists the corresponding harness skill per stage scoped to the primary documented harness (opencode), states that other-harness mappings are out of scope, and records that the resume keybinding was deliberately excluded this iteration.

---

## Success Criteria

- A user can query workflow status and save checkpoints entirely from the Workflow panel; agent-driven stages clearly point to the harness session where they run.
- The panel never suggests running something it cannot run; every advertised action works as described.
- The docs section contains a complete native-vs-guide classification with per-stage skill mapping and the chosen-not-capable rationale (verified via the AE5 checklist).
- Planning receives resolved product scope: every behavior is decided except the mechanical details explicitly listed under Deferred to Planning.

---

## Scope Boundaries

- No execution of project-specific commands (tests, e2e suites) from the TUI — they block the synchronous event loop and are meaningless outside their ecosystem.
- No non-interactive delegation to agent harnesses (e.g., `opencode run`) and no external terminal spawning.
- No clipboard-copy of suggested agent prompts.
- No Dry-Run integration in this panel — verified against the codebase: status never mutates `state.json`, resume is not bound to any key this iteration, and checkpoint writes stage transitions that are trivially reversible by re-pressing the prior stage key.
- No async/background execution rework of the TUI loop.

---

## Key Decisions

- **Honest execution only**: run exclusively native ce-ai subcommands; agent stages stay as guide text. `ce-*` skills are harness-level, not binaries — the TUI chooses not to launch them because delegation would require a harness-specific invocation path per supported harness and would break the single-dashboard experience and output capture.
- **Resume keybinding deferred**: `workflow resume` is a print-only stub adding nothing over `[1-7]` today; binding it would show false success and violate honest execution. Real checkpoint-based recovery is a candidate follow-up requiring its own defined user delta (e.g., restoring non-stage state).
- **Dry-Run excluded** despite issue #76's literal text: codebase verification shows status never writes `state.json`, resume ships unbound this iteration, and checkpoint transitions are trivially reversible via `[1-7]`; preview adds carrying cost with no safety value here.
- **Reuse existing interaction patterns** (result modal, existing key handlers): consistency with Install/Sync keeps learning cost near zero.

---

## Dependencies / Assumptions

- The `workflow` subcommands exist but print to stdout via `println!` and return only `Result<(), CeError>` — there is no return value a modal can consume today. Rendering real output in the modal requires refactoring them to return output lines (or capturing stdout).
- The existing action-execution and output-modal infrastructure is reusable for all three actions.
- Verified: `resume` never inspects checkpoint state and always succeeds (`State::load` returns defaults when `state.json` is missing entirely — no panic path exists). Checkpoints are stored in `State.last_update_check` (no dedicated checkpoint field exists; entries have the format `{phase} | {task} | {timestamp}`). With the resume keybinding dropped, no TUI-side pre-check is needed this iteration.

---

## Outstanding Questions

### Resolve Before Planning

- None.

### Deferred to Planning

- [Affects R6][Technical] Exact marker treatment for the executable-vs-skill distinction (e.g., `[run]` vs `skill:` prefixes), subject to the not-color-only constraint.
- [Affects R5][Technical] Whether missing-state and command-failure cases need distinct failure copy or one shared message class.

---

## Deferred / Open Questions

### From 2026-08-22 review

- **Clipboard-copy exclusion unjustified despite agent-stage friction** — Scope Boundaries (P2, product-lens, confidence 75)

  The problem frame's core pain is that 'every action requires quitting the dashboard and typing in a terminal', yet by the doc's own analysis five-plus of seven stages are agent skills that remain un-actionable after this ships. Copying the suggested skill prompt (e.g. `/ce-brainstorm`) to the clipboard is a small, synchronous, event-loop-safe affordance that directly reduces that residual friction, but it is excluded in Scope Boundaries with no rationale anywhere — unlike every other exclusion, which gets a justification in Key Decisions.

  <!-- dedup-key: section="scope boundaries" title="clipboardcopy exclusion unjustified despite agentstage friction" evidence="- No clipboard-copy of suggested agent prompts." -->
