# Technical Design: Phase 3 — Advisory Workflow FSM & OpenSpec Decoupling (v1.77.0)

## System Architecture

```text
┌────────────────────────────────────────────────────────────────────────┐
│                        Repository Reality (SSOT)                       │
│    Git Branch  •  Uncommitted Diffs  •  Markdown Plans  •  Receipts    │
└────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                 Advisory Workflow Observation Engine                   │
│                       (src/observation/state.rs)                       │
│                                                                        │
│   ObservableWorkflowState:                                             │
│   • active_work: bool, active_branch: Option<String>                   │
│   • plan: Option<PlanObservation>                                      │
│   • verification: VerificationObservation                              │
│   • handoff: Option<HandoffObservation>                                │
│   • knowledge_capture: KnowledgeObservation                            │
│   • openspec: Option<OpenSpecObservation> (Optional / Decoupled)       │
└────────────────────────────────────────────────────────────────────────┘
          │                                              │
          ▼                                              ▼
┌───────────────────────────────┐              ┌─────────────────────────┐
│     ce-ai workflow status     │              │    ce-ai gate check     │
│   (Advisory Capabilities      │              │ (Observe Mode Default   │
│       Matrix Diagnostic)      │              │  Zero Blocking Writes)  │
└───────────────────────────────┘              └─────────────────────────┘
```

## Data Schemas & Types

### 1. `src/observation/mod.rs` & `src/observation/state.rs`

```rust
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservableWorkflowState {
    pub active_work: bool,
    pub active_branch: Option<String>,
    pub plan: Option<PlanObservation>,
    pub verification: VerificationObservation,
    pub handoff: Option<HandoffObservation>,
    pub knowledge_capture: KnowledgeObservation,
    pub openspec: Option<OpenSpecObservation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanObservation {
    pub path: PathBuf,
    pub title: Option<String>,
    pub completed_items: usize,
    pub total_items: usize,
    pub is_requirements_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationObservation {
    pub has_evidence: bool,
    pub uncommitted_changes: usize,
    pub review_receipt_stamped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandoffObservation {
    pub exists: bool,
    pub path: PathBuf,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnowledgeObservation {
    pub required: bool,
    pub doc_detected: bool,
    pub doc_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenSpecObservation {
    pub feature: String,
    pub path: PathBuf,
    pub is_sealed: bool,
}
```

### 2. Observation Logic (`ObservableWorkflowState::observe`)

1. **Active Branch & Work:**
   - Queries `git rev-parse --abbrev-ref HEAD` or falls back to reading `.git/HEAD`.
   - Checks `git status --porcelain` to determine `uncommitted_changes`. If > 0, `active_work = true`.
2. **Plan Detection:**
   - Discovers `docs_root` via `crate::compat::CeDocsConfig::discover(repo_root)`.
   - Scans `plans_dir` for `.md` plan files, sorting by modification time to find the active plan.
   - Parses checkbox items `- [x]` vs `- [ ]` to calculate `completed_items` and `total_items`.
   - Checks frontmatter or contents for requirements-only indications.
3. **Verification Observation:**
   - Reads `state.review_receipts` or branch review stamps to evaluate `review_receipt_stamped`.
   - Checks presence of test run receipts or test artifacts.
4. **Handoff Observation:**
   - Checks for `docs/plans/handoff.md`, `ce-handoff` output, or recent handoff checkpoints.
5. **Knowledge Observation:**
   - Checks if modified files include non-trivial core code (`src/**`) and searches for recently modified solution docs in `CeDocsConfig::solutions_dir`.
6. **OpenSpec Observation (Optional):**
   - Checks `openspec/changes/` for subdirectories excluding `archive`.
   - If a change folder exists matching the current branch name or most recently modified, populates `Some(OpenSpecObservation)`.
   - If no change directory exists, returns `None` without error or warning.

### 3. Gate Policy Engine Refinement (`src/commands/gate.rs`)

- Change `resolve_gate_mode` fallback from `GateMode::Enforce` to `GateMode::Observe`.
- Update `evaluate_gate_policy`:
  - When `mode == GateMode::Observe`, any missing OpenSpec artifacts are evaluated as `GateDecision::WouldBlock` (or `GateDecision::Pass` when OpenSpec is not configured), and the gate NEVER returns `GateDecision::Blocked`.
  - When OpenSpec is optional and not active on the branch, code writes pass freely with explanatory reason `"OpenSpec is decoupled; writes permitted under Compound Engineering workflow"`.

### 4. Status Command Integration (`src/commands/workflow.rs`)

- In `status_lines_with_mode`:
  - Invoke `ObservableWorkflowState::observe(repo_root, ctx.config_dir)`.
  - Print the **Advisory Capabilities Matrix**:
    ```text
    == [Advisory Workflow Capabilities Matrix] ==
      • Active Branch        ➔ feat/my-feature (work in progress)
      • Plan Observation     ➔ 2026-10-05-my-plan.md (4/8 items completed)
      • Verification         ➔ 0 uncommitted changes, review receipt stamped
      • Knowledge Capture    ➔ Doc detected in docs/solutions/
      • OpenSpec Integration ➔ Inactive (Optional)
    ```
  - In `Action::Status { json: true, .. }`, include `"observable_state": <ObservableWorkflowState>`.
