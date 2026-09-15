# Proposal: Solution Library Drift Repair & Frontmatter Schema Normalization

## Problem Statement
In Compound Engineering, `docs/solutions/` acts as the repository's long-term institutional memory, indexed and consumed by AI agents (`learnings-researcher`, `ce-compound-refresh`, `ce-debug`) and developers. 

As the codebase rapidly evolved across 150+ releases and multiple major architectural refactors (such as the modularization of `src/tui.rs` into `src/tui/` and the deprecation of `src/harness/generic_json.rs`), 9 solution documents accumulated dead file references. Furthermore, with the introduction of the diagnostic engine in Increment 2, `probe_solution_drift` enforces a standardized YAML frontmatter contract (`title`, `category`/`module`, `problem_type`, `tags`, `applies_when`). Currently, 70 of the 74 solution documents lack one or more required frontmatter attributes (most notably `applies_when` and `title`), generating persistent `doctor-warn:` advisories in `ce-ai doctor` and cluttering the Turn-0 session start banner.

## Scope Boundaries

### In-Scope
1. **Dead Path Remediation**:
   - Repair all 9 dead file references across 9 solution files (e.g. updating `src/tui.rs` to point to the active modular implementation in `src/tui/app.rs` or `src/tui/`, and updating references to superseded modules such as `src/harness/generic_json.rs`).
2. **YAML Frontmatter Normalization**:
   - Audit and normalize all 74 solution files in `docs/solutions/` to satisfy the full schema: `title`, `category` (or `module`), `problem_type`, `tags`, and `applies_when`.
   - Derive accurate, high-signal values for `title` and `applies_when` based on each document's core content and problem domain.
3. **Deterministic Repair Tooling**:
   - Provide an idempotent, safe repair script in `scripts/` or scratch to ensure repeatable frontmatter normalization and path verification.
4. **Zero-Warning Doctor Verification**:
   - Verify that `probe_solution_drift` returns `ProbeStatus::Clean`.
   - Ensure `ce-ai doctor` and `ce-ai workflow status` emit zero solution drift warnings.

### Out-of-Scope
- Rewriting or summarizing technical prose within the solution bodies (beyond fixing broken file paths).
- Deleting or merging historical solutions (deferred to dedicated `ce-compound-refresh` editorial cycles).
- Modifying the core diagnostic probe logic in `src/commands/workflow.rs`.

## Risk Evaluation & Mitigation
- **Risk:** Unintentional alteration of document formatting or markdown structure during automated frontmatter patching.
  - *Mitigation:* The normalizer must only mutate the YAML header block between `---` delimiters (or prepend one if completely missing), leaving document body text strictly intact.
- **Risk:** Introducing invalid or non-existent replacement paths.
  - *Mitigation:* Replacement paths must be verified against the actual working tree (`std::path::Path::exists()`).

## Success Criteria
- [ ] 0 files in `docs/solutions/` contain dead source code references (`src/**/*.rs`, `tests/**/*.rs`).
- [ ] 100% of solution files in `docs/solutions/` contain valid YAML frontmatter with all 5 required fields.
- [ ] `cargo run -- doctor` outputs `doctor: ok` with zero `doctor-warn: solution` warnings.
- [ ] `cargo run -- workflow status` reports clean documentation debt.
- [ ] 100% of existing tests pass (`cargo test`).
