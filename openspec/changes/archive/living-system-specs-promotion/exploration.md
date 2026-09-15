# Exploration: Living System Specifications Architecture & Promotion Mechanics

## 1. Current State Investigation

In `ce-ai`, OpenSpec has served as the backbone of spec-driven development across 110+ delivered features and fixes. However, the repository exhibits a classic "specification drift" pattern:
1. **Delta Clutter:** Every feature writes 5 files under `openspec/changes/<feature>/`. Once archived, these files become static historical records.
2. **Empty Living Contract:** The directory `openspec/specs/` was provisioned early in the project's history but remained empty.
3. **Cognitive Overhead for Agents:** When an AI agent needs to know "What are the rules for harness registration?", it has to search through `openspec/changes/archive/milestones/2026-Q3.md`, past change folders, or reverse-engineer `src/harness/registration.rs`.
4. **Lack of Promotion Tooling:** When `ce-ai archive <feature>` runs, it moves the change directory to `openspec/changes/archive/<feature>`, updates `archive/README.md`, and reconciles `state.json`. It does not extract or elevate durable system capabilities into `openspec/specs/`.

## 2. Evaluated Options

### Option A: Manual Spec Authoring (Static Documentation Only)
- *Approach:* Hand-write static markdown files in `openspec/specs/` and rely on human memory / agent instructions in PRs to update them.
- *Pros:* Zero Rust code required.
- *Cons:* Guaranteed to rot. Without CLI commands (`spec list`, `spec validate`, `archive --promote`) or automated `doctor` probes, `specs/` will quickly drift from source code, creating a secondary documentation debt problem.

### Option B: Deep AST Extraction (Auto-Generating Specs from Rust Code)
- *Approach:* Parse Rust source code with `syn` or `rustdoc` to generate markdown specifications.
- *Pros:* High alignment with code symbols.
- *Cons:* Specs are supposed to describe functional requirements (RFC 2119, Given/When/Then), not code docstrings. AST parsing is brittle, expensive, and fails on non-Rust projects adopted by `ce-ai`. Rejected during ideation filtering (Idea 35).

### Option C: Structured Domain Specs + CLI Promotion Engine (Recommended)
- *Approach:*
  1. Seed canonical, human-audited domain specifications in `openspec/specs/` covering the core subsystems.
  2. Implement a CLI module `src/commands/spec.rs` providing `ce-ai spec list`, `show`, `validate`, and `promote`.
  3. Enhance `ce-ai archive` with automatic requirement promotion: when archiving a change whose `spec.md` specifies a target domain, the CLI integrates new requirements into the domain specification.
  4. Add a non-blocking `doctor` probe verifying domain specification validity and coverage.
- *Pros:* Balances developer ergonomics with automated governance. Keeps `openspec/specs/` clean, living, and authoritative while respecting the OpenSpec specification standard.

## 3. Tradeoffs & Architectural Decisions

### Decision 1: Specification Granularity (Subsystems vs Single Monolith)
- Rather than a single massive `system.md`, we partition specifications by cohesive functional domains:
  - `harnesses.md` (~12 AI harnesses, adapters, managed trees)
  - `workflow.md` (7-stage cycle, FSM, Turn-0 delivery, auto-checkpoints)
  - `doctor.md` (diagnostic probes, doctor-warn advisories, branch protection)
  - `state.md` (atomic writes, state.json, model assignments, overrides)
  - `archive.md` (mechanical & STATUS-attested archival, compaction)
  - `gate.md` (blocking gate check, receipts, exemptions)
  - `project-adoption.md` (init-prj, deinit-prj, managed blocks)
- *Rationale:* Agents can retrieve only the domain spec relevant to their task, minimizing token consumption.

### Decision 2: Promotion Format & Merging Strategy
- OpenSpec requirements in `spec.md` follow a structured format:
  ```markdown
  ### R1. Requirement Title
  WHEN condition
  THEN system MUST ...
  ```
- During promotion:
  - If a requirement with the same ID or title already exists, it is updated.
  - If it is new, it is appended under the appropriate capability section with a reference to the promoting change.
  - Frontmatter `last_updated` is refreshed.
  - All writes use `crate::state::write_atomic` to guarantee zero file corruption.

### Decision 3: Non-Blocking Doctor Diagnostics
- If a domain specification has syntax errors or missing required sections, `ce-ai doctor` emits a non-blocking `doctor-warn:` finding with remediation guidance rather than failing builds with exit code 6.
