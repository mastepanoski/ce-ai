---
date: 2026-10-01
topic: ce-ai-v2-architectural-migration-plan
status: ready
source: upstream-maintainer-critique
---

# Plan: CE-AI v2 Architectural Migration Plan & PRD

## Problem Frame & Scope

Following official design critique and boundary review from the upstream maintainer of Compound Engineering (`EveryInc/compound-engineering-plugin`), `ce-ai` faces fundamental architectural misalignments that threaten its maintainability, ergonomics, and upstream compatibility:

1. **State Ownership & Global Stage Cursor Divergence:** `ce-ai` currently maintains a global stage cursor in `state.json` and attempts to reconcile it against the repository via heuristics. The upstream model treats repository artifacts (plans, branches, commits, PRs, receipts) as the authoritative state. Storing state separately causes state drift, forces complex reconciliation code, and falsely assumes a rigid linear 7-stage workflow when real-world engineering includes skips, non-linear jumps, and specialized paths (`ce-debug`).
2. **Cross-Host Sync Antipattern:** `ce-ai` extracts and copies `skills/` out of release tarballs and enforces SHA256 file-level integrity loops. This bypasses per-host native packaging transformations, causes conflicts with native marketplace installations (`external-duplicate`), and creates dual sources of truth.
3. **Documentation Debt & Gate Friction:** Enforcing heavy formal OpenSpec documents (`proposal.md`, `spec.md`, `tasks.md`) prior to any write increases documentation debt and cognitive friction for small/debug tasks. Upstream Compound Engineering already addresses doc hygiene natively via `ce-compound`, `ce-compound-refresh`, and `compound audit`.
4. **Concrete Schema & Coupling Incompatibilities:** Immediate bugs in `ce-ai`'s current assumptions:
   - Solution frontmatter schema divergence (`title`, `tags`, `applies_when` vs upstream `module`, `date`, `problem_type`, `component`, `severity`).
   - Component attribute parsing (`components` vs upstream `component` + `related_components`).
   - Legacy brainstorming path (`docs/brainstorms/` vs upstream `docs/plans/` requirements-only plans).
   - Hardcoded `docs/` paths ignoring upstream `docs_root` in `.compound-engineering/config.yaml`.
   - File extension dead-path detection restricted only to `.rs` files.
   - Pi extension entrypoint collisions at `.pi/extensions/compound-engineering.ts`.
   - Fragile substring-based `ce-debug` gate exemptions.
5. **Lack of a Delimited Compatibility Boundary:** Current implementation relies on fragile internals (skill prose, file layout, bundled references, file hashes) rather than stable integration points (skill names, mode tokens like `mode:return-to-caller`, versioned schemas, and native host installers).

This plan establishes the comprehensive **CE-AI v2 Architecture PRD and Migration Plan**, repositioning `ce-ai` from an authoritative workflow gatekeeper to a **Cross-Host Operational Companion for Compound Engineering**:
- **CE owns engineering semantics and workflow artifacts.**
- **Hosts own native execution and packaging.**
- **CE-AI coordinates environment readiness, version governance, native installation orchestration, and advisory workflow observation.**

## Requirements Traceability

- **RF-1 (Probabilistic Execution vs. Deterministic State):** Transition FSM from authoritative state gatekeeper to read-only advisory observation model; deprecate global stage cursor; align state recovery with repo artifacts.
- **RF-2 (Fleet Version Governance & Native Installers):** Retire file-level skill copying/hashing; implement native host installer drivers (`claude`, `codex`, `opencode`, `pi`) and released plugin version pinning.
- **RF-3 (Documentation Debt & Integrated Hygiene):** Relax rigid write gates for lightweight/debug workflows; integrate with upstream `compound audit` and schema validators instead of duplicating checks.
- **RF-4 (Coupling & Schema Corrections):** Resolve 7 concrete coupling bugs (frontmatter schema, component fields, `docs_root`, brainstorm directory migration, dead path file types, Pi extension paths, regex gate exemptions).
- **RF-5 (Stable Integration Contracts):** Define Rust CE compatibility layer (`CeRelease`, `CeCapabilities`, `CeSkillContract`, `CeArtifactSchema`, `CeNativeInstaller`, `CeDocsRoot`) built exclusively on stable integration points.

## Implementation Units

### Unit 1: Formal OpenSpec Change Package Definition
- **Files:**
  - `openspec/changes/ce-ai-v2-architecture-prd/proposal.md`
  - `openspec/changes/ce-ai-v2-architecture-prd/exploration.md`
  - `openspec/changes/ce-ai-v2-architecture-prd/design.md`
  - `openspec/changes/ce-ai-v2-architecture-prd/spec.md`
  - `openspec/changes/ce-ai-v2-architecture-prd/tasks.md`
- **Target:** ~650 LOC
- **Approach:**
  - Formally document problem, exploration, architectural tradeoffs, formal requirements, and execution tasks according to `AGENTS.md` Invariant #6.
  - Document ruled-out architectures (retaining global stage cursor, continuing tarball file scraping).

### Unit 2: Comprehensive Architecture PRD Document
- **Files:** `docs/architecture/ce-ai-v2-architecture-prd.md`
- **Target:** ~550 LOC
- **Approach:**
  - Detail executive summary, mission pivot, core domain boundaries, and architectural pillars.
  - Formulate the CE Compatibility Layer interface design in Rust.
  - Map deprecation, refactoring, and upstream delegation matrix.
  - Outline multi-phase migration roadmap (v1.x maintenance -> v2.0-alpha -> v2.0 GA).

### Unit 3: Domain Vocabulary Capture in `CONCEPTS.md`
- **Files:** `CONCEPTS.md`
- **Target:** ~60 LOC
- **Approach:**
  - Monotonically accrete new architectural concepts:
    - `Cross-Host Operational Companion`
    - `Advisory Workflow Observation Engine`
    - `Native Harness Installer Orchestrator`
    - `Upstream Schema Compatibility Layer`
    - `Configurable Docs Root Resolution`
  - Ensure zero deletions of existing vocabulary using surgical replacement.

### Unit 4: Quality & Integrity Gate Validation
- **Files:** None (CLI verification)
- **Target:** 0 LOC
- **Approach:**
  - Run `cargo fmt --check`.
  - Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - Run `cargo test`.
  - Validate concept integrity with `python3 scripts/validate-concepts.py` or doc linting.

## Key Technical Decisions (KTDs)

- **KTD 1: Advisory Workflow FSM over Authoritative Gatekeeper:**
  - *Decision:* `state.json` will no longer hold an authoritative `current_stage` cursor that blocks or governs writes. State is observed and derived dynamically from repository artifacts (`docs/plans/`, git branches, commits, PR status, run receipts).
  - *Rationale:* Eliminates drift, removes complex reconciliation logic, and respects non-linear engineering flows (`ce-debug`, trivial fixes, exploratory tasks).
- **KTD 2: Native Host Installer Orchestration over File Syncing:**
  - *Decision:* CE-AI will delegate package installation to each host's native mechanism (e.g. Claude Code plugin marketplace, OpenCode extensions, Pi extensions) and focus on version pinning and fleet auditing.
  - *Rationale:* Avoids clobbering per-target rewrites, prevents `external-duplicate` confusion, and provides a clear single source of truth for plugin code.
- **KTD 3: Explicit Compatibility Layer in Rust:**
  - *Decision:* Encapsulate all upstream CE knowledge within typed structs (`CeRelease`, `CeArtifactSchema`, `CeDocsRoot`) rather than scattering heuristics across commands.
  - *Rationale:* Insulates CE-AI from upstream internal changes; builds strictly on documented stable contracts.

## Verification & Acceptance Scenarios

- **Scenario 1:** The OpenSpec change package in `openspec/changes/ce-ai-v2-architecture-prd/` contains all 5 required files with complete specification and acceptance criteria.
- **Scenario 2:** `docs/architecture/ce-ai-v2-architecture-prd.md` provides an exhaustive technical specification addressing all 5 maintainer critique points with concrete migration paths.
- **Scenario 3:** `CONCEPTS.md` contains the new domain terms without any clobbering or loss of previous definitions.
- **Scenario 4:** All CI/CD test gates (`cargo fmt`, `cargo clippy`, `cargo test`) execute with 100% green status.
