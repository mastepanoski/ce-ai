# Tasks: Solution Library Drift Repair & Frontmatter Schema Normalization

Work-unit changed-line estimates total: ~380 LOC (~200 LOC/work-unit policy applied; normalization touches frontmatter headers and path references across solution files).

- [x] **Unit 1: Develop Deterministic Solution Normalization Script** (~120 LOC)
  - [x] 1.1 Implement Python normalization script in scratch (`scratch/normalize_solutions.py`) capable of parsing and re-serializing YAML frontmatter blocks.
  - [x] 1.2 Implement heuristic extraction for missing `title` from markdown headings (`# ...`).
  - [x] 1.3 Implement contextual derivation for missing `applies_when` based on document summary and tags.
  - [x] 1.4 Implement problem_type and category derivation based on directory hierarchy.

- [x] **Unit 2: Remediate Dead Source Path References** (~80 LOC)
  - [x] 2.1 Update `src/tui.rs` references across the 6 affected solution files to point to `src/tui/app.rs` or `src/tui/`.
  - [x] 2.2 Update `src/harness/generic_json.rs` references in `grok-native-harness-adapter.md` and `kimi-adapter-audit-refinements.md` to point to `src/harness/mod.rs`.
  - [x] 2.3 Update `tests/mod_tests.rs` reference in `extract-inline-unit-tests-2026-08-28.md` to point to `tests/cli.rs`.

- [x] **Unit 3: Execute Frontmatter Normalization Across All Solution Files** (~140 LOC)
  - [x] 3.1 Execute `normalize_solutions.py` across all 74 files in `docs/solutions/`.
  - [x] 3.2 Verify every solution file has `title`, `category` (or `module`), `problem_type`, `tags`, and `applies_when`.
  - [x] 3.3 Verify no body prose or code examples were corrupted or altered.

- [x] **Unit 4: Verification & Quality Gates** (~40 LOC)
  - [x] 4.1 Run `cargo run -- doctor` and verify 0 solution warnings and clean health output.
  - [x] 4.2 Run `cargo run -- workflow status` and verify clean doc debt report.
  - [x] 4.3 Run `cargo fmt --check`.
  - [x] 4.4 Run `cargo clippy --all-targets --all-features -- -D warnings`.
  - [x] 4.5 Run `cargo test` (623 tests).
