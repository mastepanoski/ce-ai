---
title: "Living System Specifications and Delta Promotion Engine"
category: "architecture"
date: "2026-09-15"
problem_type: "architecture"
tags:
  - openspec
  - living-specs
  - delta-promotion
  - requirements
  - doc-debt
  - doctor
  - archive
components:
  - commands::spec
  - commands::workflow
  - commands::doctor
applies_when: "Managing canonical living specifications, promoting delta requirements from completed OpenSpec changes, or auditing specification health"
---

# Living System Specifications and Delta Promotion Engine

## Context

In standard Spec-Driven Development (SDD) and Compound Engineering, feature changes are authored as self-contained delta packages under `openspec/changes/<feature_name>/` containing `proposal.md`, `exploration.md`, `design.md`, `spec.md`, and `tasks.md`.

While delta packages excel at scoping atomic iterations and tracking incremental verification tasks, they introduce a fundamental documentation debt problem once merged and archived:
1. **Delta Ephemerality**: After a feature is archived to `openspec/changes/archive/`, its `spec.md` represents a past delta rather than the current truth of the system.
2. **Context Dilution**: Subsequent agent turns looking to understand the overall behavior of a subsystem (e.g., how harnesses work, how doctor diagnostics run, or how workflows transition) have to grep across dozens of past change directories or reverse-engineer source code.
3. **Spec-to-Code Drift**: As subsequent changes land, historical delta specs accumulate obsolete statements that no longer describe active capabilities.

To resolve this dichotomy between ephemeral delta specifications and the need for canonical system-level truth, `ce-ai` introduced **Living System Specifications** (`openspec/specs/`) and the **Delta Promotion Engine** (`ce-ai spec` and `ce-ai archive --promote`).

---

## Architecture & Implementation

### 1. Canonical Domain Specifications (`openspec/specs/`)

Living specifications are permanent, evolving domain contracts stored at the repository root under `openspec/specs/`. Each domain spec models an entire subsystem and is seeded with:
- **YAML Frontmatter**: Metadata defining `id`, `title`, `domain`, `status` (`canonical`), `version`, `owner`, and `last_updated`.
- **Purpose**: Scope boundaries and subsystem responsibilities.
- **System-Level Invariants**: Hard rules that must always hold across all changes (e.g., atomic file writes, non-clobbering of user configs).
- **Domain Requirements**: Granular, stable requirements with structured requirement IDs (`REQ-<DOMAIN>-*`).

The system seeds 7 foundational domain specifications:
- `openspec/specs/harnesses.md` (Domain: `harnesses`)
- `openspec/specs/workflow.md` (Domain: `workflow`)
- `openspec/specs/doctor.md` (Domain: `doctor`)
- `openspec/specs/state.md` (Domain: `state`)
- `openspec/specs/archive.md` (Domain: `archive`)
- `openspec/specs/gate.md` (Domain: `gate`)
- `openspec/specs/project-adoption.md` (Domain: `project-adoption`)

### 2. Delta Promotion Engine (`src/commands/spec.rs`)

The promotion engine bridges ephemeral delta specifications in `openspec/changes/<change>/spec.md` and canonical living specifications in `openspec/specs/<domain>.md`.

#### Requirement Extraction & Parsing
The extractor parses markdown headings conforming to:
```markdown
### Requirement <id>: <title>
```
Extracting the requirement title, ID, and full requirement narrative block.

#### Intelligent Domain Matching
If `--domain` is specified, the target domain file is explicitly selected. If omitted, the engine infers the domain by scanning candidate specs in `openspec/specs/` and matching keyword overlap across titles, filenames, and domain tags.

#### Deduplication & Promotion
When merging requirements:
- Existing requirements with matching IDs or titles are suppressed to avoid duplicate proliferation.
- New requirements are appended under a distinct promotional audit section:
  ```markdown
  ### Promoted from <change> (YYYY-MM-DD)
  ```
- File mutations utilize `crate::state::write_atomic` to guarantee atomic writes with zero risk of partial file corruption on process termination.

### 3. CLI Handlers (`ce-ai spec`)

The `spec` subcommand provides full lifecycle management:
- `ce-ai spec list [--json]`: Lists all canonical living domain specs with their version, requirement count, and last update date.
- `ce-ai spec show <domain> [--requirements-only]`: Displays domain specification metadata and requirements.
- `ce-ai spec validate [--strict]`: Audits all `openspec/specs/*.md` files for frontmatter completeness, valid schema fields, and markdown structure.
- `ce-ai spec promote <change> [--domain <domain>]`: Executes standalone delta promotion from an active or completed change package.

### 4. Workflow Archival Integration (`ce-ai archive --promote`)

Archival (`ce-ai archive <change>` and `ce-ai workflow archive <change>`) is extended with:
- `--promote`: Automatically triggers delta requirement promotion to living domain specs prior to directory relocation.
- `--domain <name>`: Optional target domain specification override.

When `--dry-run` is active, the engine previews the promotional delta without mutating disk files.

### 5. Non-Blocking Health Diagnostics (`probe_specs_health`)

`ce-ai doctor` includes a dedicated health probe (`probe_specs_health`) that audits:
- Presence of canonical domain specifications in `openspec/specs/`.
- Schema validity of domain specifications (valid YAML frontmatter).
- Unpromoted requirements in completed active changes.

Advisories are surfaced with clean diagnostic messages without blocking hard gates.

---

## Verification & Key Learnings

1. **Clippy Print Literal Invariant**:
   In Rust linter standards, `println!("{:<18} ... {}", ..., "TITLE")` triggers `clippy::print_literal`. Static column headers must be placed directly in format strings.
2. **Atomic Modification of Documentation**:
   Treating documentation files as code artifacts subject to `write_atomic` prevents race conditions when multi-agent workflows inspect or update specifications concurrently.
3. **Audit Trail Accountability**:
   Appending delta requirements with explicit `Promoted from <change> (date)` headers provides a transparent provenance ledger without sacrificing subsystem readability.
