# Proposal: Zero-Step Environment Drift Recovery via Live `RepoState` Sync

## 1. Problem Statement

Standard AI agent runtimes condition future tool execution on accumulated conversational history. When external actors modify the workspace outside the agent's turn—such as human git commits, branch checkouts, IDE file renames, or manual dependency upgrades—the agent's internal world model diverges from disk state. As empirically demonstrated by Badhe et al. in *SKILL.state: Scalable Long-Horizon Agent Skills* (arXiv:2608.26263v2, §5.4, Table 3), history-appending runtimes take 5 to 8 consecutive turns to recover from silent environment drift because obsolete facts in chat history overpower new observations.

In `ce-ai`, while `doctor.rs` and `sync.rs` compute cryptographic SHA256 manifest drift and verify project adoption markers, `ce-ai workflow resume` currently only probes `openspec/changes/` progress. It does not inspect:
1. **Git Working Tree State:** Active branch name, HEAD commit SHA, or dirty working tree modifications.
2. **Managed Plugin Manifest Integrity:** Drift against `InstallManifest` in `~/.config/opencode/compound-engineering`.
3. **Project Adoption Block Integrity:** Marker validity and SHA256 integrity of the adopted `AGENTS.md` block.

Consequently, when an agent wakes up or resumes an execution stage after context compaction or branch switching, it risks operating on stale assumptions for several turns before encountering a runtime failure.

## 2. In-Scope vs Out-of-Scope Boundaries

### In-Scope:
- **`RepoState` Data Model:** Define a structured, serializable `RepoState` capturing git branch, HEAD SHA, working tree dirtiness, manifest drift count, adoption block validity, and OpenSpec task progress.
- **Fast Live Probing Engine:** Implement sub-15ms live state probing in `src/commands/workflow.rs` (reusing SHA256 diffing primitives from `src/state/diff.rs` and `src/opencode/manifest.rs`).
- **Dual Integration Surface:**
  - `ce-ai workflow resume`: Format human-readable `== [Environment State & Drift Status] ==` block and emit `repo_state` in `--json` output.
  - `ce-ai status`: Surface current git branch and dirty working tree status.
- **Informative Non-Blocking Guidance:** Surface exact drift warnings and actionable remediation (`ce-ai sync`) without crashing the turn or blocking execution.
- **TDD Test Coverage:**
  - Unit tests for `OpenSpecContextInfo` probing (addressing existing coverage debt in `src/commands/workflow.rs:211`).
  - Unit tests for `TreeDrift` diffing calculation (addressing existing coverage debt in `src/commands/sync.rs:586`).
  - Integration tests for `ce-ai workflow resume` verifying 0-step state reflection across clean and drifted environments.

### Out-of-Scope:
- Modifying internal LLM inference loops of third-party agent harnesses.
- Auto-executing destructive git operations (e.g. automatic `git stash` or `git reset`).
- Making blocking network calls to GitHub during turn resumption (must remain strictly local and fast).

## 3. ISO/IEC 42001 & NIST AI RMF Risk Register

| Risk ID | Description | Severity | Mitigation |
| :--- | :--- | :--- | :--- |
| **R1** | Performance regression on turn resumption (>50ms) | Medium | Use shallow `git status --porcelain=v1` and single-pass file stats; benchmark against 15ms target. |
| **R2** | Non-deterministic state inference via file timestamps | High | Maintain SHA256 as the absolute source of truth. File modification times (`mtime`) may only serve as early-exit heuristics, never as ground-truth drift determinants. |
| **R3** | Crash when executed outside a git repository or without OpenSpec | High | Ensure all subsystems (`git`, `manifest`, `openspec`) return optional/graceful fallbacks (`None` / `is_git_clean: true`). |
| **R4** | Blocking user workflow during minor documentation drift | Medium | Keep drift warnings informative and non-blocking during `workflow resume`. |

## 4. Success Criteria

1. `ce-ai workflow resume` returns full `RepoState` in both formatted text and JSON (`--json`) under 15ms.
2. Simulated file changes and branch switches outside `ce-ai` are reflected in Turn 0 of `workflow resume` with 0 lag turns.
3. 100% test coverage added for `OpenSpecContextInfo` and `TreeDrift` calculation.
4. `cargo clippy --all-targets --all-features -- -D warnings` passes with 0 warnings.
5. All unit, integration, and E2E tests pass (`cargo test && make e2e`).
