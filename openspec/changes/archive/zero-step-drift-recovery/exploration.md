# Exploration: State Representation & Deterministic Drift Recovery

## 1. Technical Context & State of the Art

In long-running agent workflows, execution correctness depends on maintaining an accurate world model. Badhe et al. (*SKILL.state*, arXiv:2608.26263v2) demonstrated that conversational memory models suffer from quadratic prompt bloat ($O(t)$ prompt size, $O(T^2)$ tokens) and severe latency in detecting external environment drift. When an external actor alters the environment, standard agents hallucinate for 5–8 turns because stale textual assertions in conversation history dominate model attention over subtle contradiction signals in new observations.

In `ce-ai`, the CLI acts as an external orchestrator and governor for 10 native AI harnesses (Claude, Cursor, Codex, OpenCode, Antigravity, etc.). Because `ce-ai` does not own the raw token generation loop of third-party harnesses, the most effective architectural lever is **Turn-0 Ground-Truth State Injection**: providing an explicit, validated `RepoState` projection at turn resumption (`ce-ai workflow resume`), establishing the canonical state $\Sigma_0$ before the agent plans its next action.

## 2. Evaluated Architectural Options

### Option 1: Conversational ReAct History (Status Quo)
- **Mechanism:** The agent runs commands like `git status` or `git branch` when it suspects changes, appending stdout to conversational history.
- **Tradeoffs:**
  - ❌ Suffers from 5–8 turns lag when drift is unprompted.
  - ❌ Wastes context tokens on raw verbose terminal outputs.
  - ❌ Fails to check plugin manifest integrity or `AGENTS.md` adoption marker consistency.

### Option 2: Deep Synchronous Manifest & Repository Recalculation
- **Mechanism:** Recalculate SHA256 hashes of all files in the repository and managed plugin trees on every single invocation of `workflow resume`.
- **Tradeoffs:**
  - ❌ Can introduce 50–150ms of latency on large codebases, violating fast-path interactive responsiveness.
  - ❌ Redundant computation for files untouched in git working tree.

### Option 3: Hybrid Canonical State Projection with SHA256 Authority (Chosen)
- **Mechanism:**
  - Use `git rev-parse` and `git status --porcelain=v1` to extract git branch and modified working tree files in <5ms.
  - Inspect managed plugin directory against `InstallManifest` using cached manifest structures from `src/opencode/manifest.rs` and `src/state/diff.rs`.
  - Check project adoption marker in `AGENTS.md` against the recorded block SHA in `state.json`.
  - Re-hydrate `OpenSpecContextInfo` from `openspec/changes/`.
- **Tradeoffs:**
  - ✅ Delivers sub-15ms execution time.
  - ✅ Cryptographically exact: uses SHA256 as the canonical source of truth.
  - ✅ Provides zero-step world-model synchronization across both human and agent interfaces.

## 3. Cryptographic Determinism vs. Timestamp Heuristics (v1.23.0 Lesson)

In `ce-ai v1.23.0` (`registry.rs:247`), a subtle determinism bug occurred when feature inference relied on file modification times (`mtime`) across different filesystems and git worktrees, leading to non-reproducible resolution order when worktrees were cloned with uniform creation timestamps.

To prevent any regression of this class in `zero-step-drift-recovery`:
1. **SHA256 is the Exclusive Source of Truth:** A file is drifted if and only if `SHA256(disk_bytes) != SHA256(manifest_bytes)`.
2. **`mtime` is Strictly an Early-Exit Heuristic:** If a file's `mtime` matches the last recorded verification timestamp, the engine may skip re-hashing in ultra-fast modes; however, whenever `mtime` indicates potential modification or when `ce-ai doctor / sync` runs, SHA256 hashing is unconditionally executed. `mtime` is never stored or reported as a proof of integrity.

## 4. Addressing Existing Test Debt

An audit of `src/commands/workflow.rs` and `src/commands/sync.rs` revealed two untested internal structs that this feature builds upon:
- `OpenSpecContextInfo` (`src/commands/workflow.rs:211`): Probing logic for `openspec/changes/` currently lacks dedicated unit test fixtures.
- `TreeDrift` (`src/commands/sync.rs:586`): Calculates differences between desired and actual file trees.

The technical design must encapsulate these structures into testable, pure functions and include comprehensive unit tests as part of this feature delivery.
