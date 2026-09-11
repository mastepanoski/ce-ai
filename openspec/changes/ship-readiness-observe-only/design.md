# Design: Observe-only Ship-readiness gate

## State (`src/state/state.rs`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReviewReceipt {
    pub head_sha: String,
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_reason: Option<String>,
}
```
- `State.review_receipts: BTreeMap<String, ReviewReceipt>` (`#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]`), keyed by `workspace_branch_key(root, branch)`.
- `State::review_receipt_for_branch(&self, root, branch) -> Option<&ReviewReceipt>`.
- `State::record_review_receipt(&mut self, root, branch, head_sha, override_reason) -> ReviewReceipt` (sets `recorded_at = Utc::now()`), persisted with `state.save` (atomic).

## Workflow (`src/commands/workflow.rs`)
- **Base resolution refactor**: extract `resolve_branch_base(repo_root) -> Option<String>` from `probe_branch_committed_files` and reuse it, so `probe_branch_committed_files` and `probe_commits_ahead` share one base. No behavior change.
- `probe_commits_ahead(repo_root) -> u32`: `git rev-list --count <base>..HEAD` (0 when no base).
- `probe_stage6_artifact(repo_root, dirty: &[String]) -> bool`: any `docs/solutions/**.md` in committed range **or** dirty/untracked.
- `RepoState` gains: `commits_ahead: u32` (`#[serde(default)]`), `stage6_artifact_present: bool` (`#[serde(default)]`), `review_receipt: Option<ReviewReceipt>` (`skip_serializing_if = None`). Populated in `probe_repo_state` (receipt via best-effort `State::load`).
- **Pure evaluator** (no I/O, unit-tested):
```rust
pub enum ShipReadinessGap { Stage6Missing, ReviewReceiptMissing, ReviewReceiptStale, EarlyStageWithCommits }

pub fn evaluate_ship_readiness(
    commits_ahead: u32,
    stage6_artifact_present: bool,
    receipt: Option<&ReviewReceipt>,
    head_sha: Option<&str>,
    stage: WorkflowStage,
) -> Vec<ShipReadinessGap>;
```
  Returns empty when `commits_ahead == 0`. Gap order: Stage6 -> ReviewReceipt(missing/stale) -> EarlyStageWithCommits (`stage < Verification`).
- `RepoState::ship_readiness_gaps(&self, stage)` delegates to it.
- `ship_readiness_lines(&RepoState, stage) -> Vec<String>` renders the `== [Ship Readiness (observe-only)] ==` block plus `! Warning:` lines; empty when no commits ahead.
- `resume_lines` inserts the block after the drift/ledger section; `status_lines` appends the warnings too.
- New subcommand `Action::ReviewReceipt { head: Option<String>, override_reason: Option<String> }`: resolves HEAD (or `--head`), records the receipt, prints confirmation; no-op write under `--dry-run`.

## Doctor (`src/commands/doctor.rs`)
- After the existing OpenSpec warnings: when `commits_ahead > 0`, evaluate readiness with the current workflow stage and print one `doctor-warn: ship-readiness: <gap>` per gap. Never pushes to `findings`; exit code unchanged.

## Invariants
- Observe-only: no command returns a blocking error for gaps; all exit codes unchanged.
- All state mutations use `write_atomic` via `State::save`.
- Git/state probing is best-effort: missing base, non-git dir, or absent state never panics or errors.
- `RepoState` is serialized in the resume JSON payload; new fields are additive and backward compatible.

## Test Strategy
- Unit (`src/commands/tests/workflow.rs`): `evaluate_ship_readiness` truth table (no commits; all present; missing artifact; missing receipt; stale receipt; early stage).
- CLI/integration: `review-receipt` records and resume reports present; stale after a new commit; resume warns when commits ahead with no artifact/receipt; `--dry-run` records nothing.
- Doctor (`src/commands/tests/doctor.rs`): a repo-scoped fixture with commits ahead emits `doctor-warn: ship-readiness:` and `run` stays `Ok`.
