# Technical Design: Solution Library Clustering & Refresh Engine

## 1. System Architecture

The solution refresh and deduplication subsystem is implemented in `src/commands/doc.rs` with integration into CLI dispatch (`src/main.rs`, `src/commands/mod.rs`, `src/commands/registry.rs`) and health diagnostics (`src/commands/doctor.rs`).

```
┌─────────────────────────────────────────────────────────────┐
│                       CLI Entry Point                       │
│                     ce-ai doc <subcommand>                  │
└──────────────────────────────┬──────────────────────────────┘
                               │
       ┌───────────────────────┼───────────────────────┐
       ▼                       ▼                       ▼
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│ doc cluster │         │ doc refresh │         │  doc stats  │
└──────┬──────┘         └──────┬──────┘         └──────┬──────┘
       │                       │                       │
       └───────────────────────┼───────────────────────┘
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                 Solution Analysis Engine                    │
│  - Frontmatter parser & token extractor                     │
│  - Multi-dimensional similarity scorer (Tag, Comp, Title)   │
│  - Graph-based deterministic clustering                     │
│  - Refresh scope generator (/ce-compound-refresh bridge)   │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│              Repository Storage: docs/solutions/            │
│  75+ markdown files with YAML frontmatter across categories │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Data Structures & Schema (`src/commands/doc.rs`)

```rust
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone)]
pub struct DocArgs {
    #[command(subcommand)]
    pub command: DocCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum DocCommand {
    /// Analyze solution files and group them into semantic consolidation clusters.
    Cluster {
        /// Minimum number of solutions required to form a cluster.
        #[arg(long, default_value_t = 3)]
        min_size: usize,
        /// Minimum similarity coefficient threshold (0.0 to 1.0).
        #[arg(long, default_value_t = 0.40)]
        threshold: f32,
        /// Output results as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Audit a cluster or specific scope for refresh and consolidation.
    Refresh {
        /// Scope hint (category, module, tag, or cluster name).
        scope: Option<String>,
        /// Dry run preview without suggesting modifications.
        #[arg(long)]
        dry_run: bool,
    },
    /// Audit solution files for missing frontmatter and dead path references.
    Lint {
        /// Fail with non-zero exit code if warnings are found.
        #[arg(long)]
        strict: bool,
        /// Output report as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Display inventory statistics for the solution library.
    Stats {
        /// Output statistics as JSON.
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolutionMetadata {
    pub file_path: PathBuf,
    pub rel_path: String,
    pub title: String,
    pub category: String,
    pub problem_type: String,
    pub tags: Vec<String>,
    pub components: Vec<String>,
    pub applies_when: String,
    pub date: String,
    #[serde(skip)]
    pub title_tokens: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolutionClusterMember {
    pub rel_path: String,
    pub title: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SolutionCluster {
    pub id: String,
    pub name: String,
    pub dominant_tags: Vec<String>,
    pub suggested_refresh_scope: String,
    pub members: Vec<SolutionClusterMember>,
    pub average_similarity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClusterReport {
    pub total_solutions: usize,
    pub clusters_count: usize,
    pub clustered_solutions_count: usize,
    pub clusters: Vec<SolutionCluster>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocStatsReport {
    pub total_solutions: usize,
    pub category_distribution: BTreeMap<String, usize>,
    pub top_tags: Vec<(String, usize)>,
    pub clusters_count: usize,
}
```

---

## 3. Similarity Scoring & Clustering Algorithm

### Multi-Dimensional Scoring Function
For any two solutions $A$ and $B$:
$$S(A, B) = w_{\text{tag}} \cdot J(\text{Tags}_A, \text{Tags}_B) + w_{\text{comp}} \cdot J(\text{Comp}_A, \text{Comp}_B) + w_{\text{title}} \cdot J(\text{Tokens}_A, \text{Tokens}_B) + w_{\text{cat}} \cdot \delta(\text{Cat}_A, \text{Cat}_B)$$

Where:
- Weights: $w_{\text{tag}} = 0.35$, $w_{\text{comp}} = 0.25$, $w_{\text{title}} = 0.25$, $w_{\text{cat}} = 0.15$.
- $J(X, Y) = \frac{|X \cap Y|}{|X \cup Y|}$ is the Jaccard similarity coefficient. If both sets are empty, $J = 0.0$.
- $\delta(\text{Cat}_A, \text{Cat}_B) = 1.0$ if categories match, else $0.0$.
- Title tokenization: Lowercase, punctuation stripped, common English stopwords removed (`a`, `the`, `and`, `of`, `for`, `in`, `to`, `with`, `on`, `at`, `by`).

### Deterministic Greedy Clustering
1. Construct an undirected adjacency graph $G = (V, E)$ where vertices $V$ are solutions, and edges $E$ connect solutions with $S(A, B) \ge \text{threshold}$.
2. Compute connected components or greedy clique partitions where each component contains $\ge \text{min\_size}$ nodes.
3. For each cluster:
   - Rank tags by frequency to determine `dominant_tags`.
   - Select the most representative tag/component as the `suggested_refresh_scope`.
   - Name the cluster using the dominant topic (e.g. `harness-turn-0-adapters`).

---

## 4. Subcommand Interfaces & Outputs

### `ce-ai doc cluster`
Outputs formatted human-readable ASCII tables:
```
CLUSTER: harness-adapters (9 solutions, avg similarity: 0.62)
  Dominant Tags: [harness, turn-0, adapter]
  Suggested Scope: /ce-compound-refresh turn-0
  Members:
    • architecture/2026-09-02-agy-native-harness-adapter.md
    • architecture/2026-09-02-claude-code-native-harness-adapter.md
    • architecture/2026-09-02-codex-native-harness-adapter.md
    ...
```

### `ce-ai doc refresh --scope <name>`
Prints audit summary for the matched cluster or directory and displays exact `/ce-compound-refresh` command invocation.

### `ce-ai doc stats`
Displays total solution count, breakdown by category, top 10 most frequent tags, and cluster density.

### `ce-ai doctor` Integration
Adds `probe_solution_clusters`:
```
doctor-info: solution library: 3 dense topic cluster(s) detected (run 'ce-ai doc cluster' for consolidation suggestions)
```
Non-blocking informational advisory.
