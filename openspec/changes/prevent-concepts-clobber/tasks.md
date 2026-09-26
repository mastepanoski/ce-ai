# Tasks: Monotonic Accretion Guard & Anti-Clobber Protection for CONCEPTS.md

Total Estimated Impact: ~310 LOC (~80-100 LOC per work unit).

- [x] 1.0 Implement Portable Python Monotonic Accretion Script (`scripts/validate-concepts.py`) (~95 LOC)
  - [x] 1.1 Implement entry extractor for markdown headings (`## `, `### `) and list items (`- **Term**:`) ignoring preamble.
  - [x] 1.2 Implement git HEAD comparison via `git show HEAD:<path>` with fallback for untracked/offline files.
  - [x] 1.3 Add scrub tag recognition (`<!-- scrub: <Term> -->`) and `--allow-shrink` flag.
  - [x] 1.4 Format empirical diff summary output and exit code contract (0 = pass, 1 = clobber, 2 = usage).

- [x] 2.0 Native Rust Probe & Doctor Integration (~95 LOC)
  - [x] 2.1 Define `ConceptsDriftFinding` struct in `src/commands/workflow.rs` and add `concepts_drift` field to `DocDebtReport`.
  - [x] 2.2 Implement `probe_concepts_drift(repo_root: &Path, git_available: bool) -> ProbeStatus<ConceptsDriftFinding>`.
  - [x] 2.3 Wire `concepts_drift` warning into `src/commands/doctor.rs` with actionable remediation guidance.
  - [x] 2.4 Add unit tests for `probe_concepts_drift` covering clean accretion and destructive shrinkage.

- [x] 3.0 CLI Doc Lint Strict Gate Integration (~80 LOC)
  - [x] 3.1 Integrate `probe_concepts_drift` into `src/commands/doc.rs` (`run_doc_lint`).
  - [x] 3.2 Ensure `--strict` mode returns `CeError::Verification` (exit code 6) upon concepts clobber detection.
  - [x] 3.3 Ensure JSON output includes `concepts_drift` in machine-readable format.
  - [x] 3.4 Add integration tests for `ce-ai doc lint` and `ce-ai doc lint --strict` in `tests/cli.rs`.

- [x] 4.0 Agent Operational Directives & Documentation (~40 LOC)
  - [x] 4.1 Update `CONCEPTS.md` with definition for `Monotonic Concept Accretion`.
  - [x] 4.2 Document anti-clobber directives and empirical verification in `AGENTS.md`.
  - [x] 4.3 Update `SKILL.md` (and reference files) for `ce-compound` to enforce pre-read and surgical editing.
