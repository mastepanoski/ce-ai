# Specification: Observe-only Ship-readiness gate

## Requirements

### R1: Ship-readiness evaluation
- WHEN an **adopted** workspace has `commits_ahead > 0` relative to the resolved base, THEN ship readiness MUST evaluate and report:
  - commits ahead count,
  - Stage 6 artifact presence (`docs/solutions/**.md` added/modified on the branch, or a dirty file that still exists on disk),
  - code-review receipt presence and freshness against the current HEAD.
- The base MUST be resolved by trying `origin/main`, `main`, `origin/master`, `master`, then `@{upstream}`. WHEN no base resolves, THEN readiness MUST NOT be reported as satisfied (no false "ready") — the block is omitted.
- WHEN `commits_ahead == 0`, THEN the readiness block MUST be omitted (no gaps).
- The evaluation MUST be pure and deterministic given `(commits_ahead, stage6_present, receipt, head_sha, stage)`.

### R2: Gap classification
- WHEN commits are ahead and no `docs/solutions/**.md` artifact was **added or modified** on the branch (deletions MUST NOT count as presence), THEN the evaluator MUST report `Stage6Missing`.
- WHEN commits are ahead and no receipt exists, THEN it MUST report `ReviewReceiptMissing`.
- WHEN commits are ahead and a receipt exists whose full SHA differs from the current full HEAD, THEN it MUST report `ReviewReceiptStale`.
- WHEN commits are ahead and the checkpoint stage is earlier than `Verification`, THEN it MUST report `EarlyStageWithCommits`.
- Gaps MUST be emitted in that order.

### R3: Receipt recording
- WHEN `ce-ai workflow review-receipt` runs, THEN it MUST record a `ReviewReceipt { head_sha (full 40-char SHA), recorded_at }` in `state.json` keyed by workspace+branch.
- The head MUST be resolved to a full commit SHA via `git rev-parse --verify <spec>^{commit}` (HEAD when `--head` is absent). WHEN it cannot be resolved, THEN the command MUST fail with a Usage error (exit `2`) and MUST NOT store a placeholder.
- WHEN `--override-reason <text>` is supplied, THEN the receipt MUST store it verbatim for audit.
- WHEN `--dry-run` is supplied, THEN no state MUST be written.
- Recording MUST use the atomic writer; a corrupt `state.json` MUST return `CeError::State` (absent -> new state).

### R4: Resume and status reporting
- WHEN `ce-ai workflow resume` runs with commits ahead in an adopted workspace, THEN output MUST include the ship-readiness signals block and each gap as a `! Warning:` line, **without duplicating** a warning already emitted by the nested status renderer (status owns the gap lines; the resume block reports signals only).
- WHEN `ce-ai workflow status` runs with commits ahead in an adopted workspace, THEN the gap warnings MUST appear in its output.
- These commands MUST still exit `0`; readiness is advisory and MUST NOT block.

### R5: Doctor diagnostics
- WHEN `ce-ai doctor` runs in an **adopted** workspace with commits ahead and any readiness gap, THEN it MUST print `doctor-warn: ship-readiness: <gap>` for each gap and MUST NOT add them to fatal `findings`.
- `doctor`'s exit code MUST be unaffected by ship-readiness warnings.

### R6: Resume awareness
- WHEN `ce-ai workflow resume` finds commits ahead of base, THEN it MUST report the committed-ahead count and the base-relative state, and MUST NOT re-run or reset earlier-stage checkpoints (reporting only).
- WHEN the checkpoint stage is earlier than the committed work implies, THEN `EarlyStageWithCommits` MUST be reported.

### R7: Scope
- Readiness probing MUST be limited to adopted workspaces (`adoption_status` present / `State::is_project_adopted`). Unrelated git repositories MUST NOT receive ship-readiness warnings.

## Acceptance Criteria Mapping
- #354 "warn on missing `docs/solutions` artifact" -> R1/R2/R4.
- #354 "warn on missing code-review receipt" -> R1/R2/R3/R4.
- #354 "resume reports committed/unpushed, no silent re-run" -> R6.
- #354 "override possible and recorded" -> R3 `--override-reason` (observe-only: no block to override yet; enforcement deferred to #334).
