---
module: workflow
tags: [workflow, repo-state, drift-recovery, skill-state, arxiv-2608-26263v2, sha256, openspec]
problem_type: architecture
---

# Zero-Step Environment Drift Recovery via Live `RepoState` Sync

## Problem
In long-horizon autonomous AI agent sessions, human developers or external tools frequently alter workspace state (switching git branches, editing files, modifying plugin manifests) between turns. History-accumulating agent runtimes (ReAct) suffer from 5–8 turns of observation lag before detecting silent drift because stale conversational history dominates attention (arXiv:2608.26263v2). In `ce-ai`, prior to `v1.30.0`, `ce-ai workflow resume` only checked OpenSpec files without probing the git working tree, managed plugin SHA256 integrity, or project adoption marker status.

## Solution
Implemented Turn-0 canonical environment synchronization via live `RepoState` probing:

1. **`RepoState` Data Model (`src/commands/workflow.rs`):**
   - Captures `git_branch`, `head_sha`, `is_git_clean`, `modified_files`, `manifest_drift_count`, `adoption_status`, and `openspec_context`.
   - Serialized in human-readable plain text and structured `--json` payload on `ce-ai workflow resume`.

2. **Sub-15ms Live Probing Engine (`probe_repo_state`):**
   - Runs shallow `git rev-parse` and `git status --porcelain=v1` to discover active branch and uncommitted modifications.
   - Computes plugin manifest drift against `InstallManifest` using `crate::state::diff::diff`.
   - Delegates adoption block classification directly to `check_adoption_block_status()` in `src/commands/init_prj.rs` (preserving the Single Source of Truth).

3. **Deterministic Integrity Rule:**
   - Cryptographic SHA256 hashing is the sole source of truth for drift determination. File modification timestamps (`mtime`) serve only as fast-path change triggers, preventing timestamp determinism bugs.

## Key Learnings
1. **Turn-0 Ground-Truth Injection Eliminates Agent Lag:** Providing structured disk truth at the exact moment of context resume prevents agents from generating multi-turn hallucinated plans on top of stale working tree assumptions.
2. **SSOT Diagnostic Reuse:** Reusing `check_adoption_block_status()` across `doctor`, `status`, and `workflow resume` ensures all subsystems report identical adoption block diagnostics without diverging.
