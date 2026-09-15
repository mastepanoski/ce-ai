---
title: "Workflow Lifecycle, FSM & Turn-0 Delivery Engine"
domain: workflow
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: Workflow Lifecycle, FSM & Turn-0 Delivery Engine

## 1. Overview & Architectural Boundaries

The `workflow` subsystem manages the 7-stage Compound Engineering Flywheel (`Ideation` ➔ `OpenSpec` ➔ `Plan` ➔ `Work/TDD` ➔ `Verify` ➔ `Compound` ➔ `Ship`), enforces state-machine transition invariants, and delivers sub-15ms Turn-0 state awareness to AI coding agents.

## 2. Capabilities & Requirements

### R1. 7-Stage Monotonic Progression & Rewind
WHEN advancing workflow stages via `ce-ai workflow checkpoint`  
THEN the FSM MUST reject non-monotonic jumps (e.g. Stage 1 directly to Stage 3).  
WHEN rewinding stages  
THEN the operator MAY transition backwards to earlier stages or reset to Stage 1 at any time.

### R2. Zero-Step Turn-0 Environment Drift Recovery
WHEN a new agent session initializes (Turn-0)  
THEN `ce-ai workflow resume` MUST execute in under 15ms and output `RepoState`, identifying active Git branch, dirty uncommitted files, manifest SHA256 integrity, adoption marker validity, and open OpenSpec tasks.

### R3. OpenSpec Tasks Desync Detection
WHEN files are modified in git or branch history  
THEN the system MUST verify whether tasks in `tasks.md` are synchronized, emitting diagnostic warnings when code is committed but subtasks remain unchecked.

### R4. Branch-Scoped Workflow Isolation
WHEN working in repositories with multiple concurrent feature branches or worktrees  
THEN workflow state MUST be indexed by canonical repository path and branch name (`<root>::<branch>`), preventing cross-branch checkpoint clobbering.

## 3. Data Models & CLI Contracts

- `WorkflowStage` enum: `Ideation`, `OpenSpec`, `Plan`, `Work`, `Verify`, `Compound`, `Ship`.
- `RepoState` struct: serializable representation of live disk reality.
- CLI commands: `ce-ai workflow resume`, `ce-ai workflow status`, `ce-ai workflow checkpoint`.

## 4. Invariants & Operational Boundaries

- Automated stage inference MUST never downgrade or overwrite a higher manual checkpoint (Monotonic Provenance Guard).
- Turn-0 delivery MUST NOT perform network requests or heavy disk traversals.

### R1. Canonical Living Domain Specifications
<!-- promoted-from: change:living-system-specs-promotion date:2026-09-15 -->
WHEN `ce-ai` inspects `openspec/specs/`  
THEN the system MUST maintain canonical living domain specifications covering all core functional subsystems:
- `harnesses.md`: Multi-harness integration, adapters, managed configs, registration, plugins, skills.
- `workflow.md`: 7-stage compound engineering lifecycle, FSM, checkpoints, auto-progression, Turn-0 delivery.
- `doctor.md`: Diagnostic health check engine, probes, branch protection, doc debt, warnings.
- `state.md`: Configuration management (`state.json`), atomic writes, model assignments, profiles, workspace overrides (`.ce-ai.json`).
- `archive.md`: OpenSpec change archival, completion criteria (mechanical & STATUS-attested), generational compaction, milestone rollups.
- `gate.md`: Blocking gate check, structured validation receipts, observe-only and active modes.
- `project-adoption.md`: `init-prj`, `deinit-prj`, managed blocks in `AGENTS.md`/`CLAUDE.md`, adoption tiers.

Each living domain specification MUST contain valid YAML frontmatter specifying `title`, `domain`, `version`, and `last_updated`.

### R2. CLI Command: `ce-ai spec list`
<!-- promoted-from: change:living-system-specs-promotion date:2026-09-15 -->
WHEN the user executes `ce-ai spec list`  
THEN the system MUST scan `openspec/specs/*.md` and display a tabular summary listing the domain slug, title, version, last updated date, and requirement count.  
WHEN `--json` is provided  
THEN the system MUST output the list as structured JSON array conforming to `Vec<DomainSpecSummary>`.  
WHEN no specifications exist in `openspec/specs/`  
THEN the system MUST exit with code 0 and output `spec: no domain specifications found in openspec/specs/`.

### R3. CLI Command: `ce-ai spec show <domain>`
<!-- promoted-from: change:living-system-specs-promotion date:2026-09-15 -->
WHEN the user executes `ce-ai spec show <domain>`  
THEN the system MUST display the complete content of `openspec/specs/<domain>.md`.  
WHEN the specified `<domain>` does not exist  
THEN the system MUST exit with code 2 (`CeError::Usage`) and print `error: domain specification '<domain>' not found in openspec/specs/`.

### R4. CLI Command: `ce-ai spec validate`
<!-- promoted-from: change:living-system-specs-promotion date:2026-09-15 -->
WHEN the user executes `ce-ai spec validate`  
THEN the system MUST validate all markdown files in `openspec/specs/`:
1. Verify presence and non-emptiness of YAML frontmatter fields: `title`, `domain`, `version`, `last_updated`.
2. Verify presence of required section `## Capabilities & Requirements` (or `## 2. Capabilities & Requirements`).
3. Verify that file basename matches `domain.md` (e.g. `domain: harnesses` matches `harnesses.md`).  
WHEN any validation error is detected  
THEN the command MUST exit with code 6 (`CeError::Verification`) and print itemized error diagnostics.  
WHEN all domain specs are valid  
THEN the command MUST exit with code 0 and print `spec: all N domain specification(s) valid`.  
WHEN `--json` is specified  
THEN the command MUST serialize `SpecValidationReport` to stdout.

### R5. Requirement Delta Promotion (`ce-ai spec promote` / `ce-ai archive --promote`)
<!-- promoted-from: change:living-system-specs-promotion date:2026-09-15 -->
WHEN `ce-ai spec promote <change> [--domain <name>]` is executed, OR WHEN `ce-ai archive <feature> --promote` is executed  
THEN the system MUST:
1. Resolve the target domain from `--domain <name>` flag or fallback to `domain: <domain>` in `openspec/changes/<change>/spec.md` frontmatter.
2. If neither is present, fail with code 2 (`CeError::Usage`).
3. Extract all requirement headers (`### R...` or `### <Title>`) and their descriptive body text from `spec.md`.
4. Locate or initialize `openspec/specs/<domain>.md`.
5. Append or update the extracted requirements under `## Capabilities & Requirements`, annotating each promoted requirement with change provenance: `<!-- source: change:<change> date:<date> -->`.
6. Update the frontmatter `last_updated` date to current UTC date.
7. Persist mutations using atomic write (`crate::state::write_atomic`).  
WHEN `--dry-run` is passed  
THEN the system MUST preview the planned additions and modifications without mutating disk assets.

### R6. Doctor Health Check Probe Integration
<!-- promoted-from: change:living-system-specs-promotion date:2026-09-15 -->
WHEN `ce-ai doctor` runs  
THEN the system MUST probe `openspec/specs/` health:
1. If `openspec/specs/` is missing or empty, emit `doctor-warn: openspec/specs/ is empty — run 'ce-ai spec validate' or seed domain specs`.
2. For each active, unarchived change in `openspec/changes/`: if `spec.md` lacks a valid `domain: <domain>` frontmatter tag, emit `doctor-warn: active change '<feature>' lacks target domain mapping in spec.md`.
3. If any domain spec has validation findings, emit `doctor-warn: domain spec '<domain>' validation issue: <message>`.  
All findings in `doctor` MUST remain non-blocking (advisory only) so normal development is not interrupted.

### Requirement R1: Solution Inventory & Metadata Collection
<!-- promoted-from: change:solution-refresh-and-deduplication date:2026-09-15 -->
WHEN the system inspects `docs/solutions/`  
THEN it MUST recursively collect all `.md` solution files  
AND parse their YAML frontmatter into structured metadata containing `title`, `category`, `problem_type`, `tags`, `components`, `applies_when`, and `date`  
AND tokenize the title into normalized lowercase keyword tokens excluding standard stopwords.  
WHEN `docs/solutions/` does not exist or contains 0 markdown files  
THEN the inventory MUST return an empty collection without error.

### Requirement R2: Multi-Dimensional Solution Similarity Scoring
<!-- promoted-from: change:solution-refresh-and-deduplication date:2026-09-15 -->
WHEN computing similarity between two solution documents  
THEN the engine MUST calculate a composite similarity score between $0.0$ and $1.0$ based on:
1. Tag Jaccard similarity (weight: 0.35)
2. Component Jaccard similarity (weight: 0.25)
3. Title token Jaccard similarity (weight: 0.25)
4. Category exact match (weight: 0.15)  
WHEN two solutions share zero tags, components, title tokens, and categories  
THEN the computed similarity MUST be exactly `0.0`.  
WHEN two identical solutions are compared  
THEN the computed similarity MUST be exactly `1.0`.

### Requirement R3: Semantic Clustering Algorithm (`ce-ai doc cluster`)
<!-- promoted-from: change:solution-refresh-and-deduplication date:2026-09-15 -->
WHEN `ce-ai doc cluster` is executed  
THEN the system MUST:
1. Form an adjacency graph connecting pairs of solutions whose similarity exceeds `--threshold` (default: 0.40).
2. Partition the graph into connected components.
3. Filter for clusters containing at least `--min-size` solutions (default: 3).
4. Compute the cluster's dominant tags and assign an explanatory name and suggested `/ce-compound-refresh` scope hint.  
WHEN `--json` is specified  
THEN the command MUST serialize a `ClusterReport` to stdout.  
WHEN no clusters meet the `--min-size` criterion  
THEN the command MUST exit with code 0 and report that 0 clusters were found.

### Requirement R4: Solution Scope Refresh Directives (`ce-ai doc refresh`)
<!-- promoted-from: change:solution-refresh-and-deduplication date:2026-09-15 -->
WHEN `ce-ai doc refresh [--scope <name>]` is executed  
THEN the system MUST:
1. If `--scope` is provided, filter solutions whose category, title, tags, or path match the scope query.
2. If `--scope` is omitted, evaluate the highest-density cluster identified by the clustering algorithm.
3. Display the member documents, identifying the oldest vs newest solutions, and format an actionable `/ce-compound-refresh <scope>` invocation.  
WHEN `--dry-run` is passed  
THEN the command MUST preview the refresh recommendation without executing external actions.

### Requirement R5: Solution Library Linting (`ce-ai doc lint`)
<!-- promoted-from: change:solution-refresh-and-deduplication date:2026-09-15 -->
WHEN `ce-ai doc lint` is executed  
THEN the system MUST execute the solution drift probe across `docs/solutions/**/*.md`:
1. Validate required YAML frontmatter fields (`title`, `category` or `module`, `problem_type`, `tags`, `applies_when`).
2. Verify that backticked repository source code paths (`src/**/*.rs`, `tests/**/*.rs`) exist in the workspace.  
WHEN `--strict` is specified and one or more findings are detected  
THEN the command MUST exit with code 6 (`CeError::Verification`).  
WHEN `--json` is specified  
THEN the findings MUST be serialized as JSON.

### Requirement R6: Solution Library Statistics (`ce-ai doc stats`)
<!-- promoted-from: change:solution-refresh-and-deduplication date:2026-09-15 -->
WHEN `ce-ai doc stats` is executed  
THEN the system MUST output:
1. Total number of solution documents.
2. Distribution across categories (`architecture`, `workflow-issues`, etc.).
3. Top 10 most common tags and their frequency counts.
4. Total count of detected dense clusters.  
WHEN `--json` is specified  
THEN the command MUST output structured JSON conforming to `DocStatsReport`.

### Requirement R7: Doctor Health Check Integration
<!-- promoted-from: change:solution-refresh-and-deduplication date:2026-09-15 -->
WHEN `ce-ai doctor` is executed  
THEN the system MUST probe `docs/solutions/` for dense clusters:
1. If 1 or more clusters with $\ge 3$ overlapping solutions are found, emit non-blocking `doctor-info: solution library: N consolidation cluster(s) detected — run 'ce-ai doc cluster' for details`.
2. This probe MUST NOT block the doctor exit code (non-fatal advisory).
