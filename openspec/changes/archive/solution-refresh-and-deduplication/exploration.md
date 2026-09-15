# Exploration: Solution Library Clustering & Refresh Architecture

## 1. Technical Context & Investigation

`ce-ai` currently tracks 75 solution files in `docs/solutions/` across subdirectories:
- `architecture/`: 53 files
- `cli-issues/`: 1 file
- `git-delivery-issues/`: 1 file
- `packaging-issues/`: 1 file
- `test-issues/`: 1 file
- `workflow-issues/`: 18 files

Each solution file adheres to the normalized YAML frontmatter schema:
```yaml
---
title: "..."
category: "architecture"
date: "YYYY-MM-DD"
problem_type: "architecture"
tags:
  - tag1
  - tag2
components:
  - comp1
applies_when: "..."
---
```

### Semantic Redundancy Patterns Observed
1. **Harness Turn-0 Drift Delivery**: At least 9 distinct solutions document adapter turn-0 delivery for individual harnesses (`2026-09-02-agy-native-harness-adapter.md`, `2026-09-02-claude-code-native-harness-adapter.md`, `2026-09-02-copilot-cli-session-start-hook-and-additional-context.md`, etc.).
2. **Doctor Probes & Branch Protection**: 5 solutions document doctor checks and diagnostic probes.
3. **Workflow FSM & Stage Checkpoints**: 6 solutions document workflow states, mtime fallbacks, and task desync.
4. **Archive & Compaction**: 4 solutions document archive commands and milestone rollups.

## 2. Evaluated Options for Similarity & Clustering

### Option A: External Embedding API / Vector DB
- **Mechanism**: Use an external embedding provider (OpenAI, Gemini, Ollama) to embed markdown texts and perform cosine clustering.
- **Pros**: Captures subtle semantic rephrasings.
- **Cons**: Requires network access, API keys, external dependencies, or heavy local models. Fails completely offline or in isolated CI container gates (`make e2e`). Adds non-determinism.
- **Verdict**: **Rejected**. `ce-ai` core must remain completely self-contained, lightning fast (<15ms), zero-dependency, and deterministic.

### Option B: Deterministic Multi-Dimensional Jaccard & Attribute Overlap
- **Mechanism**: Compute pairwise similarity scores based on 4 weighted orthogonal dimensions:
  1. **Tag Overlap (Weight 0.35)**: Jaccard coefficient $J(T_a, T_b) = \frac{|T_a \cap T_b|}{|T_a \cup T_b|}$.
  2. **Component Overlap (Weight 0.25)**: Overlap of cited code components in frontmatter.
  3. **Category / Module Matching (Weight 0.15)**: Exact match or sibling match within `docs/solutions/`.
  4. **Title Token Overlap (Weight 0.25)**: Jaccard similarity of normalized title keywords (stopword-filtered, lowercase, alphanumeric).
- **Clustering Algorithm**: Greedy Leader-Follower or Graph Connected Components with a minimum similarity threshold (e.g. $\ge 0.40$) and minimum cluster size ($\ge 3$ files).
- **Pros**:
  - Deterministic and 100% reproducible.
  - Zero external dependencies; runs entirely with Rust stdlib.
  - Microsecond execution time (<5ms for 100 documents).
  - High explainability: outputs exact matching tags and keywords that formed the cluster.
- **Cons**: Requires tuned weights, but works exceptionally well for structured technical documents.
- **Verdict**: **Selected**.

### Option C: LLM In-Band Synthesis in the CLI
- **Mechanism**: The CLI directly rewrites and merges solution files using an LLM.
- **Pros**: Fully automated consolidation.
- **Cons**: High token cost, unpredictable changes to historical documentation, potential loss of critical subtle edge cases, violates the separation of concerns between `ce-ai` CLI (deterministic harness orchestrator) and Agent Skills (reasoning / synthesis).
- **Verdict**: **Rejected**. The CLI must identify clusters, compute metrics, and suggest scopes for the `/ce-compound-refresh` agent skill.

## 3. CLI Design & Subcommand Hierarchy

We evaluated whether to put this under `ce-ai workflow`, `ce-ai doctor`, or a dedicated `ce-ai doc` top-level command.

| Placement | Analysis |
|-----------|----------|
| `ce-ai workflow doc` | Clutters `ce-ai workflow` which is focused on 7-stage FSM and Turn-0 state. |
| `ce-ai doctor --cluster` | `doctor` is designed for fast health checks, not interactive multi-action reporting. |
| `ce-ai doc <subcommand>` | Clean, intuitive, aligns with `ce-ai spec`, `ce-ai models`, and `ce-ai skills`. Allows `cluster`, `refresh`, `lint`, and `stats`. |

**Verdict**: Adopt `ce-ai doc` as a dedicated top-level subcommand.

## 4. Integration with `/ce-compound-refresh`

The `/ce-compound-refresh` skill accepts an argument:
`[optional: scope hint — directory, filename, module, or keyword]`
When `ce-ai doc cluster` identifies a cluster, e.g.:
`Cluster: Harness Turn-0 Drift Delivery (9 documents, dominant tag: 'turn-0')`
The CLI outputs:
`Suggested action: run '/ce-compound-refresh turn-0' or '/ce-compound-refresh architecture/harness'`
This creates a seamless bridge between deterministic CLI diagnostics and agent-driven documentation compounding.
