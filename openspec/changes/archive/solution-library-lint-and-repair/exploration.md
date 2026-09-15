# Exploration: Solution Library Drift Repair & Frontmatter Schema Normalization

## Context & Background
`ce-ai doctor` incorporates `probe_solution_drift` (developed in Increments 1 & 2), which checks:
1. **Frontmatter Schema Compliance:**
   - Requires top-level YAML keys:
     - `title`
     - `category` (or `module`)
     - `problem_type`
     - `tags`
     - `applies_when`
2. **Dead Source Code Path Checks:**
   - Scans backticked strings outside code blocks matching `src/**/*.rs` or `tests/**/*.rs`.
   - Strips line number suffixes (`:123`, `:86`) and anchors (`#...`).
   - Asserts physical file existence in `repo_root`.

An initial empirical scan of the 74 solution files in `docs/solutions/` reveals:
- 70 files lack complete YAML frontmatter (37 missing `title`, 49 missing `applies_when`, 20 missing `problem_type`, 1 missing `category`, 2 missing `tags`).
- 9 files cite non-existent source paths:
  - 6 files cite `src/tui.rs` (which was split into `src/tui/app.rs`, `src/tui/mod.rs`, `src/tui/views/`, etc.).
  - 2 files cite `src/harness/generic_json.rs` (which was superseded by native harness implementations).
  - 1 file cites `tests/mod_tests.rs` (which was refactored into domain-scoped integration tests).

## Evaluated Approaches

### Approach A: Manual Per-File Edits
- *Description:* Manually open and edit all 74 files one by one in the editor.
- *Pros:* Human inspection of each document.
- *Cons:* Extremely high token consumption, time-consuming, prone to human inconsistency across 74 files, and high risk of formatting errors.
- *Decision:* **Rejected.**

### Approach B: Deterministic Scripted Normalization with Semantic Extraction + Targeted Path Fixing
- *Description:*
  1. Develop an automated normalization script in Python that parses existing YAML frontmatter, retains all existing keys and formatting, extracts the document title from the first `# <Title>` heading, infers missing `problem_type` based on directory name (e.g. `bugfixes/` -> `bugfix`, `architecture/` -> `architectural_refactor`), generates a precise `applies_when` context clause based on the problem summary, and ensures tags and categories are fully populated.
  2. Map dead paths (`src/tui.rs` -> `src/tui/app.rs` or `src/tui/mod.rs`, `src/harness/generic_json.rs` -> `src/harness/mod.rs`, `tests/mod_tests.rs` -> `tests/cli.rs`) according to the active codebase architecture.
  3. Validate using the project's own `probe_solution_drift` via `cargo run -- doctor`.
- *Pros:* 100% deterministic, idempotent, preserves all existing content without corruption, fast, verifiable by the Rust compiler and doctor probe.
- *Decision:* **Selected.**

## Architectural Tradeoffs
- **Path Substitution vs Context Framing:** In historical notes (e.g. `refactor-transversal-maintainability-2026-08-27.md`), text states: `"El archivo src/tui.rs de 1.791 líneas se dividió en 6 submódulos..."`. Because `probe_solution_drift` checks backticked paths ending in `.rs`, replacing `` `src/tui.rs` `` with `` `src/tui/` `` or referencing the historical file as `` `src/tui/` (formerly `src/tui.rs`) `` or removing the backticks prevents the probe from flagging a dead file path while preserving historical prose accuracy.
