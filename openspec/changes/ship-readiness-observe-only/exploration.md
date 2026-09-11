# Exploration: Ship-readiness signals

## Question
What existing signals can tell whether a managed change reached Stage 6 and had a code review, without adding blocking behavior or external dependencies?

## Findings (current tree)
- `src/commands/workflow.rs`
  - `probe_repo_state` builds `RepoState` (branch, head sha, dirty files, manifest drift, adoption, OpenSpec context, task desync, unarchived changes). No readiness concept.
  - `probe_branch_committed_files` resolves a base via `git merge-base HEAD origin/main` (fallback `main`) then `git diff --name-only <base>...HEAD`. There is **no commits-ahead count** helper.
  - `has_committed_solutions_on_branch` checks only the **HEAD commit** (`git diff-tree ... HEAD`), not the base..HEAD range — too narrow for "Stage 6 artifact on this branch".
  - `infer_stage_from_repo` already checks dirty `docs/solutions/**.md` inline, and `maybe_auto_checkpoint` uses it monotonically — so resume already infers stage; it just does not *report* the committed-but-early mismatch.
  - `resume_lines` (env/drift/OpenSpec blocks) and `status_lines` are the natural print surfaces.
- `src/state/state.rs`: `State` has no receipt storage; `workflows: BTreeMap<String, WorkflowState>` is keyed by `workspace_branch_key` (canonical root + branch). `write_atomic` is available; `save` uses it.
- `src/commands/gate.rs` proves the observe-only pattern: pure decision fn + a `run` that always returns `Ok(())`.
- `src/commands/doctor.rs` collects fatal `findings` but emits non-fatal `doctor-warn:`/`doctor-info:` via `println!`; workspace OpenSpec probing already sits at the end of `run`.

## Options Evaluated
- **A — Detect review from external artifacts** (GitHub PR reviews, CI artifacts): network-dependent, unreliable offline, and conflates review *existence* with review *scope*. Rejected.
- **B — Record a branch/head-scoped receipt in `state.json`** via a dedicated command, and evaluate readiness from local git + state. Fully offline, deterministic, testable. Chosen.
- **C — Implement the blocking gate now**: that is #334's job and depends on the observe-only results (#333/#334). Deferred.

## Decision
**Option B, observe-only.** Evaluate readiness from local signals, record receipts in `state.json`, warn everywhere, block nowhere. Blocking + override enforcement stays with #334, which can consume this receipt.
