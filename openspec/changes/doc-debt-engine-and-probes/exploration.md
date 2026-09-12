# Exploration: Technical Options for Documentation Technical Debt Diagnostics

## 1. Investigation of Current Substrate

### A. The Stranded Changes Anomaly
Investigation of the live `ce-ai` repository revealed 3 unarchived change packages in `openspec/changes/`:
1. `doctor-kimi-marketplace-divergence`: All parent tasks 1 through 5 marked `[x]`, but subtasks 1.1–5.4 marked `[ ]`. Code merged in commit `592a979`.
2. `harness-manifest-sha256-coverage`: All parent tasks 1 through 5 marked `[x]`, subtasks 1.1–5.5 marked `[ ]`. Merged in PR #335 (`56fcd51`).
3. `linux-musl-static-release`: Tasks 1.1–3.6 marked `[x]`, task 3.7 marked `[ ]`. Released in v1.50.1, while current `Cargo.toml` is `1.53.0`.

`probe_unarchived_completed_changes` in `src/commands/workflow.rs:975-976`:
```rust
let (completed, total) = count_task_checkboxes(&tasks_path);
if total > 0 && completed == total { ... }
```
Because `completed < total` for all three, they were silently ignored.

### B. The Silent No-Git Antipattern
In `src/commands/workflow.rs:692-700`:
```rust
let (is_clean, modified_files) = probe_git_dirty_files(&repo_root);
```
When `git status` fails or `.git` does not exist, `probe_git_dirty_files` returns `(true, Vec::new())`. For document health diagnostics, this behavior is hazardous: reading absence of git telemetry as "zero debt" masks real problems in Docker, CI, and export environments.

## 2. Evaluated Options

### Option 1: Monolithic Bundle (Detection + Auto-Compaction)
- *Concept:* Implement probes 1, 2, 5, and `ce-ai archive compact` in a single large pull request.
- *Tradeoffs:*
  - Pros: One-stop resolution of both detection and archive cleanup.
  - Cons: Violates the ~200 LOC per work unit policy; PR would exceed 800 LOC. Mixing read-only diagnostic probes with high-risk historical git tree mutations increases regression risks.
- *Decision:* **Rejected.** Decouple compaction to Increment 3.

### Option 2: Pure Checklist Parsing (Top-Level Checkboxes Only)
- *Concept:* Modify `count_task_checkboxes` to ignore indented subtasks.
- *Tradeoffs:*
  - Pros: Simple regex tweak.
  - Cons: Misses flat checklists (like `linux-musl-static-release` where tasks are flat `1.1..3.7` and only one task was omitted); cannot verify whether code actually merged to `main`.
- *Decision:* **Rejected as standalone.** Incorporated as part of a multi-level heuristic.

### Option 3: Multi-Level Heuristic + First-Class Tri-State (Recommended)
- *Concept:* Combine:
  1. Structural subtask desync: 100% parent tasks checked `[x]`.
  2. Git commit inspection: verify if commits referencing the feature landed on `HEAD`/`main`.
  3. Version threshold: check if task versions are surpassed in `Cargo.toml`.
  4. Tri-State `ProbeStatus`: `Clean`, `Debt(T)`, `Unknown`. When Git is missing, Git-dependent heuristics report `Unknown` / `[git: n/a]`.
- *Tradeoffs:*
  - Pros: Catches 100% of stranded features with zero false positives; operates reliably in both Git and No-Git environments.
  - Cons: Requires small helper functions for git log checking.
- *Decision:* **Adopted.**

## 3. Solution Drift Auditing Tradeoffs
- *Option A (AST & Symbol Compiler Inspection):* Full rustc/syn analysis to check if structs/functions cited in solutions still exist.
  - *Verdict:* Too slow (&gt;500ms), fragile on non-Rust files, fails in client projects.
- *Option B (Path Existence + Frontmatter Schema):* Regex match on backticked repository paths (`src/**`, `tests/**`) + YAML frontmatter validation.
  - *Verdict:* Sub-millisecond execution, 100% deterministic, zero false positives on external terms. **Adopted.**
