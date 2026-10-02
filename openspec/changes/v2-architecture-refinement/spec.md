# Specification: Decoupling Semantic Authority & OpenSpec in CE-AI v2

## Requirements & Acceptance Criteria

### REQ-1: Principle of No Semantic Authority
- **WHEN** CE-AI v2 evaluates repository state or coordinates tasks,
- **THEN** it MUST NOT introduce mandatory workflow stages, artifacts, or transition requirements beyond those defined by the Compound Engineering contracts it integrates with.
- **WHEN** Compound Engineering does not require an artifact for a given operation (e.g. trivial work or direct bugfix),
- **THEN** CE-AI MUST NOT block writes or report blocking errors for that missing artifact.

### REQ-2: Principle of Repository Reality Over Mirrored State
- **WHEN** CE-AI v2 reconstructs workflow context (e.g. in `ce-ai workflow resume`),
- **THEN** it MUST derive its observation directly from authoritative repository artifacts (git branches, commits, PR status, `docs_root/plans/`, run receipts).
- **WHEN** stored workflow state in `state.json` differs from or is missing relative to repository artifacts,
- **THEN** repository artifacts MUST take absolute precedence, and CE-AI state MUST be disposable and reconstructable.

### REQ-3: OpenSpec Decoupling from Core Domain
- **WHEN** CE-AI operates under native Compound Engineering workflows,
- **THEN** OpenSpec MUST be treated as an optional, opt-in integration outside the core domain model.
- **WHEN** a user or agent has not explicitly activated OpenSpec for a task,
- **THEN** CE-AI MUST NOT require `openspec/changes/<feature>/` directories or files (`proposal.md`, `spec.md`, `tasks.md`).

### REQ-4: Observable Capabilities State Model
- **WHEN** CE-AI inspects the repository,
- **THEN** it MUST construct an `ObservableWorkflowState` matrix representing objective capabilities (`active_work`, `plan`, `verification`, `handoff`, `knowledge_capture`, `openspec`), rather than coercing the repository into a scalar linear stage number.

### REQ-5: PRD & Migration Plan Alignment
- **WHEN** the CE-AI v2 PRD (`docs/architecture/ce-ai-v2-architecture-prd.md`) and Migration Plan (`docs/plans/2026-10-01-ce-ai-v2-architectural-migration-plan.md`) are published,
- **THEN** they MUST explicitly document the "No Semantic Authority" and "Repository Reality Over Mirrored State" principles, the decoupled core domain architecture, and the capabilities-based observation engine.
