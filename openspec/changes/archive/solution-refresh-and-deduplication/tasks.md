# Tasks: Solution Library Refresh & Deduplication Engine (`ce-ai doc`)

## Work Breakdown & LOC Forecast (~910 LOC total)

- **Unit 1**: Data Structures, Frontmatter Parser & Solution Inventory (~170 LOC)
- **Unit 2**: Multi-Dimensional Similarity Scoring & Deterministic Clustering (~190 LOC)
- **Unit 3**: CLI Subcommand Handlers (`cluster`, `refresh`, `lint`, `stats`) (~210 LOC)
- **Unit 4**: Command Registration & Doctor Health Probe Integration (~120 LOC)
- **Unit 5**: Comprehensive Unit and CLI Integration Tests (~220 LOC)

---

## Task Checklist

### Unit 1: Data Models & Inventory Engine (~170 LOC)
- [x] 1.1 Define `DocArgs` and `DocCommand` enum with subcommands `Cluster`, `Refresh`, `Lint`, `Stats` in `src/commands/doc.rs`.
- [x] 1.2 Implement `SolutionMetadata`, `SolutionClusterMember`, `SolutionCluster`, `ClusterReport`, and `DocStatsReport` structs.
- [x] 1.3 Implement `parse_solution_file` and `collect_solutions_inventory(repo_root: &Path) -> Vec<SolutionMetadata>`.
- [x] 1.4 Implement title tokenization with standard English stopword filtering.

### Unit 2: Similarity Scoring & Clustering Algorithm (~190 LOC)
- [x] 2.1 Implement `jaccard_similarity<T: Eq + Hash>(a: &HashSet<T>, b: &HashSet<T>) -> f32`.
- [x] 2.2 Implement `calculate_solution_similarity(a: &SolutionMetadata, b: &SolutionMetadata) -> f32` with weights (0.35 tag, 0.25 comp, 0.25 title, 0.15 category).
- [x] 2.3 Implement graph adjacency and connected component partitioning in `cluster_solutions(solutions: &[SolutionMetadata], threshold: f32, min_size: usize) -> Vec<SolutionCluster>`.
- [x] 2.4 Implement dominant tag resolution and `/ce-compound-refresh` scope hint generator.

### Unit 3: CLI Subcommand Handlers (~210 LOC)
- [x] 3.1 Implement `run_doc_cluster(repo_root: &Path, min_size: usize, threshold: f32, json: bool) -> Result<(), CeError>`.
- [x] 3.2 Implement `run_doc_refresh(repo_root: &Path, scope: Option<&str>, dry_run: bool) -> Result<(), CeError>`.
- [x] 3.3 Implement `run_doc_lint(repo_root: &Path, strict: bool, json: bool) -> Result<(), CeError>` wrapping `probe_solution_drift`.
- [x] 3.4 Implement `run_doc_stats(repo_root: &Path, json: bool) -> Result<(), CeError>`.

### Unit 4: Command Registration & Doctor Integration (~120 LOC)
- [x] 4.1 Register `pub mod doc;` in `src/commands/mod.rs`.
- [x] 4.2 Add `Doc(DocArgs)` variant to `Commands` in `src/main.rs` and dispatch in `src/commands/registry.rs`.
- [x] 4.3 Implement `probe_solution_clusters(repo_root: &Path) -> Vec<String>` in `src/commands/doc.rs`.
- [x] 4.4 Wire `probe_solution_clusters` as a non-blocking diagnostic into `src/commands/doctor.rs`.

### Unit 5: Verification & Testing (~220 LOC)
- [x] 5.1 Add unit tests in `src/commands/tests/doc.rs` for similarity calculation, clustering, and tokenization.
- [x] 5.2 Add CLI integration tests in `tests/cli.rs` testing `ce-ai doc stats`, `ce-ai doc cluster`, and `ce-ai doc lint`.
- [x] 5.3 Verify formatting (`cargo fmt --check`) and clippy (`cargo clippy --all-targets --all-features -- -D warnings`).
- [x] 5.4 Execute full test suite (`cargo test`) and containerized E2E gate (`make e2e`).
