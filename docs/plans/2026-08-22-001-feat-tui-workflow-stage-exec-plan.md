# Implementation Plan: TUI Workflow Panel — Native Action Execution

**Origin:** docs/brainstorms/tui-workflow-stage-exec-requirements.md
**OpenSpec:** openspec/changes/tui_workflow_stage_exec/
**Date:** 2026-08-22

## Overview

Make the Workflow (FSM) panel in the Ratatui dashboard actionable for ce-ai's native subcommands: `[Enter]` renders the real `workflow status` output (today it shows a canned message), `[1-7]` checkpoints stay as-is, stage rows distinguish native actions from agent-harness skills, and a teacher-style docs section explains the native-vs-skill boundary. The `workflow resume` keybinding is deliberately excluded this iteration.

---

## Requirements Traceability

| Req | Summary | Unit |
|---|---|---|
| R1 | `[Enter]` renders actual `workflow status` output | U1, U2 |
| R2 | `[1-7]` checkpoint behavior preserved | U2 |
| R3 | Resume keybinding excluded; recorded as follow-up | U3 |
| R4 | Command failures render actionable modal copy | U1, U2 |
| R5 | Modal success/failure classes; close-precedence | U2 |
| R6 | Native-vs-skill markers, not color-only | U3 |
| R7 | Tech-neutral guide copy | U3 |
| R8 | Hints list every action | U3 |
| R9 | Teacher-style why-docs | U4 |
| R10 | Repo style-guide compliance | U4 |

---

## Implementation Units

### U1. Workflow commands return output lines

- **Goal:** `workflow status/checkpoint/resume` produce renderable lines instead of printing to stdout.
- **Requirements:** R1, R4.
- **Dependencies:** none.
- **Files:** src/commands/workflow.rs
- **Approach:** Internal per-action functions return `Result<Vec<String>, CeError>`; the public `run(ctx, args)` keeps its signature and prints returned lines via `println!`, preserving CLI behavior and exit-code mapping. No stdout capture — the TUI owns stdout via crossterm's alternate screen. Status lines derive from real state reads (stage, task, checkpoint from `State.last_update_check`), replacing today's canned strings.
- **Patterns to follow:** `run_*_cmd -> Vec<String>` shape in src/tui.rs:828-1052; error propagation conventions from docs/solutions/backup-restore-management-and-point-in-time-recovery.md.
- **Test scenarios:**
  - Happy path: `status_lines` on a temp state file returns lines containing current phase/task and no canned "Use 'ce-ai workflow checkpoint'" filler unless truly absent.
  - Checkpoint: saving a transition returns confirmation lines including phase and task; state.json gains the formatted `last_update_check` entry (`{phase} | {task} | {timestamp}`).
  - Error path: unreadable/corrupt state.json yields `CeError::Io`/`CeError::State` (not a panic); CLI run maps exit codes unchanged.
  - Integration: `ce-ai workflow status` CLI output equals the lines the TUI would render (single source of truth).
- **Verification:** unit tests pass; CLI behavior byte-compatible with pre-refactor output.

### U2. TUI renders real output with failure class

- **Goal:** Workflow tab `[Enter]` shows actual status output; failures render as a distinct modal class.
- **Requirements:** R1, R2, R4, R5.
- **Dependencies:** U1.
- **Files:** src/tui.rs
- **Approach:** Replace `run_workflow_cmd`'s canned success path with a call into U1's line-returning functions via `execute_action`. Introduce a small pure helper that maps `Ok(lines)` → success block and `Err(CeError)` → failure block (❌ prefix + actionable copy). Modal-close precedence and reload-on-close already exist at src/tui.rs:257-262 — no change needed there.
- **Execution note:** test-first on the pure helper before wiring the event loop.
- **Test scenarios:**
  - Covers AE1. Given readable state, status action produces lines matching U1 output.
  - Covers AE2. Given corrupt state file, helper returns failure-class lines containing an actionable message and ❌ marker.
  - `[1-7]` checkpoint flow still produces "Workflow Stage Transition" modal content (regression guard).
- **Verification:** cargo test green; manual TUI smoke check optional.

### U3. Panel guide content rework

- **Goals:** native-vs-skill visual distinction without color-only cues; tech-neutral Verify copy; hints list every action; resume keybinding excluded.
- **Requirements:** R3, R6, R7, R8.
- **Dependencies:** U2.
- **Files:** src/tui.rs
- **Approach:** Rework the `MenuTab::Workflow` render block (src/tui.rs:552-582): each stage row gets a `[run]` / `skill:` text-prefix marker (perceivable without color); Verify row says "project test/e2e commands"; footer hint enumerates `[Enter]` status, `[1-7]` checkpoints — no resume hint. Remove any suggestion of unbound keys.
- **Test scenarios:**
  - Rendered workflow panel lines contain exactly one marker per stage row (`[run]` or `skill:`).
  - Verify-stage line contains no toolchain names (`cargo`, `make`, `npm`).
  - Footer hint lists `[Enter]` and `[1-7]`; grep of tui.rs confirms no resume-key binding added.
- **Verification:** unit tests on the lines-building function.

### U4. Teacher-style documentation section

- **Goals:** newbie-friendly explanation of the native-vs-skill boundary and the chosen-not-capable rationale; style-guide compliance.
- **Requirements:** R9, R10.
- **Dependencies:** none (can parallel U1-U3).
- **Files:** docs/user-guide/workflow-panel.md (new), README.md (one-line map entry only if ≤100 lines maintained)
- **Approach:** Single Diátaxis intent (explanation) covering: what each Workflow-panel action does natively, why agent stages are guide-only (harness-specific delegation paths across 12 harnesses; single-dashboard experience), how each stage maps to its opencode skill, and that the resume keybinding was deliberately excluded. Must satisfy the AE5 checklist verbatim.
- **Test scenarios:**
  - Checklist test (AE5): passage classifies every action native-vs-guide; states rationale; lists skill per stage scoped to opencode; notes other-harness mappings out of scope; records resume exclusion.
  - README line-count stays within budget after edit (or README untouched).
- **Verification:** checklist review; docs-styling.md conformance.

### U5. Release governance artifacts

- **Goal:** SemVer bump, changelog, OpenSpec sync.
- **Requirements:** repo DoD (mandatory versioning).
- **Dependencies:** U1-U4.
- **Files:** Cargo.toml, Formula/ce-ai.rb, CHANGELOG.md, openspec/changes/tui_workflow_stage_exec/tasks.md (checkbox completion)
- **Approach:** MINOR bump (new user-facing capability). Conventional-commit friendly.
- **Test expectation:** none — version/changelog metadata only.

---

## Key Technical Decisions

- **Return-lines refactor over stdout capture**: capturing stdout is incompatible with ratatui's alternate-screen ownership; return-values match every other tab's pattern (see origin Dependencies section).
- **No resume keybinding**: origin decision — print-only stub adds nothing over `[1-7]`; honest execution forbids false-success UX.
- **Failure class in modal**: `Err(CeError)` renders as defined failure copy rather than falling back to canned success text.

## Risks & Mitigations

- Refactor changes CLI-visible formatting → mitigate by keeping `println!` at the `run()` boundary and diffing CLI output pre/post refactor (U1 integration scenario).
- Marker prefixes could crowd narrow terminals → keep markers short (`[run]`, `skill:`).

## Scope Boundaries

- No execution of project-specific commands from the TUI; no harness delegation or external terminals; no clipboard features; no Dry-Run in this panel; no async TUI rework; no resume keybinding.

### Deferred to Follow-Up Work

- Real checkpoint-based recovery (requires defined user delta beyond `[1-7]`) — candidate follow-up OpenSpec change.
- Other-harness skill-name mappings in docs.
