# Proposal: Living System Specifications & Delta Promotion (`openspec/specs/`)

## 1. Problem Statement

In the OpenSpec specification-driven methodology, repository documentation is bifurcated into two essential layers:
1. **Transient Delta Changes (`openspec/changes/<change>/`)**: "Flight recorders" that specify *what is changing* during a specific feature or bugfix cycle (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`). Once implemented and verified, these change folders are archived.
2. **Durable Living System Specifications (`openspec/specs/<domain>.md`)**: The authoritative, evergreen system contract describing *how the system behaves today* across key functional domains.

Currently in `ce-ai`:
- The directory `openspec/specs/` exists but is completely empty (0 files).
- All architectural knowledge and functional requirements are fragmented across 110+ archived change packages in `openspec/changes/archive/`.
- When AI coding agents (or human engineers) need to understand the authoritative contract of a subsystem (e.g. `harnesses`, `doctor`, `workflow`, `state`, `archive`, `gate`, `project-adoption`), they must search through dozens of historical proposals and delta specs, risking hallucinations, ingesting superseded rules, and consuming massive token context.
- Furthermore, `ce-ai archive` safely archives completed changes to `openspec/changes/archive/`, but lacks a mechanism to promote or sync validated delta requirements into living domain specs in `openspec/specs/`.

## 2. In-Scope Boundaries

1. **Foundational Living Domain Specifications**:
   - Establish the initial canonical living domain specifications in `openspec/specs/`:
     - `harnesses.md`: Multi-harness integration, adapters, managed configs, registration, plugins, skills.
     - `workflow.md`: 7-stage compound engineering lifecycle, FSM, checkpoints, auto-progression, Turn-0 delivery.
     - `doctor.md`: Diagnostic health check engine, probes, branch protection, doc debt, warnings.
     - `state.md`: Configuration management (`state.json`), atomic writes, model assignments, profiles, workspace overrides (`.ce-ai.json`).
     - `archive.md`: OpenSpec change archival, completion criteria (mechanical & STATUS-attested), generational compaction, milestone rollups.
     - `gate.md`: Blocking gate check, structured validation receipts, observe-only and active modes.
     - `project-adoption.md`: `init-prj`, `deinit-prj`, managed blocks in `AGENTS.md`/`CLAUDE.md`, adoption tiers.

2. **Domain Specification Schema & Linter**:
   - Define a standardized markdown schema for domain specifications in `openspec/specs/` with structured sections:
     - Frontmatter: `title`, `domain`, `version`, `last_updated`.
     - `## 1. Overview & Architectural Boundaries`
     - `## 2. Capabilities & Requirements (Given/When/Then, RFC 2119)`
     - `## 3. Data Models & CLI Contracts`
     - `## 4. Operational Invariants & Security Boundaries`
   - Implement validation in `ce-ai spec validate` / `ce-ai doctor` ensuring specs adhere to the schema and contain zero broken internal references.

3. **CLI Commands (`ce-ai spec`)**:
   - `ce-ai spec list`: List all living domain specifications, their domains, requirements count, and last updated dates (with `--json` support).
   - `ce-ai spec show <domain>`: Display the full specification for a given domain.
   - `ce-ai spec validate`: Validate all domain specifications against schema rules.

4. **Automated Promotion Engine (`ce-ai archive --promote` / `ce-ai spec promote`)**:
   - Allow completed OpenSpec changes to declare target domain(s) in `spec.md` frontmatter or via CLI `--domain <name>`.
   - On archival, provide seamless promotion of delta requirements into the targeted living domain spec, appending or updating requirements while preserving structure.

5. **Doctor Health Check Integration**:
   - Non-blocking `doctor` probe auditing domain specification coverage and alerting when an active change lacks a domain mapping or when a domain spec is missing or malformed.

## 3. Out-of-Scope Boundaries

- Full NLP semantic merging of freeform conversational prose between arbitrary documents (semantic updates are guided by structured requirement IDs and sections).
- Modifying historical compacted milestone tarballs.
- Replacing external user documentation (`docs/user-guide/`, `README.md`).

## 4. Risk Evaluation & Mitigations

| Risk | Impact | Likelihood | Mitigation |
| :--- | :--- | :--- | :--- |
| **Accidental Overwrites** | Loss of existing spec details | Low | Domain spec updates use append/merge logic with atomic writes (`write_atomic`); dry-run mode previews exact diffs. |
| **Schema Rigidness** | Friction when authoring specs | Medium | Flexible markdown structure with mandatory high-level headings and lenient sub-sections. |
| **Performance Overhead** | Slower `doctor` or `archive` execution | Low | In-memory parsing of local markdown files; cached file reads keep probe runtimes under 10ms. |

## 5. Success Criteria

1. Canonical living specifications created in `openspec/specs/` for all core `ce-ai` domains.
2. New CLI commands `ce-ai spec list`, `ce-ai spec show`, `ce-ai spec validate` functional and tested.
3. `ce-ai archive` supports `--promote` and auto-promotes requirements from `spec.md`.
4. `ce-ai doctor` verifies living specs health with zero false positives.
5. 100% test pass rate (`cargo test`), zero clippy warnings (`cargo clippy -- -D warnings`), strict formatting (`cargo fmt --check`).
