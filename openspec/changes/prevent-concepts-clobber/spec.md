# Spec: Monotonic Accretion Guard & Anti-Clobber Protection for CONCEPTS.md

## Requirements

### R1: Python Monotonic Accretion Script (`scripts/validate-concepts.py`)
- **R1.1**: WHEN `validate-concepts.py` is invoked with a path to `CONCEPTS.md` where all entries present in git `HEAD` are preserved, THEN it SHALL exit with code `0` and print empirical statistics (`N entries (was M, +X added, 0 removed)`).
- **R1.2**: WHEN `validate-concepts.py` is invoked and one or more entries from `HEAD` are missing in the current working copy without an inline scrub tag, THEN it SHALL exit with code `1` and print all missing entry names to `stderr`.
- **R1.3**: WHEN an entry from `HEAD` is missing but is marked with `<!-- scrub: <Term> -->`, THEN it SHALL NOT be treated as an unexpected deletion and SHALL exit with code `0`.
- **R1.4**: WHEN invoked with `--allow-shrink`, THEN deletions SHALL be reported but SHALL NOT cause a non-zero exit code.
- **R1.5**: WHEN invoked outside a git repository or on a freshly created untracked `CONCEPTS.md`, THEN it SHALL validate that entries exist and exit with code `0`.

### R2: Native Rust Diagnostic Probe (`probe_concepts_drift`)
- **R2.1**: WHEN `probe_concepts_drift` is invoked on a repository where `CONCEPTS.md` has not lost any headings compared to `HEAD`, THEN it SHALL return `ProbeStatus::Clean`.
- **R2.2**: WHEN `CONCEPTS.md` in the working tree is missing entries that existed in git `HEAD`, THEN `probe_concepts_drift` SHALL return `ProbeStatus::Debt(ConceptsDriftFinding)`.
- **R2.3**: The finding SHALL contain the list of deleted entries, the count of HEAD entries, and the count of current entries.

### R3: CLI Quality Gate Integration (`ce-ai doc lint` & `ce-ai doctor`)
- **R3.1**: WHEN `ce-ai doc lint` runs on a repository with a clobbered `CONCEPTS.md`, THEN it SHALL report the destructive shrinkage findings.
- **R3.2**: WHEN `ce-ai doc lint --strict` is run and concepts drift is detected, THEN it SHALL return `CeError::Verification` with exit code `6`.
- **R3.3**: WHEN `ce-ai doctor` is executed and concepts drift is detected, THEN it SHALL emit a `doctor-warn` line alerting the user to restore or fix `CONCEPTS.md`.

### R4: Agent Operational Directives
- **R4.1**: In `ce-compound`, agents SHALL execute a `Read` of `CONCEPTS.md` before making any modifications to it.
- **R4.2**: Agents SHALL perform surgical edits (`Edit` / `replace_file_content` / append) to `CONCEPTS.md` and SHALL NOT execute full-file overwrites (`Write`).
- **R4.3**: Terminal success reports for `CONCEPTS.md` updates SHALL communicate empirical counts of entries before and after.
