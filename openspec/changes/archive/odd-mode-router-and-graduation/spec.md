# Specification: Adaptive Turn-0 Mode Router & Graduation Bridge

## Requirements

### Requirement 1: ExecutionMode Domain Model
- **WHEN** `ExecutionMode` is serialized or deserialized,
- **THEN** it MUST support variants `Auto`, `Organic`, and `Compound`.
- **WHEN** parsed from user CLI strings via `ExecutionMode::parse`,
- **THEN** it MUST parse `"odd"` and `"organic"` as `ExecutionMode::Organic`, `"compound"`, `"ce"`, and `"openspec"` as `ExecutionMode::Compound`, and `"auto"` as `ExecutionMode::Auto`.

### Requirement 2: Sub-5ms Deterministic Turn-0 Mode Evaluation
- **WHEN** `ce-ai workflow resume` is executed on Turn-0,
- **THEN** `probe_execution_mode()` MUST complete evaluation in under 5ms without issuing HTTP requests or calling external LLM APIs.

### Requirement 3: Branch-Based Organic Heuristics
- **WHEN** the active git branch prefix is `fix/*`, `chore/*`, `spike/*`, or `test/*`,
- **AND** no unresolved directory exists in `openspec/changes/`,
- **THEN** `probe_execution_mode()` MUST resolve to `ExecutionMode::Organic`.

### Requirement 3b: Non-Git Workspace Heuristics
- **WHEN** running in a non-git directory,
- **THEN** `probe_execution_mode()` MUST evaluate directory presence (`openspec/changes/` ➔ `Compound`, `odd/tasks/` ➔ `Organic`), falling back to `ExecutionMode::Organic` if `AdoptionTier::Minimal` is set, or `ExecutionMode::Compound` if `AdoptionTier::Full` is set.

### Requirement 4: Branch-Based Compound Heuristics
- **WHEN** the active git branch prefix is `feat/*` or `spec/*`,
- **THEN** `probe_execution_mode()` MUST resolve to `ExecutionMode::Compound`.

### Requirement 4b: OpenSpec Precedence Rule (Anti-Dual-Tracking)
- **WHEN** an unresolved change directory exists in `openspec/changes/<feature>/`,
- **THEN** `probe_execution_mode()` MUST resolve to `ExecutionMode::Compound`, strictly taking precedence over branch naming prefixes and `odd/tasks/` presence.

### Requirement 5: Manual CLI Mode Overrides
- **WHEN** the `--mode` flag is passed to `workflow resume`, `workflow checkpoint`, or `workflow status`,
- **THEN** the specified mode MUST take precedence over heuristic branch and directory inspection.
- **WHEN** an invalid mode value is supplied,
- **THEN** the CLI MUST exit with code 2 (`CeError::Usage`).

### Requirement 6: Tailored Turn-0 Guidance Banners
- **WHEN** `workflow resume` executes in `Organic Mode`,
- **THEN** `resume_lines()` MUST render the ODD Fast-Path banner displaying the Problem + Guardrails + DoD guidance, and MUST suppress the 7-stage OpenSpec warning table.
- **WHEN** `workflow resume` executes in `Compound Mode`,
- **THEN** `resume_lines()` MUST render the formal 7-stage Compound Engineering progress table and OpenSpec status.

### Requirement 8: Standard Path for Organic Tasks
- **WHEN** an organic task is initialized,
- **THEN** its specification brief MUST be located at `odd/tasks/<feature>.md`.

### Requirement 9: Canonical ODD Brief Schema
- **WHEN** an ODD task brief is created or inspected,
- **THEN** it MUST contain YAML frontmatter (`feature`, `mode: organic`, `created`, `status: active`), followed by `# Problem Statement`, `# Guardrails & Invariants`, and `# Definition of Done (DoD)`.

### Requirement 10: Definition of Done Checkbox Format
- **WHEN** tasks are enumerated in the `# Definition of Done (DoD)` section of `odd/tasks/<feature>.md`,
- **THEN** they MUST be formatted as GitHub-compatible markdown checkboxes (`- [ ]` or `- [x]`).

### Requirement 11: Graduation CLI Command & Alias
- **WHEN** the developer or agent invokes graduation,
- **THEN** the command MUST be accessible via both `ce-ai workflow graduate [feature]` and the top-level alias `ce-ai graduate [feature]`.

### Requirement 12: Source Validation & Destination Collision Guard
- **WHEN** `workflow graduate [feature]` is invoked,
- **THEN** if `odd/tasks/<feature>.md` does not exist, it MUST fail with `CeError::Usage` (exit code 2).
- **AND** if `openspec/changes/<feature>` already exists, it MUST fail with `CeError::State` (exit code 3) without overwriting existing files.

### Requirement 13: Problem Statement Mapping
- **WHEN** graduating an ODD task,
- **THEN** the content of `# Problem Statement` MUST be written into `openspec/changes/<feature>/proposal.md`.

### Requirement 14: Guardrails Mapping
- **WHEN** graduating an ODD task,
- **THEN** the content of `# Guardrails & Invariants` MUST be written into `openspec/changes/<feature>/spec.md` as formal requirements.

### Requirement 15: Definition of Done Mapping & Checkbox State Preservation
- **WHEN** graduating an ODD task,
- **THEN** the checklist items in `# Definition of Done (DoD)` MUST be written into `openspec/changes/<feature>/tasks.md`, preserving the exact completion status of all checked (`- [x]`) and unchecked (`- [ ]`) items.

### Requirement 16: Post-Graduation Source Cleanup
- **WHEN** all OpenSpec files are successfully created on disk,
- **THEN** `odd/tasks/<feature>.md` MUST be deleted to eliminate dual-ledger drift.

### Requirement 17: Direct Workflow Stage 4 (WorkTdd) Transition
- **WHEN** graduation completes successfully,
- **THEN** `state.json` MUST be updated via `write_atomic`:
  - `task` set to `<feature>`
  - `stage` set to `WorkflowStage::WorkTdd` (Stage 4)
  - `execution_mode` set to `Some(ExecutionMode::Compound)`

### Requirement 18: Organic Mode Gate Exemption
- **WHEN** `ce-ai gate check` executes,
- **AND** `ExecutionMode::Organic` is active,
- **THEN** the gate MUST permit file writes without requiring an existing `openspec/changes/` directory.

### Requirement 19: Observe-Only 200 LOC Ceiling Advisory
- **WHEN** `ExecutionMode::Organic` is active,
- **AND** uncommitted git diff exceeds 200 lines of code,
- **THEN** `ce-ai gate check` MUST emit an advisory notice recommending `ce-ai workflow graduate`.

### Requirement 20: Non-Blocking Gate Execution
- **WHEN** the 200 LOC tactical ceiling advisory is emitted,
- **THEN** `ce-ai gate check` MUST exit with code 0 (`Pass`) and MUST NOT block tool writes.

### Requirement 21: Unified Engram Topic Key Mirroring
- **WHEN** mirroring task progress into Engram persistent memory,
- **THEN** both ODD tasks and OpenSpec tasks MUST use the stable topic key `tasks/<feature>`.

### Requirement 22: Context Re-hydration Parity
- **WHEN** an agent performs context recovery via `mem_context`,
- **THEN** the active task state MUST be accurately recovered regardless of whether it originated as an ODD brief or an OpenSpec package.
