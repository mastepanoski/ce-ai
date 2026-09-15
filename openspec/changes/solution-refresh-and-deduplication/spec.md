---
title: "Solution Library Refresh & Deduplication Specification"
domain: workflow
version: 1.0.0
last_updated: "2026-09-15"
---

# Specification: Solution Library Refresh & Deduplication Engine

## Overview
Defines requirements for the `ce-ai doc` CLI suite, semantic similarity clustering, and integration with the `/ce-compound-refresh` workflow.

---

## Capabilities & Requirements

### Requirement R1: Solution Inventory & Metadata Collection
WHEN the system inspects `docs/solutions/`  
THEN it MUST recursively collect all `.md` solution files  
AND parse their YAML frontmatter into structured metadata containing `title`, `category`, `problem_type`, `tags`, `components`, `applies_when`, and `date`  
AND tokenize the title into normalized lowercase keyword tokens excluding standard stopwords.  
WHEN `docs/solutions/` does not exist or contains 0 markdown files  
THEN the inventory MUST return an empty collection without error.

### Requirement R2: Multi-Dimensional Solution Similarity Scoring
WHEN computing similarity between two solution documents  
THEN the engine MUST calculate a composite similarity score between $0.0$ and $1.0$ based on:
1. Tag Jaccard similarity (weight: 0.35)
2. Component Jaccard similarity (weight: 0.25)
3. Title token Jaccard similarity (weight: 0.25)
4. Category exact match (weight: 0.15)  
WHEN two solutions share zero tags, components, title tokens, and categories  
THEN the computed similarity MUST be exactly `0.0`.  
WHEN two identical solutions are compared  
THEN the computed similarity MUST be exactly `1.0`.

### Requirement R3: Semantic Clustering Algorithm (`ce-ai doc cluster`)
WHEN `ce-ai doc cluster` is executed  
THEN the system MUST:
1. Form an adjacency graph connecting pairs of solutions whose similarity exceeds `--threshold` (default: 0.40).
2. Partition the graph into connected components.
3. Filter for clusters containing at least `--min-size` solutions (default: 3).
4. Compute the cluster's dominant tags and assign an explanatory name and suggested `/ce-compound-refresh` scope hint.  
WHEN `--json` is specified  
THEN the command MUST serialize a `ClusterReport` to stdout.  
WHEN no clusters meet the `--min-size` criterion  
THEN the command MUST exit with code 0 and report that 0 clusters were found.

### Requirement R4: Solution Scope Refresh Directives (`ce-ai doc refresh`)
WHEN `ce-ai doc refresh [--scope <name>]` is executed  
THEN the system MUST:
1. If `--scope` is provided, filter solutions whose category, title, tags, or path match the scope query.
2. If `--scope` is omitted, evaluate the highest-density cluster identified by the clustering algorithm.
3. Display the member documents, identifying the oldest vs newest solutions, and format an actionable `/ce-compound-refresh <scope>` invocation.  
WHEN `--dry-run` is passed  
THEN the command MUST preview the refresh recommendation without executing external actions.

### Requirement R5: Solution Library Linting (`ce-ai doc lint`)
WHEN `ce-ai doc lint` is executed  
THEN the system MUST execute the solution drift probe across `docs/solutions/**/*.md`:
1. Validate required YAML frontmatter fields (`title`, `category` or `module`, `problem_type`, `tags`, `applies_when`).
2. Verify that backticked repository source code paths (`src/**/*.rs`, `tests/**/*.rs`) exist in the workspace.  
WHEN `--strict` is specified and one or more findings are detected  
THEN the command MUST exit with code 6 (`CeError::Verification`).  
WHEN `--json` is specified  
THEN the findings MUST be serialized as JSON.

### Requirement R6: Solution Library Statistics (`ce-ai doc stats`)
WHEN `ce-ai doc stats` is executed  
THEN the system MUST output:
1. Total number of solution documents.
2. Distribution across categories (`architecture`, `workflow-issues`, etc.).
3. Top 10 most common tags and their frequency counts.
4. Total count of detected dense clusters.  
WHEN `--json` is specified  
THEN the command MUST output structured JSON conforming to `DocStatsReport`.

### Requirement R7: Doctor Health Check Integration
WHEN `ce-ai doctor` is executed  
THEN the system MUST probe `docs/solutions/` for dense clusters:
1. If 1 or more clusters with $\ge 3$ overlapping solutions are found, emit non-blocking `doctor-info: solution library: N consolidation cluster(s) detected — run 'ce-ai doc cluster' for details`.
2. This probe MUST NOT block the doctor exit code (non-fatal advisory).
