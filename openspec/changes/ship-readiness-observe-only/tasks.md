# Tasks: Observe-only Ship-readiness gate

- [x] 1. State receipt storage in `src/state/state.rs` (~40 LOC)
  - `ReviewReceipt` struct; `State.review_receipts: BTreeMap<String, ReviewReceipt>`.
  - `review_receipt_for_branch` / `record_review_receipt` helpers keyed by `workspace_branch_key`.
- [x] 2. Git probes + readiness model in `src/commands/workflow.rs` (~110 LOC)
  - Extract `resolve_branch_base`; add `probe_commits_ahead`, `probe_stage6_artifact`.
  - Add `ReviewReceipt` fields to `RepoState`; populate in `probe_repo_state`.
  - Add `ShipReadinessGap`, pure `evaluate_ship_readiness`, `RepoState::ship_readiness_gaps`, `ship_readiness_lines`.
- [x] 3. `review-receipt` subcommand + resume/status wiring (~60 LOC)
  - `Action::ReviewReceipt { head, override_reason }`; record + confirm; `--dry-run` no write.
  - Insert readiness block in `resume_lines`; append warnings in `status_lines`.
- [x] 4. Doctor diagnostics in `src/commands/doctor.rs` (~25 LOC)
  - Non-fatal `doctor-warn: ship-readiness:` per gap when commits ahead.
- [x] 5. Tests (~130 LOC)
  - Unit: `evaluate_ship_readiness` truth table.
  - Integration: receipt record/present/stale, resume warnings, `--dry-run` no write.
  - Doctor: `doctor-warn` emitted, `run` stays `Ok`.
- [x] 6. Verification gate (~0 LOC)
  - `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`.
- [x] 7. Version + changelog (~10 LOC)
  - Bump MINOR in `Cargo.toml`; `CHANGELOG.md` entry referencing #354.
- [x] 8. Stage 6 compound + code review (~65 LOC docs)
  - `docs/solutions/` learning; run `ce-code-review` and remediate.

Total estimated LOC: ~375 code+test + ~65 docs.
