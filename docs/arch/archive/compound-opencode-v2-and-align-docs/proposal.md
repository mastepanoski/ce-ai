# Proposal: Compound Knowledge Capture & Documentation Style Alignment

## Problem Statement
Following the v1.74.0 release (OpenCode V2 dual plugin loader), several critical Compound Engineering lifecycle steps were omitted before shipping:
1. Stage 6 (`ce-compound`) was skipped: no structured solution was written to `docs/solutions/bugfixes/`, and `CONCEPTS.md` was not updated with the newly established terminology.
2. `README.md` and related documentation drifted from the mandatory documentation style contract in `docs/references/docs-styling.md`:
   - `README.md` is 101 lines (exceeds the `<= 100` lines limit).
   - `README.md` violates the section order by placing explanations (`Why CE-AI?`, `How the pieces fit`, `The 7-stage workflow`) before the Quick Path.
   - `docs/user-guide/project-adoption-guide.md` violates Diátaxis by declaring a blended intent (`How-to & Explanation`).
3. No formal code simplification pass (`ce-simplify-code`) or review receipt (`ce-ai workflow review-receipt`) was recorded.

## In-Scope
- Author `docs/solutions/bugfixes/opencode-v2-dual-loader.md` with schema-compliant YAML frontmatter.
- Monotonically add new terms (*OpenCode Dual Plugin Loader*, *Native Command Materialization*, *OpenCode Config Key Agnosticism*) to `CONCEPTS.md`.
- Refactor `README.md` to conform to `docs/references/docs-styling.md` (≤ 100 lines, Quick Path directly after Title/What-Why, doc map with audience labels).
- Ensure single Diátaxis intent in `docs/user-guide/project-adoption-guide.md` (`How-to`).
- Add markdown links in `docs/references/docs-styling.md` for reference and explanation examples.
- Audit `src/opencode/` with `ce-simplify-code` principles.
- Record review receipt and verify zero regressions with `cargo test`, `make e2e`, and `cargo run -- doc lint --strict`.

## Out-of-Scope
- Breaking changes to CLI commands or configuration file formats.
- Rewriting unrelated guides in `docs/user-guide/`.

## Success Criteria
- `docs/solutions/bugfixes/opencode-v2-dual-loader.md` exists and passes `ce-ai doc lint --strict`.
- `CONCEPTS.md` accreted monotonically (verified by `ce-ai doc lint --strict`).
- `README.md` is strictly ≤ 100 lines and matches the Section 2 structure of `docs-styling.md`.
- All tests pass (`cargo test`, `make e2e`).
- `ce-ai workflow status` reflects clean state without unrecorded review receipt.
