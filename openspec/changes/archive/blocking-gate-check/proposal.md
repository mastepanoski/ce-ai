# Proposal: Blocking Gate Check & Validation Receipt for Unvetted Writes (Issue #334)

## 1. Problem Statement

In Spec-Driven Development and Compound Engineering, production code under `src/**` should only be written during Stage 4 (`ce-work`) when formal specifications (`proposal.md`, `spec.md`, `tasks.md`) have been approved. In Issue #333 (v1.49.0), an observe-only spike (`ce-ai gate check`) was deployed to measure empirical agent behavior without disrupting developers. Both blocking dependencies for Issue #334 (Issue #333 and Issue #337) are now closed in `main`.

However, without active mechanical enforcement, agents can still skip planning and jump straight into editing production code, creating untracked architectural drift. Furthermore, without a structured validation receipt, `ce-ai doctor` and `ce-ai status` lack visibility into whether code changes complied with formal contracts or bypassed them.

This proposal implements **Issue #334**:
1. Elevating `ce-ai gate check` from passive observation to an **active blocking gate** that rejects unauthorized `src/**` writes when Stage 4 OpenSpec contracts are missing.
2. Generating **structured validation receipts** (`.validation.json` and state-managed records) legible by `ce-ai doctor` and `status`.
3. Enforcing an explicit **policy table** to eliminate false positives on legitimate workflows (`ce-debug` bugfix exemption, `--tier minimal` projects, non-`src/**` paths, and edge case isolation).
4. Fixing the pre-existing **`openspec/changes/archive` mtime fallback bug** in `probe_openspec_context_in`.

## 2. In-Scope / Out-of-Scope Boundaries

### In-Scope:
1. **Deterministic Exit Code 2 Enforcement**:
   - Claude Code `PreToolUse` hook requires exit code `2` to abort tool execution and feed stderr back into the model context.
   - Any write under `src/**` during Stage 4 without approved `proposal.md`, `spec.md`, and `tasks.md` is rejected with exit code 2 and actionable remediation instructions.
2. **Policy Matrix Differentiation (Zero False Positives)**:
   - **Direct Entry Point Exemption**: `ce-debug` (task declared with `ce-debug` / bug fix) is permitted to write without upfront OpenSpec contracts.
   - **Adoption Tier Exemption**: Repositories adopted with `--tier minimal` do not enforce full OpenSpec contracts.
   - **Path Filtering**: Writes outside `src/**` (`docs/**`, `openspec/**`, `README.md`, config files) are never blocked.
   - **Edge Case Isolation**: States flagged as `mtime_fallback`, `worktree_uncommitted`, or `stale_cycle_guard` remain non-blocking (advisory only).
3. **Structured Validation Receipts**:
   - Schema `GateReceipt` recording timestamp, feature, target path, decision (`Passed`, `Blocked`, `Exempt`), stage, tier, missing artifacts, and reason.
   - Written to `openspec/changes/<feature>/.validation.json` when the feature folder exists.
   - Indexed in `state.json` under `state.gate_receipts: BTreeMap<String, GateReceipt>`.
   - Appended to `~/.ce-ai/gate-events.jsonl` with `decision: blocked`.
4. **Visibility in `doctor` and `status`**:
   - `ce-ai status` reports recent blocked write attempts.
   - `ce-ai doctor` validates gate receipts and reports blocked write findings.
5. **Pre-Existing Bugfix**:
   - In `probe_openspec_context_in` (`src/commands/workflow.rs`), explicitly ignore `archive/` and dot-prefixed directories to prevent false `archive` feature resolution via mtime fallback.
6. **Configurable Gate Mode & Kill-Switch**:
   - Configurable mode: `--mode enforce|observe` and env var `CE_AI_GATE_MODE=enforce|observe`.
   - Emergency kill-switch (`CE_AI_DISABLE_GATE_CHECK=1`, `--disabled`) preserved with immediate zero-I/O bypass.

### Out-of-Scope:
- Blocking shell execution bypasses (`bash -c "cat > src/file"`). Hook gates intercept tool calls (`Write`/`Edit`), not arbitrary shell sub-processes.
- Implementing blocking hooks for harnesses without tool interception support (Copilot, Cursor, etc.). Claude Code is the primary supported harness.

## 3. Risk Evaluation & Mitigations

| Risk | Likelihood | Impact | Mitigation Strategy |
|---|---|---|---|
| **False positive blocks legitimate bugfixes** | Medium | High | Explicit policy rule for `ce-debug` entry point; task string matching `ce-debug`/`debug` permits writes. |
| **False positive on minimal-tier projects** | Medium | High | `AdoptionTier::Minimal` explicitly bypasses full OpenSpec artifact checks. |
| **Lockout from ambiguous git state** | Low | High | Isolated edge cases (`mtime_fallback`, `worktree_uncommitted`, `stale_cycle_guard`) always degrade to non-blocking pass. |
| **Emergency block prevents recovery** | Low | Critical | Kill-switch env var `CE_AI_DISABLE_GATE_CHECK=1` short-circuits execution before any state checks. |
| **Silent non-blocking if wrong exit code used** | High | Critical | Verified via Claude Code binary inspection that Exit Code 2 is required; verified in tests. |

## 4. Success Criteria

1. `ce-ai gate check` returns Exit Code 2 with descriptive stderr when an unauthorized write to `src/**` is attempted in Stage 4 without OpenSpec.
2. `ce-ai gate check` returns Exit Code 0 when `ce-debug` is active, project is `--tier minimal`, or path is outside `src/**`.
3. Structured validation receipt `.validation.json` is generated upon gate evaluation and surfaced in `status` and `doctor`.
4. `probe_openspec_context_in` does not resolve `archive` as an active feature candidate.
5. 100% test coverage on policy decisions, receipt creation, exit code mapping, and doctor/status presentation.
6. `cargo clippy --all-targets --all-features -- -D warnings` and `cargo fmt --check` pass with zero warnings.
