# Proposal: Monotonic Accretion Guard & Anti-Clobber Protection for CONCEPTS.md

## Problem Statement
In `ce-compound`, during Phase 2.4 ("Vocabulary Capture"), when domain terms or concepts are extracted and documented in `CONCEPTS.md`, agents frequently perform destructive file writes (`Write` / `write_to_file`) with only the newly extracted terms, inadvertently clobbering all pre-existing entries in `CONCEPTS.md`.

Because existing doc validation scripts (`validate-frontmatter.py`, `validate-doc-claims.py`) and CLI probes (`ce-ai doc lint`, `probe_solution_drift`) audit exclusively files under `docs/solutions/*.md`, destructive overwrites of `CONCEPTS.md` go completely unnoticed by automated quality gates. Furthermore, completion reporting blindly prints "CONCEPTS.md: updated — N added" based on agent intent rather than empirical diff inspection, hiding data loss from developers.

## In-Scope
1. **Mechanical Monotonic Accretion Guard (`scripts/validate-concepts.py`)**:
   - Provide a pure-Python-3 (no external dependencies) validation script that compares working-tree `CONCEPTS.md` against git `HEAD`.
   - Parse concept headings (both `#`/`##`/`###` headings and `- **Term**:` bullet formats).
   - Detect destructive shrinkage: flag any heading from `HEAD` that has been removed or truncated without an explicit scrub directive (`<!-- scrub: <Term> -->` or `--allow-shrink`).
   - Emit empirical diff statistics: total current entries, entries in `HEAD`, additions count, and deletions count.
2. **Native Rust Diagnostic & Verification Probes (`ce-ai doc lint` & `ce-ai doctor`)**:
   - Add `probe_concepts_drift` to `src/commands/workflow.rs`, incorporating it into `DocDebtReport`.
   - Update `ce-ai doc lint` to audit `CONCEPTS.md` for destructive shrinkage alongside `docs/solutions/`.
   - Enforce `--strict` failure (`CeError::Verification`) in `ce-ai doc lint` when concepts clobber is detected.
   - Surface findings in `ce-ai doctor` as actionable health warnings.
3. **Agent Skill Contract Hardening**:
   - Mandate pre-read (`Read`) before modifying `CONCEPTS.md`.
   - Explicitly instruct agents to prefer surgical non-destructive editing (`Edit` / `replace_file_content` / append) over full-file `Write`.
   - Mandate empirical before/after entry count reporting.

## Out-of-Scope
- Re-architecting `CONCEPTS.md` markdown format or semantic taxonomy.
- Automatic semantic merging of contradictory glossary definitions (remains agent editorial judgment).
- Unrelated doc debt or OpenSpec probes.

## Risk Evaluation
- **Risk**: Projects with uncommitted initial `CONCEPTS.md` (no git `HEAD` version) might fail validation.
  - *Mitigation*: If `CONCEPTS.md` is untracked or newly created, verify positive entry count without requiring a git `HEAD` comparison.
- **Risk**: Legitimate removal of obsolete terms blocked by guard.
  - *Mitigation*: Support deliberate term removal via inline comment directive (`<!-- scrub: <Term> -->`) or `--allow-shrink` flag in the script.

## Success Criteria
1. `validate-concepts.py` catches destructive deletions and outputs empirical before/after statistics.
2. `ce-ai doc lint` detects and reports destructive shrinkage in `CONCEPTS.md` and fails when run with `--strict`.
3. `ce-ai doctor` includes `CONCEPTS.md` shrinkage diagnostics in its report.
4. Unit and CLI integration tests comprehensively verify monotonic accretion and error states.
