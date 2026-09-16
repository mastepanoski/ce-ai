# Technical Exploration: Adaptive Mode Router & Graduation Bridge

## Context & Problem Space

`ce-ai` combines Compound Engineering with multi-harness agent management. It enforces structured stage transitions and validation gates to protect repository integrity. However, enforcing the full 5-file OpenSpec contract on every bugfix or minor chore creates ceremony overload and burns LLM tokens. Gentle AI solved this friction in its ecosystem through **Organic Driven Development (ODD)**, centering developer and agent focus on a single brief containing:
- **Problem Statement**
- **Guardrails & Invariants**
- **Definition of Done (DoD)**

This exploration investigates how `ce-ai` can seamlessly accommodate both paradigms: lightweight Organic Driven Development for everyday tasks (< 200 LOC) and full 7-stage Compound Engineering for structural architecture, connected by a lossless graduation bridge.

## Architectural Tradeoffs & Evaluated Alternatives

### 1. Mode Router: LLM Agent vs Deterministic In-Binary Classifier

| Dimension | Option A: Runtime LLM Routing Agent | Option B: Deterministic In-Binary Heuristics (Selected) |
|---|---|---|
| **Latency** | 1,500ms – 3,500ms per session start | < 5ms (pure in-memory string prefix & stat) |
| **Token Cost** | ~500–1,200 tokens per invocation | 0 tokens (pure Rust) |
| **Reliability** | Non-deterministic; prompt injection risk | 100% deterministic; reproducible in tests |
| **Offline Support** | Fails in air-gapped / offline dev environments | Works everywhere with zero network dependence |
| **Edge Cases** | Can hallucinate mode on ambiguous requests | Simple precedence rules (`--mode` flag, OpenSpec dir presence) |

**Decision:** Option B. Routing must execute inside `ce-ai workflow resume`, which is invoked on Turn-0 of agent sessions and hook lifecycles. Sub-15ms performance is a hard architectural requirement of `ce-ai` (see `openspec/specs/workflow.md` R2).

### 2. Gate Policy on Tactical LOC Ceiling: Hard Block vs Observe-Only Advisory

| Policy | Behavior on Diff > 200 LOC in Organic Mode | Tradeoffs |
|---|---|---|
| **Hard Gate (Blocking, Exit 2)** | Immediately blocks file writes and returns tool failure to harness | Frustrates developers and AI agents in the middle of active debugging; causes loop retries |
| **Observe-Only Advisory (Exit 0) (Selected)** | Emits clear diagnostic notice recommending graduation, allows tool writes | Non-disruptive; provides actionable guidance while honoring developer velocity |
| **No Ceiling Check** | Allows infinite LOC drift in Organic Mode | Risk of unbounded architectural changes bypassing living specifications and review receipts |

**Decision:** Observe-only advisory. Following `ce-ai`'s philosophy of deterministic guidance without spurious agent interruptions, the gate notifies the agent:
`"Notice: Organic task diff exceeds 200 LOC ceiling. Consider running 'ce-ai workflow graduate' to formalize in OpenSpec."`

### 3. Graduation Bridge: Dual-Ledger Sync vs Destructive Cleanup vs Lossless Migration

| Strategy | Mechanism | Evaluation |
|---|---|---|
| **A: Dual-Ledger Sync** | Keep both `odd/tasks/<feature>.md` and `openspec/changes/<feature>/` active and try to synchronize | High risk of desynchronization, token duplication, conflicting status checkboxes |
| **B: Manual Copy** | Instruct developer/agent to manually create OpenSpec files | High friction, human error, inconsistent structure |
| **C: Lossless Migration & Cleanup (Selected)** | Transform ODD sections to OpenSpec files, flush to disk, delete ODD source file | Single source of truth at all times; zero duplicate tracking; deterministic promotion |

**Decision:** Option C. `ce-ai workflow graduate <feature>` mechanically maps:
- `# Problem Statement` ➔ `openspec/changes/<feature>/proposal.md`
- `# Guardrails & Invariants` ➔ `openspec/changes/<feature>/spec.md`
- `# Definition of Done (DoD)` ➔ `openspec/changes/<feature>/tasks.md` (preserving `[x]` completion)
- Deletes `odd/tasks/<feature>.md` upon successful disk commit.
- Updates `state.json` directly to Stage 4 (`WorkTdd`).

## System Injection Points in `ce-ai`

1. **`src/state/state.rs`:**
   - Add `ExecutionMode` enum:
     ```rust
     #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
     #[serde(rename_all = "lowercase")]
     pub enum ExecutionMode {
         #[default]
         Auto,
         Organic,
         Compound,
     }
     ```
   - Add `pub execution_mode: Option<ExecutionMode>` to `WorkflowState` with serde defaults.
2. **`src/commands/workflow.rs`:**
   - Implement `probe_execution_mode(repo_root, branch, wf, tier, cli_override) -> ExecutionMode`.
   - Heuristics:
     1. CLI `--mode` override flag takes absolute precedence.
     2. OpenSpec directory presence (`openspec/changes/<feature>/`) takes strict precedence (Compound).
     3. Non-git workspace: check directory presence (`odd/tasks/` vs `openspec/changes/`), then `AdoptionTier`.
     4. Git branch prefix: `fix/*`, `chore/*`, `spike/*`, `test/*` ➔ `Organic`; `feat/*`, `spec/*` ➔ `Compound`.
     5. Active `odd/tasks/<feature>.md` ➔ `Organic`.
   - Update `resume_lines()` to render ODD banner vs 7-Stage FSM banner.
   - Implement `generate_odd_task_template` and `parse_odd_task_file`.
   - Implement `run_graduate(ctx, args)`.
3. **`src/commands/registry.rs`:**
   - Register subcommand `Action::Graduate(GraduateArgs)` under `workflow` and top-level alias `Commands::Graduate(GraduateArgs)`.
4. **`src/commands/gate.rs`:**
   - Inspect active `ExecutionMode`.
   - If `Organic`, bypass OpenSpec package requirement.
   - Compute uncommitted git diff line count. If > 200 LOC, append observe-only advisory message to reasons while returning `GateDecision::Pass` (exit code 0).
5. **`tests/cli.rs`:**
   - Add full integration test suite verifying the complete lifecycle: ODD creation ➔ Organic resume banner ➔ Gate pass ➔ Graduation ➔ OpenSpec files verification ➔ Clean removal of ODD file.
