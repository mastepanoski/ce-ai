# Exploration: Anti-Clobber Guard & Monotonic Accretion for CONCEPTS.md

## 1. Problem Investigation & Root Causes

Analysis of Issue #424 revealed a systemic blind spot in how vocabulary capture is handled in Compound Engineering workflows:

1. **Agent Context Isolation**:
   When an agent runs Phase 2.4 ("Vocabulary Capture"), it extracts new terms in its short-term context. If it calls `Write` (`write_to_file`) on `CONCEPTS.md` without reading the full existing file into context, the pre-existing 10, 20, or 50 glossary entries are instantly deleted, leaving only the 1 or 2 new terms.

2. **Absence of Tool-Level Guard**:
   Existing validation tools (`validate-frontmatter.py`, `validate-doc-claims.py`) explicitly filter on `docs/solutions/**/*.md`. No tool or hook inspects changes to `CONCEPTS.md`.

3. **Hallucinated Terminal Reporting**:
   The output line `CONCEPTS.md: updated — N added` is generated purely by LLM declaration without measuring the filesystem or git diff. If an agent deleted 20 entries and added 1, it still reports "1 added".

## 2. Evaluated Architectural Options

### Option A: Static Prompt / Instructions Only (`SKILL.md` update)
- **Concept**: Add stronger wording to `SKILL.md` demanding agents read `CONCEPTS.md` and use `Edit` instead of `Write`.
- **Pros**: Zero runtime code changes.
- **Cons**: Prompt drift, model non-compliance, and stochastic failures still occur. Without a mechanical validator, regressions are silent and unfixable.
- **Verdict**: **Rejected as standalone**. Necessary as a first defense layer, but insufficient on its own.

### Option B: Mechanical Validator Script Only (`validate-concepts.py`)
- **Concept**: Implement a standalone Python script similar to `validate-frontmatter.py` that checks `CONCEPTS.md` against git `HEAD`.
- **Pros**: Easy to run in scripts or skills.
- **Cons**: Does not integrate into `ce-ai doc lint`, `ce-ai doctor`, or CI pipelines automatically.
- **Verdict**: **Adopted as component**. Provides portable, stdlib-based validation for all harnesses.

### Option C: Native Rust Guard in `ce-ai` CLI + Portable Script + Instruction Hardening (Defense-in-Depth)
- **Concept**:
  1. Instruction Layer: Update `SKILL.md` (mandate `Read`, prefer surgical `Edit`, forbid blind `Write`).
  2. Portable Script: `scripts/validate-concepts.py` for direct harness/hook execution.
  3. Native Engine: `probe_concepts_drift` in `src/commands/workflow.rs`, integrated into `ce-ai doc lint`, `ce-ai doctor`, and `DocDebtReport`.
  4. CI Verification: Strict gate enforcement in `cargo test` and `make e2e`.
- **Pros**: Comprehensive multi-layer protection. CLI gates block corrupted commits; doctor warns developers; skills guide models correctly.
- **Cons**: Requires modest additions to both Rust CLI and script assets (~250 LOC total).
- **Verdict**: **Selected**. Provides absolute defense-in-depth across all supported harnesses and CI.

## 3. Trade-off Analysis & Boundary Decisions
- **LOC Budget**: Target implementation under 300 LOC (well below the 400 LOC review threshold).
- **Git Head Resolution**: When running outside git or on a brand new untracked file, the validator gracefully audits format and entry counts without panicking or failing falsely.
- **Scrubbing Support**: Deliberate removal of obsolete concepts is supported via inline HTML comments (e.g. `<!-- scrub: <Term> -->`) or `--allow-shrink` CLI flag.
