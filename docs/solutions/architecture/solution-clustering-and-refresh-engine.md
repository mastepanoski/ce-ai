---
title: "Solution Library Semantic Clustering and Refresh Engine"
category: "architecture"
problem_type: "architecture"
date: "2026-09-15"
applies_when: "managing large solution libraries with semantic duplication or planning doc consolidation"
tags:
  - doc
  - clustering
  - refresh
  - deduplication
  - jaccard
  - graph
components:
  - src/commands/doc.rs
  - src/commands/doctor.rs
  - src/commands/registry.rs
---

# Solution Library Semantic Clustering and Refresh Engine

## Problem Statement
As the `docs/solutions/` repository expanded past 75+ markdown files, semantic overlap and redundant guidance naturally accumulated across features developed in different quarters. While individual solutions were accurate, locating all related documents for a topic (e.g. SQLite locking, harness adapters, or doctor probes) required manual searches. Furthermore, human and AI agents had no automated diagnostic mechanism to detect topic density clusters or suggest consolidation scopes for the `/ce-compound-refresh` workflow.

## Architecture & Design Decisions

### 1. Zero-Dependency Multi-Dimensional Jaccard Similarity
Rather than pulling in external embedding models or native vector libraries which would bloat the CLI binary and introduce non-deterministic results, `ce-ai doc` computes a multi-dimensional Jaccard similarity score across four orthogonal facets:
- **Tags Affinity (0.35 weight)**: Jaccard similarity of normalized YAML frontmatter tags (`|A ∩ B| / |A ∪ B|`).
- **Component Affinity (0.25 weight)**: Jaccard similarity of declared source file paths (`components`).
- **Title Token Affinity (0.25 weight)**: Jaccard similarity of title tokens normalized with lowercase alphanumeric filtering and standard English stopword removal.
- **Category Affinity (0.15 weight)**: Binary match (1.0 if both files share the same category/module, 0.0 otherwise).

This composite score ranges from 0.0 to 1.0, executing across 75+ documents in under 15 milliseconds.

### 2. Connected Component Graph Clustering
To partition solutions into consolidation groups:
1. An undirected graph is constructed where vertices represent solution files and edges exist when `calculate_solution_similarity(u, v) >= threshold` (default `0.40`).
2. Breadth-First Search (BFS) traverses connected components.
3. Components with cardinality `>= min_size` (default `3`) are retained as active clusters.
4. For each cluster:
   - The top 3 dominant tags across member documents are resolved.
   - The primary dominant tag serves as the recommended scope argument for the agent skill: `/ce-compound-refresh <scope>`.
   - The average pairwise similarity is computed to quantify cluster coherence.

### 3. Command Suite Separation
The `ce-ai doc` subcommand suite provides focused, single-responsibility entry points:
- `ce-ai doc cluster [--min-size <N>] [--threshold <F>] [--json]`: Discovers and groups dense clusters.
- `ce-ai doc refresh [--scope <name>] [--dry-run]`: Surfaces candidate solutions for a specific scope or top cluster and provides copy-pasteable `/ce-compound-refresh` directives.
- `ce-ai doc lint [--strict] [--json]`: Verifies frontmatter completeness and validates that referenced source code paths exist.
- `ce-ai doc stats [--json]`: Reports inventory distributions, category counts, and top tag occurrences.

### 4. Non-Fatal Doctor Health Probe (`probe_solution_clusters`)
In `ce-ai doctor`, a non-blocking informational probe inspects `docs/solutions/`. If 1 or more dense clusters exist, doctor emits an informational notice advising the developer to run `ce-ai doc cluster`, maintaining zero exit-code disruptions while alerting developers to consolidation opportunities.

## Key Learnings
1. **Alphabetical Tie-Breaking on Frequencies**: When multiple tags share identical maximum frequencies in a cluster, deterministic alphabetical sorting ensures reproducible cluster naming across runs and operating systems.
2. **Deterministic Graph Connected Components**: Standard BFS connected component clustering avoids stochastic clustering variance (e.g. k-means initial seeds), ensuring identical results across CI runs and platforms.
3. **Synergy with Agent Skills**: The CLI handles fast deterministic clustering and inventory diagnostics, while deferring actual markdown prose consolidation and editing to the LLM agent skill (`/ce-compound-refresh`).
