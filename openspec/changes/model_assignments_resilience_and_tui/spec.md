# Specification: Model Assignments Resilience and TUI Editor

## Formal Requirements

### Requirement 1: Doctor Model Assignment Drift Probe
WHEN `ce-ai doctor` is executed AND an `agent.<slot>` assignment exists in `opencode.json` that is missing from `state.json` (or vice-versa), THEN `ce-ai doctor` SHALL report a diagnostic finding warning of model assignment drift and recommending `ce-ai sync`.

### Requirement 2: Sync Model Assignment Reconciliation
WHEN `ce-ai sync` is executed, THEN it SHALL reconcile model assignments bidirectionally between `state.json` and active harness configurations (`opencode.json`), persisting missing entries into `state.json` and writing missing harness entries into `opencode.json`.

### Requirement 3: Default Model Assignments on Fresh Install
WHEN `ce-ai install` is executed on a fresh environment, THEN it SHALL populate standard default model assignments for `ce-ai`, `ce-brainstorm`, `ce-plan`, `ce-work`, `ce-code-review`, and `ce-doc-review` into `state.json` and `opencode.json` without overwriting any pre-existing user configurations.

### Requirement 4: Interactive TUI Model Assignment Editor
WHEN a user navigates to the Models & Profiles tab in the TUI dashboard, THEN they SHALL be able to select any model slot using keyboard navigation (`Up`/`Down`, `j`/`k`), edit the assigned model string, and commit the update atomically using `Enter`.
