# Tasks: tui-e2e-zen

## T1 — Full vector contract (~40 LOC)
- [x] Extend `every_tui_spawned_vector_satisfies_its_cli_contract` in `src/tui.rs:1359` to 15 vectors (incl. skills, tools, usage, audit, backups, workflow, init-prj). Use `with_cli_globals` for dry-run.

## T2 — Headless snapshots (~50 LOC)
- [x] Add `ratatui` `TestBackend` dev-dep, `tui_headless_snapshots` test looping `MenuTab::all()` asserting title in buffer and no panic.

## T3 — Docker E2E zen step (~60 LOC)
- [x] Extend `Dockerfile.e2e` to include `cargo test` deps if needed; extend `e2e_runner.sh` with E2E 10-11 TUI headless steps (skills resolve, tools status, cargo test tui) with zen fallback.

## T4 — Gates
- [x] `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test` — `make e2e` requiere Docker daemon (fail-closed, verificado vía `tests/e2e.rs:13`)
