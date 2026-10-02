---
title: "Remediating Upstream Compound Engineering Coupling Divergences"
category: "bugfixes"
module: "compat"
date: "2026-10-02"
problem_type: "bugfix"
component: "workflow"
severity: "medium"
tags:
  - upstream
  - compatibility
  - frontmatter
  - pi
  - dead-paths
  - docs_root
components:
  - commands::workflow
  - commands::doc
  - commands::gate
  - harness::pi
---

# Remediating Upstream Compound Engineering Coupling Divergences

## Problem & Context

Following direct feedback from the upstream Compound Engineering maintainer, seven concrete coupling divergences were identified where `ce-ai`'s previous implementation assumed internal structures or imposed contracts that conflicted with upstream Compound Engineering:

1. **Hardcoded Documentation Root**: `ce-ai` assumed all plans, solutions, and brainstorms were located under `docs/`, ignoring the upstream `.compound-engineering/config.yaml` `docs_root` relocation capability.
2. **Over-Strict Frontmatter Requirements**: `ce-ai` required `title`, `tags`, and `applies_when` on all solution files. In upstream's `schema.yaml`, `tags` and `applies_when` are optional, and `applies_when` is knowledge-track only, causing all upstream bug-track documents to fail `ce-ai doc lint`.
3. **Plural vs. Singular Component Parsing**: `ce-ai` only inspected `components`, whereas upstream uses `component` (singular) and `related_components`, yielding a 0 component similarity score for all upstream documents during clustering.
4. **Rust-Only Dead Path Validation**: `clean_code_path` in `probe_solution_drift` only validated paths ending in `.rs`, ignoring multi-language source files (`.ts`, `.py`, `.go`, `.java`, etc.).
5. **Legacy Brainstorm Discovery**: `infer_stage_from_repo` checked legacy `docs/brainstorms/` and `docs/ideation/`, missing upstream's modern pattern of authoring requirements-only plans (`*-requirements.md`) under `<docs_root>/plans/`.
6. **Pi Extension Entrypoint Collision**: `ce-ai` wrote its companion lifecycle extension to `.pi/extensions/compound-engineering.ts`, colliding with upstream's native Pi plugin entrypoint.
7. **Loose `ce-debug` Gate Exemption**: The gate check used a naive substring check for `"ce-debug"`, allowing unrelated task strings like `"fix bug in ce-debug-parser"` to bypass the Stage 2 gate.

## Key Changes & Solution

1. **Configurable Docs Root Resolution**:
   - Implemented `resolve_docs_root(repo_root: &Path) -> PathBuf` in `src/commands/workflow.rs`, reading `.compound-engineering/config.yaml` and `.compound-engineering/config.local.yaml` for `docs_root` with safe fallback to `docs/`.
   - Wired `resolve_docs_root` into `probe_solution_drift`, `infer_stage_from_repo`, and `src/commands/doc.rs`.

2. **Upstream Schema Conformance**:
   - Updated `check_solution_frontmatter` to accept `module` as an alias for `category`.
   - Made `applies_when` optional for bug-track solutions (`problem_type: bugfix | bug`), eliminating false-positive lint failures on upstream bug documents.
   - Made `tags` and `title` optional.

3. **Multi-Source Component Metadata**:
   - Updated `parse_solution_file` in `src/commands/doc.rs` to parse singular `component: <name>`, `related_components: [...]`, and plural `components: [...]`, unifying them into a complete metadata array for clustering and analysis.

4. **Multi-Language Code Path Extraction**:
   - Expanded `clean_code_path` to recognize standard source extensions across languages: `.rs`, `.ts`, `.tsx`, `.js`, `.jsx`, `.py`, `.go`, `.rb`, `.c`, `.cpp`, `.cc`, `.h`, `.hpp`, `.java`, `.kt`, `.swift`, `.sh`, `.bash`, `.zsh`, `.yaml`, `.yml`, `.json`, and `.toml`.

5. **Modern Requirements Plan Discovery**:
   - Enhanced Stage 1 ideation probing to scan `<docs_root>/plans/` for `*-requirements.md` and `*-requirements.html` files, accurately identifying modern CE requirements plans before OpenSpec promotion.

6. **Pi Extension Namespacing & Migration**:
   - Renamed Pi companion extension to `.pi/extensions/ce-ai-companion.ts` (`PI_EXTENSION_FILENAME`).
   - Added automatic migration and cleanup in `ensure_session_start_hook` and `remove_session_start_hook` for legacy `compound-engineering.ts` files containing managed `ce-ai` markers, completely preventing collisions with upstream CE.

7. **Exact Task Prefix Gate Exemption**:
   - Hardened `ce-debug` exemption matching in `src/commands/gate.rs` to require exact string equality (`clean == "ce-debug"`) or structured prefixes (`ce-debug:`, `ce-debug `, `ce-debug/`).

## Verification & Impact

- 615 library unit tests and 218 CLI integration tests pass (833 tests total, 0 failures).
- `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings` pass cleanly.
- `cargo run -- doc lint --strict` passes with 0 findings across all 57+ solution files.
- `python3 scripts/validate-concepts.py CONCEPTS.md` verifies monotonic accretion (58 entries, +1 added, 0 scrubbed).
- `python3 scripts/validate-tasks-tail.py` confirms tasks tail compliance.
