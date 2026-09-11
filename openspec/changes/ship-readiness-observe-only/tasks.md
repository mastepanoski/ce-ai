# Tasks: Observe-only Ship-readiness gate

> Revision after code review (issue #354): estimates reconciled to actuals and
> signals hardened (full-SHA identity, `--diff-filter=ACMR`, base fallbacks,
> adoption gating, no duplicated warnings). This is a rescope of the same change,
> not a widened budget for new scope.

- [x] 1. State receipt storage in `src/state/state.rs` (~42 LOC)
  - `ReviewReceipt` struct; `State.review_receipts: BTreeMap<String, ReviewReceipt>`.
  - `review_receipt_for_branch` / `record_review_receipt` keyed by `workspace_branch_key`.
- [x] 2. Git probes + readiness model in `src/commands/workflow.rs` (~281 LOC)
  - `resolve_branch_base` (main/master/upstream fallbacks), `probe_commits_ahead`, `probe_branch_added_files` (`--diff-filter=ACMR`), `probe_stage6_artifact`.
  - `probe_git_head_full_sha` + `resolve_commit_sha` for stable receipt identity.
  - `RepoState` fields (`commits_ahead`, `stage6_artifact_present`, `review_receipt`, `head_full_sha`); populated in `probe_repo_state`.
  - `ShipReadinessGap`, pure `evaluate_ship_readiness`, `RepoState::ship_readiness_gaps`, `ship_readiness_lines`.
- [x] 3. `review-receipt` subcommand + resume/status wiring (~folded into 2)
  - `Action::ReviewReceipt { head, override_reason }`; resolves full SHA, rejects unresolvable heads; `--dry-run` no write.
  - Resume reports the signals block; status owns the `! Warning:` gap lines (no duplication); both gated on adopted workspaces.
- [x] 4. Doctor diagnostics in `src/commands/doctor.rs` (~20 LOC)
  - Non-fatal `doctor-warn: ship-readiness:` per gap for adopted workspaces with commits ahead.
- [x] 5. Tests (~332 LOC)
  - Unit: evaluator truth table, gap descriptions, signal block, receipt record/override/dry-run/invalid-head, real-git commits+solutions, deletion ignored, receipt clear/stale.
  - Doctor: non-fatal on commits ahead. CLI: `review-receipt` record + dry-run inert.
- [x] 6. Verification gate (~0 LOC)
  - `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`.
- [x] 7. Version + changelog (~13 LOC)
  - Bump MINOR `1.50.2 -> 1.51.0`; `CHANGELOG.md` entry referencing #354.
- [x] 8. Stage 6 compound + code review (~90 LOC docs)
  - Multi-agent `ce-code-review` (correctness/testing/standards/adversarial) + P2 remediation.
  - `docs/solutions/architecture/observe-only-gate-signal-integrity.md`.

Total actual: ~675 code+test + ~90 docs (size exception documented in the PR, per CONTRIBUTING §PR Size Boundaries).
