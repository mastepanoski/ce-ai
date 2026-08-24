# Spec: TUI Workflow Panel — Native Action Execution

## ADDED Requirements

### Requirement: Real status output in the result modal
The Workflow tab `[Enter]` action SHALL execute `ce-ai workflow status` and render its actual output lines in the result modal, replacing the current canned success message.

#### Scenario: Status renders real output
- **WHEN** the user presses `[Enter]` on the Workflow tab with a readable `state.json`
- **THEN** the modal shows the actual status content derived from state (phase/task/checkpoint), not a canned success message

### Requirement: Checkpoint keys unchanged
Keys `[1-7]` SHALL continue saving stage-transition checkpoints exactly as today.

#### Scenario: Stage transition preserved
- **WHEN** the user presses any key in `[1-7]`
- **THEN** a checkpoint transition is recorded and the confirmation modal appears as before

### Requirement: No resume keybinding this iteration
The panel SHALL NOT bind any key to `workflow resume`; real checkpoint-based recovery SHALL be recorded as a candidate follow-up.

#### Scenario: No unbound-key suggestions
- **WHEN** the Workflow panel renders its hints
- **THEN** no hint advertises a resume action or key

### Requirement: Command failure modal class
Native command failures (`CeError`) SHALL render in the result modal as a distinct failure class with actionable copy; dashboard state remains unchanged.

#### Scenario: Corrupt state renders failure copy
- **WHEN** `state.json` is corrupted or unreadable and a native action runs
- **THEN** the modal shows ❌ failure copy naming the cause and remedy, and no silent/canned success text appears

### Requirement: Modal interaction contract
Any key SHALL close an open modal and reload dashboard state; modal-close SHALL take precedence over action keybindings.

#### Scenario: Modal close precedence
- **WHEN** a modal is open and any key is pressed
- **THEN** the modal closes and the panel reflects freshly loaded state

### Requirement: Native-vs-skill guide distinction
Stage rows SHALL distinguish executable native actions from agent-session skills using markers perceivable without color alone, naming each stage's agent-harness skill scoped to opencode.

#### Scenario: Markers present and tech-neutral
- **WHEN** the Workflow panel renders
- **THEN** each stage row carries exactly one `[run]` or `skill:` marker, and the Verify row names no project-specific toolchain

### Requirement: Complete hints
Panel hints/footer SHALL enumerate every available action so nothing is undiscoverable.

#### Scenario: Hints list all actions
- **WHEN** the Workflow panel renders its footer hint
- **THEN** it lists `[Enter]` status and `[1-7]` checkpoints

### Requirement: Teacher-style documentation
A docs section SHALL explain why the panel only executes native subcommands and how agent stages connect to harness skills, satisfying the AE5 classification checklist.

#### Scenario: Docs checklist passes
- **WHEN** the docs section is reviewed against the checklist
- **THEN** it contains: native-vs-guide classification for every action, chosen-not-capable rationale, per-stage skill mapping (opencode), other-harness mappings declared out of scope, and the resume-exclusion note
