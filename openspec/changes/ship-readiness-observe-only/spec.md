# Specification: Observe-only Ship-readiness gate

## Requirements

### R1: Ship-readiness evaluation
- WHEN an adopted workspace has `commits_ahead > 0` relative to the resolved base (`origin/main` -> `main` fallback), THEN ship readiness MUST evaluate and report:
  - commits ahead count,
  - Stage 6 artifact presence (`docs/solutions/**.md` in the committed range or dirty/untracked),
  - code-review receipt presence and freshness against the current HEAD.
- WHEN `commits_ahead == 0`, THEN the readiness block MUST be omitted (no gaps).
- The evaluation MUST be pure and deterministic given `(commits_ahead, stage6_present, receipt, head_sha, stage)`.

### R2: Gap classification
- WHEN commits are ahead and no `docs/solutions/**.md` artifact is present on the branch, THEN the evaluator MUST report `Stage6Missing`.
- WHEN commits are ahead and no receipt exists, THEN it MUST report `ReviewReceiptMissing`.
- WHEN commits are ahead and a receipt exists whose `head_sha` differs from the current HEAD, THEN it MUST report `ReviewReceiptStale`.
- WHEN commits are ahead and the checkpoint stage is earlier than `Verification`, THEN it MUST report `EarlyStageWithCommits`.
- Gaps MUST be emitted in that order.

### R3: Receipt recording
- WHEN `ce-ai workflow review-receipt` runs on an adopted branch, THEN it MUST record a `ReviewReceipt { head_sha, recorded_at }` in `state.json` keyed by workspace+branch, using HEAD when `--head` is not given.
- WHEN `--override-reason <text>` is supplied, THEN the receipt MUST store it verbatim for audit.
- WHEN `--dry-run` is supplied, THEN no state MUST be written.
- Recording MUST use the atomic writer; a corrupt/absent `state.json` MUST NOT panic (absent -> new state; corrupt -> `CeError::State`).

### R4: Resume and status reporting
- WHEN `ce-ai workflow resume` runs with commits ahead, THEN output MUST include the ship-readiness block naming each gap as a `! Warning:` line.
- WHEN `ce-ai workflow status` runs with commits ahead, THEN the same warnings MUST appear in its output.
- These commands MUST still exit `0`; readiness is advisory and MUST NOT block.

### R5: Doctor diagnostics
- WHEN `ce-ai doctor` runs in an adopted workspace with commits ahead and any readiness gap, THEN it MUST print `doctor-warn: ship-readiness: <gap>` for each gap and MUST NOT add them to fatal `findings`.
- `doctor`'s exit code MUST be unaffected by ship-readiness warnings.

### R6: Resume awareness
- WHEN `ce-ai workflow resume` finds commits ahead of base, THEN it MUST report the committed-ahead count and the base-relative state, and MUST NOT re-run or reset earlier-stage checkpoints (reporting only).
- WHEN the checkpoint stage is earlier than the committed work implies, THEN `EarlyStageWithCommits` MUST be reported.

## Acceptance Criteria Mapping
- #354 "warn on missing `docs/solutions` artifact" -> R1/R2/R4.
- #354 "warn on missing code-review receipt" -> R1/R2/R3/R4.
- #354 "resume reports committed/unpushed, no silent re-run" -> R6.
- #354 "override possible and recorded" -> R3 `--override-reason` (observe-only: no block to override yet; enforcement deferred to #334).
