# Tasks: `init-prj` dry-run output correctness

- [x] 1. Restructure `run()` in `src/commands/init_prj.rs` (~45 LOC)
  - Add a dry-run branch that invokes `reconcile_rtk_hooks_if_supported` when RTK is not opted out, else prints the opt-out notice.
  - Replace the unconditional final print with three modes: up-to-date message, dry-run preview line, real success line.
  - Keep the real (`!ctx.dry_run`) side-effect ordering unchanged (`State::load` stays after the writes).
- [x] 2. Integration tests in `tests/cli.rs` (~120 LOC)
  - `init_prj_dry_run_does_not_claim_adoption`: stdout contains `dry-run: would adopt project`, does not contain `Adopted project`, exits `0`.
  - `init_prj_dry_run_performs_no_writes`: no file created in the target or config dir, contents byte-identical.
  - `init_prj_dry_run_previews_rtk_hooks_on_fresh_project`: fresh project (no `.claude`, no installed harnesses) still previews `[dry-run] would configure rtk hook for claude`; no writes to project or `$HOME`.
  - `init_prj_dry_run_rtk_opt_out_skips_preview`: `--skip-rtk` prints the opt-out notice and no hook lines.
  - `init_prj_dry_run_quiet_emits_nothing`: `--quiet` in both modes prints nothing.
  - `init_prj_dry_run_up_to_date_reports_no_adoption`: second-run dry-run reports already-adopted, not "would adopt".
- [x] 3. Unit test in `src/commands/tests/init_prj.rs` (~15 LOC)
  - Assert `reconcile_rtk_hooks_if_supported` returns `Ok` and creates no files when `ctx.dry_run = true`.
- [x] 4. Verification gate (~0 LOC)
  - `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`.
- [x] 5. Version + changelog (~10 LOC)
  - Bump PATCH in `Cargo.toml`; add `CHANGELOG.md` entry referencing issue #351.
- [x] 6. Code-review remediation (~35 LOC)
  - P1 detection parity: pass `claude_rule_expected` so dry-run previews the Claude hook implied by the always-created `CLAUDE.md` stub.
  - P2/P3: keep `State::load` inside the real branch (restores real-run ordering) and make the dry-run registry read best-effort so a corrupt `state.json` cannot abort a write-free preview.
  - Tests for R4/R6 and the RTK no-write surface; `tasks.md` verification gate now names `make e2e`.

Total estimated LOC: ~225 lines.
