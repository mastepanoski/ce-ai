# Specification: CE-AI v2 Architectural Pivot & PRD

## Requirements & Acceptance Criteria

### REQ-1: Workflow State Observability without Authoritative Storage
- **WHEN** a user or agent invokes `ce-ai workflow status` or `ce-ai workflow resume`,
- **THEN** the system MUST observe workflow progress directly from repository artifacts (`git branch`, plan files, commits, PR status, run receipts) without relying on a stored authoritative stage cursor in `state.json`.
- **WHEN** repository state is evaluated,
- **THEN** non-linear flows (including `ce-debug` bugfix loops and direct plan-less execution) MUST be recognized as valid workflow states rather than emitting gating errors.

### REQ-2: Fleet Version Governance via Native Installers
- **WHEN** a user specifies a target Compound Engineering version (e.g. `ce-ai fleet pin v1.32.0`),
- **THEN** `ce-ai` MUST record the target release version in configuration without downloading or unpacking raw `.tar.gz` files into harness directories.
- **WHEN** `ce-ai fleet sync` or `ce-ai fleet status` is executed,
- **THEN** `ce-ai` MUST invoke or inspect each host's native plugin manager to report or update installed versions, flagging version divergence across harnesses without creating duplicate asset directories.

### REQ-3: Upstream Document Schema & Directory Configuration
- **WHEN** inspecting documentation artifacts,
- **THEN** `ce-ai` MUST resolve `<docs_root>` dynamically by reading `.compound-engineering/config.yaml` (defaulting to `docs/` if absent).
- **WHEN** validating solution frontmatter in `<docs_root>/solutions/`,
- **THEN** the validator MUST enforce upstream `schema.yaml` fields: mandatory `module`, `date`, `problem_type`, `component`, `severity`, and optional `schema_version`, `related_components`, `tags`, and `applies_when` (knowledge-track only).
- **WHEN** identifying ideation/brainstorm documents,
- **THEN** `ce-ai` MUST treat requirements-only plan files under `<docs_root>/plans/` as the primary modern format, while treating `<docs_root>/brainstorms/` as a backward-compatible legacy location.

### REQ-4: Conflict Prevention & Surface Cleanliness
- **WHEN** configuring Pi extensions,
- **THEN** `ce-ai` MUST NEVER overwrite `.pi/extensions/compound-engineering.ts` (reserved for upstream CE), writing any companion extension to a distinct namespaced file (`.pi/extensions/ce-ai-companion.ts`).
- **WHEN** evaluating dead paths in documentation,
- **THEN** the dead-path checker MUST inspect all relevant source code extensions (`.rs`, `.ts`, `.js`, `.py`, `.go`, `.rb`) rather than restricting checks solely to `.rs`.
