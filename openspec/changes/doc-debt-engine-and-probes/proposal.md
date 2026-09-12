# Proposal: Documentation Technical Debt Diagnostic Engine & Probes (Increments 1 & 2)

## Problem Statement
In Compound Engineering, documentation acts as the project's living contract and persistent memory layer. As code evolves rapidly across releases, documentation inevitably accumulates technical debt:
1. **Subtask Desync Stranding Completed Features:** `probe_unarchived_completed_changes` in `src/commands/workflow.rs` strictly checks `completed == total`. Features with 100% top-level tasks checked (`- [x] Task 1`) but open subtasks (`- [ ] 1.1`), or with code already merged to `main` while a post-merge release checkbox remains open (e.g. `doctor-kimi-marketplace-divergence`, `harness-manifest-sha256-coverage`, and `linux-musl-static-release`), are ignored by `doctor` and linger indefinitely in `openspec/changes/`.
2. **Silent OpenSpec Inactivity Rot:** Changes abandoned or stalled with 0% or partial task completion receive zero diagnostics from `doctor`, cluttering active change directories.
3. **Solution Library Drift in `docs/solutions/`:** Refactorings and file moves render past solution file paths obsolete. AI coding agents searching `docs/solutions/` ingest dead file pointers and hallucinate deleted APIs or superseded patterns.
4. **Substrate Concealment Antipattern:** Collapsing missing substrate (such as environments without `.git`) to `clean` or `0` creates false confidence by equating absence of diagnostic signal with absence of debt.
5. **Newbie Comprehension Gap:** Without an accessible, pedagogical explanation of *why* doc rot misleads AI agents and *how* automated probes protect development velocity, beginners struggle to understand the value of doc hygiene.

## In-Scope
1. **Umbrella Diagnostic Engine in `RepoState`**:
   - `DocDebtReport` struct aggregating tri-state probe statuses (`Clean`, `Debt`, `Unknown`).
   - Workspace override thresholds in `.ce-ai.json` via `State::load_with_workspace_overrides`.
   - Compact Turn-0 summary banner in `ce-ai workflow resume` and `workflow status`.
2. **Deterministic Diagnostic Probes**:
   - **Probe 1 (Git-Aware Desync)**: Multi-level heuristic detecting parent tasks `[x]` with subtasks `[ ]`, commits merged to `main`, or surpassed version tags.
   - **Probe 2 (Stale Inactivity Watchdog)**: Flags incomplete changes with no commits/activity exceeding configured days (default: 21d).
   - **Probe 5 (Solution Library Drift)**: Audits dead file paths in `docs/solutions/**/*.md` and validates YAML frontmatter schema.
3. **Actionable Doctor Integration**:
   - Non-blocking diagnostic warnings in `ce-ai doctor` with copy-pasteable CLI commands (`ce-ai archive <feature> --auto-mark` or `--status "..."`).
4. **First-Class No-Git Degradation**:
   - Explicit `ProbeStatus::Unknown` / `[git: n/a]` when Git is absent or indeterminate.
5. **Newbie Documentation & README Update**:
   - Pedagogical guide `docs/user-guide/doc-hygiene-and-debt-explained.md` in Diátaxis Explanation quadrant for Beginners.
   - Newbie-friendly `README.md` update strictly adhering to the `README.md <= 100` lines invariant.

## Out-of-Scope
- **Increment 3 (Compaction):** `ce-ai archive compact`, generational archive rollups, and milestone digests are decoupled into an independent PR to respect the ~200 LOC per work unit boundary.
- **Automated Silent Disk Mutations:** Probes strictly diagnose and report; they do not alter markdown files or move directories without user command invocation.
- **Living Spec Promotion (Idea 4):** Formal promotion of delta specs to `openspec/specs/` is deferred as an architectural methodology standard.
- **Automated Solution Clustering (Idea 6):** Semantic clustering of overlapping solutions is deferred until solution volume warrants it.

## Risk Evaluation & Mitigation
- **Risk (False Sense of Cleanliness without Git):** In CI environments or Docker containers running without `.git`, probes might silently report `0 debt`.
  - *Mitigation:* Implement strict `ProbeStatus` tri-state; missing Git explicitly reports `Unknown` / `[git: n/a]` and never collapses to `Clean`.
- **Risk (Performance Degradation on Turn-0):** Deep filesystem or Git history inspection could slow down `workflow resume`.
  - *Mitigation:* Bounded reads: `git log -n 1 -- <path>` only for change directories, lightweight regex file path scans for `docs/solutions/`, total budget capped under 15ms.
- **Risk (README Line Count Regression):** Adding documentation debt explanations could exceed the 100-line limit.
  - *Mitigation:* Use concise progressive disclosure: 2 lines in introduction, 1 row in Documentation Map table, verified with `wc -l README.md`.

## Success Criteria
1. The 3 real-world stranded changes in `ce-ai` are immediately detected by Probe 1 in `ce-ai doctor`.
2. Tests in a non-git directory verify probes report `Unknown` / `[git: n/a]` and do not panic or report false `Clean`.
3. `probe_repo_state` executes in under 15ms in benchmark tests.
4. `docs/user-guide/doc-hygiene-and-debt-explained.md` is authored and `README.md` stays <= 100 lines.
5. 100% green verification: `cargo clippy --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `cargo test`.
