# Tasks: Upstream Coupling Remediation & CE Contract Alignment

- [x] **Unit 1 (~120 LOC):** Dynamic `docs_root` Discovery & Config Parser
  - Implement `resolve_docs_root(repo_root: &Path) -> PathBuf` checking `.compound-engineering/config.yaml`.
  - Update `src/commands/workflow.rs` and `src/commands/doc.rs` to use `resolve_docs_root`.
  - Add unit tests verifying fallback to `docs/` and custom directory parsing.
  - *Verification*: `cargo test test_resolve_docs_root`.

- [x] **Unit 2 (~140 LOC):** Solution Frontmatter Upstream Conformance & Component Parsing
  - Update `check_solution_frontmatter` in `src/commands/workflow.rs` to validate `module`, `date`, `problem_type`, `component`, and `severity`.
  - Make `applies_when` conditional (exempting bugfix docs) and `tags` optional.
  - Update `parse_solution_file` in `src/commands/doc.rs` to parse singular `component` and `related_components`.
  - Add unit tests for upstream frontmatter conformance and bugfix exemption.
  - *Verification*: `cargo test test_solution_frontmatter_upstream_conformance`.

- [x] **Unit 3 (~110 LOC):** Modern Brainstorms, Multi-Language Dead Paths, Pi Extension & Gate Precision
  - In `src/commands/workflow.rs`, update brainstorm detection to recognize `*-requirements.md` under `<docs_root>/plans/`.
  - In `clean_code_path`, accept multi-language code extensions (`.ts`, `.py`, `.go`, etc.).
  - In `src/harness/pi.rs`, update `PI_EXTENSION_FILENAME` to `ce-ai-companion.ts` with backward-compatible cleanup.
  - In `src/commands/gate.rs`, harden `ce-debug` exemption to structured prefixes.
  - Add unit tests for dead-path languages, Pi extension renaming, and gate prefix matching.
  - *Verification*: `cargo test test_gate_ce_debug_structured_prefix`.

- [x] **Unit 4: Code Simplification & Review (`ce-simplify-code`)**
  - Simplify newly added logic for clarity, performance, and cross-platform path handling.
  - Verify zero clippy warnings.

- [x] **Unit 5: Structured Code Review & Review Receipt**
  - Verify all 7 maintainer coupling issues are remediated.
  - Record workflow review receipt: `cargo run -- workflow review-receipt`.

- [x] **Unit 6: Knowledge Compounding & Vocabulary Capture (`ce-compound`)**
  - Create solution document in `docs/solutions/bugfixes/upstream-coupling-remediation.md`.
  - Verify monotonic accretion of `CONCEPTS.md`.
  - *Verification*: `cargo run -- doc lint --strict`.

- [x] **Unit 7: Verification, SemVer bump, and PR Creation (`ce-commit-push-pr`)**
  - Bump PATCH version to `1.75.1` in `Cargo.toml` and record changes in `CHANGELOG.md`.
  - Run full test suite: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
  - Push branch and open PR with upfront empirical evidence.
