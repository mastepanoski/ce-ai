# Proposal: Automated Solution Refresh & Deduplication Engine (`ce-ai doc`)

## 1. Problem Statement

In Compound Engineering, `docs/solutions/` functions as the persistent knowledge layer that captures architectural discoveries, bug post-mortems, and recurring engineering patterns.

As `ce-ai` has matured, the solution library has grown to over 75 files across multiple categories (`architecture/`, `cli-issues/`, `test-issues/`, `workflow-issues/`, etc.). While each solution captured valuable context at the time it was authored, this uncontrolled accumulation creates four major operational problems:

1. **Fragmentation of Subsystem Knowledge**: Highly related topics are scattered across numerous separate files (e.g., 9 separate solutions covering harness turn-0 drift delivery and native adapters, 6 solutions on test isolation, 4 on doctor diagnostics). Agents searching for guidance ingest dozens of fragmented fragments instead of a unified, high-leverage architectural reference.
2. **Subtle Divergence & Obsolete Workarounds**: Older solutions authored months ago may propose temporary workarounds or cite configurations that have since been superseded by core architectural changes (e.g. early workarounds for harness registration superseded by the unified registry engine). When AI agents retrieve both, they risk hallucinating outdated solutions or conflicting instructions.
3. **Lack of Automated Cluster Discovery**: Although the `/ce-compound-refresh` skill exists to review, update, consolidate, or replace solution documents, there is no CLI-native discovery mechanism in `ce-ai` that automatically analyzes the solution repository, identifies overlapping document clusters, and suggests targeted refresh scopes.
4. **Documentation Hygiene Disconnect**: While `ce-ai doctor` already checks for dead file paths in solutions, it has no awareness of cluster sprawl or semantic redundancy.

## 2. In-Scope & Out-of-Scope Boundaries

### In-Scope
- **CLI Subcommand Suite (`ce-ai doc`)**:
  - `ce-ai doc cluster [--min-size <N>] [--threshold <float>] [--json]`: Analyzes solution files in `docs/solutions/`, calculates similarity metrics (tag overlap, module matching, component citations, title token Jaccard similarity), groups documents into thematic clusters, and surfaces consolidation opportunities.
  - `ce-ai doc refresh [--scope <name>] [--dry-run]`: Generates targeted consolidation directives for `/ce-compound-refresh`, identifying canonical master docs versus candidates for consolidation or archiving.
  - `ce-ai doc lint [--strict] [--json]`: Standalone CLI diagnostic for solution health (frontmatter validity, dead path citations), exposing `probe_solution_drift`.
  - `ce-ai doc stats [--json]`: High-level inventory of the solution library (total docs, category counts, top tags, cluster density).
- **Doctor Diagnostic Integration**:
  - Non-blocking advisory probe in `ce-ai doctor` (`probe_solution_clusters`) reporting when 3 or more dense overlapping clusters are detected.
- **TUI & Workflow Integration**:
  - Expose cluster recommendations in `ce-ai workflow status` and Turn-0 repo state when applicable.
- **Comprehensive Test Suite**:
  - Unit tests for the clustering and similarity algorithms.
  - Integration CLI tests for `ce-ai doc` subcommands.

### Out-of-Scope
- **Automated Text Rewriting / Merging**: Merging markdown prose and synthesizing engineering principles requires LLM reasoning and editorial judgment. The CLI provides deterministic clustering, discovery, and scope hints, while delegating actual prose consolidation to the agent skill `/ce-compound-refresh`.
- **Replacing `docs/solutions/` Structure**: The directory structure and YAML frontmatter standard remain completely unchanged.

## 3. Risk Evaluation & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| False-positive clustering of loosely related solutions | Medium | Use multi-dimensional similarity scoring (tags, modules, components, title tokens) with configurable thresholds (`--threshold`, default 0.40) and minimum cluster size (`--min-size`, default 3). |
| Performance degradation on large repositories | Low | Solution files are lightweight markdown text (<10KB each). Frontmatter parsing and tokenization across 100-500 files executes in <15ms in native Rust. |
| Inadvertent file deletion | High | `ce-ai doc` is strictly read-only and analytical; it never deletes or modifies solution markdown files directly. Mutations are deferred to `/ce-compound-refresh` or explicit git workflows. |

## 4. Success Criteria

1. `ce-ai doc cluster` correctly identifies existing dense topic clusters in `docs/solutions/` (e.g. harness turn-0 adapters, test isolation, doctor probes).
2. `ce-ai doc stats` reports accurate category, tag, and cluster density metrics.
3. `ce-ai doc lint` provides a clean CLI entry point for solution frontmatter and path integrity checks.
4. `ce-ai doctor` emits non-blocking advisories when dense consolidation clusters exist.
5. 100% unit and CLI integration test coverage with 0 clippy warnings and green CI matrix.
