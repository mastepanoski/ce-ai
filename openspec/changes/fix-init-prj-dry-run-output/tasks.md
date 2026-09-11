# Tasks: `init-prj` dry-run output correctness

- [x] 1. Restructure `run()` in `src/commands/init_prj.rs` (~45 LOC)
  - Load `State` unconditionally before the dry-run branch (read-only).
  - Add a dry-run branch that invokes `reconcile_rtk_hooks_if_supported` when RTK is not opted out, else prints the opt-out notice.
  - Replace the unconditional final print with three modes: up-to-date message, dry-run preview line, real success line.
  - Keep the real (`!ctx.dry_run`) side-effect ordering unchanged.
- [x] 2. Integration tests in `tests/cli.rs` (~60 LOC)
  - `init_prj_dry_run_does_not_claim_adoption`: stdout contains `dry-run: would adopt project`, does not contain `Adopted project`, exits `0`.
  - `init_prj_dry_run_performs_no_writes`: snapshot target tree + `state.json` before/after, assert no file created and contents byte-identical.
  - `init_prj_dry_run_previews_rtk_hooks_or_opts_out`: with RTK available, assert `[dry-run]` hook lines appear; with `--skip-rtk`, assert the opt-out notice and no hook lines.
- [x] 3. Unit test in `src/commands/tests/init_prj.rs` (~15 LOC)
  - Assert `reconcile_rtk_hooks_if_supported` returns `Ok` and creates no files when `ctx.dry_run = true`.
- [x] 4. Verification gate (~0 LOC)
  - `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
- [x] 5. Version + changelog (~10 LOC)
  - Bump PATCH in `Cargo.toml`; add `CHANGELOG.md` entry referencing issue #351.

Total estimated LOC: ~130 lines.
