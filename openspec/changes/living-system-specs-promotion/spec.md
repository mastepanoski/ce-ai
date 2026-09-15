---
title: "Living System Specifications & Delta Promotion Specification"
domain: workflow
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: Living System Specifications & Delta Promotion

## Requirements (RFC 2119 / Given-When-Then)

### R1. Canonical Living Domain Specifications
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
WHEN the user executes `ce-ai spec list`  
THEN the system MUST scan `openspec/specs/*.md` and display a tabular summary listing the domain slug, title, version, last updated date, and requirement count.  
WHEN `--json` is provided  
THEN the system MUST output the list as structured JSON array conforming to `Vec<DomainSpecSummary>`.  
WHEN no specifications exist in `openspec/specs/`  
THEN the system MUST exit with code 0 and output `spec: no domain specifications found in openspec/specs/`.

### R3. CLI Command: `ce-ai spec show <domain>`
WHEN the user executes `ce-ai spec show <domain>`  
THEN the system MUST display the complete content of `openspec/specs/<domain>.md`.  
WHEN the specified `<domain>` does not exist  
THEN the system MUST exit with code 2 (`CeError::Usage`) and print `error: domain specification '<domain>' not found in openspec/specs/`.

### R4. CLI Command: `ce-ai spec validate`
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
WHEN `ce-ai doctor` runs  
THEN the system MUST probe `openspec/specs/` health:
1. If `openspec/specs/` is missing or empty, emit `doctor-warn: openspec/specs/ is empty — run 'ce-ai spec validate' or seed domain specs`.
2. For each active, unarchived change in `openspec/changes/`: if `spec.md` lacks a valid `domain: <domain>` frontmatter tag, emit `doctor-warn: active change '<feature>' lacks target domain mapping in spec.md`.
3. If any domain spec has validation findings, emit `doctor-warn: domain spec '<domain>' validation issue: <message>`.  
All findings in `doctor` MUST remain non-blocking (advisory only) so normal development is not interrupted.
