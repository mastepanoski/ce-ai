# Spec: tui-e2e-zen

## ADDED Requirements

### Requirement TUI-E2E-1: Full TUI vector contract
Every vector spawned by `src/tui.rs` via `capture_cli` SHALL parse against its subcommand's live `clap` surface. The test `every_tui_spawned_vector_satisfies_its_cli_contract` SHALL cover all 15 CLI subcommands reachable from TUI (install, sync, upgrade, models list, skills list/resolve/doctor/adopt, status, uninstall, doctor, backups list, tools status, usage report, workflow status, audit, init-prj). Vectors that require `--harness` SHALL use a concrete harness (`opencode`) and `with_cli_globals` where `--dry-run` is global.

#### Scenario: Full coverage
- **WHEN** `cargo test --test tui` runs
- **THEN** the contract test asserts 15 vectors parse and fails if any new CLI flag breaks TUI.

### Requirement TUI-E2E-2: Headless TUI snapshots
`ui()` SHALL render without a TTY via `ratatui::backend::TestBackend` for each `MenuTab` (10 tabs). Test SHALL assert no panic, no layout overflow, and buffer contains tab title.

#### Scenario: Snapshot
- **WHEN** `cargo test tui_snapshots` runs
- **THEN** each tab renders into `TestBackend(80,24)` and its title appears in the buffer.

### Requirement TUI-E2E-3: Docker E2E TUI zen step
`e2e_runner.sh` SHALL, after install/sync/status gates, run `ce-ai skills resolve --harness opencode "test"` and `ce-ai tools status` headless, and assert `cargo test tui` passes inside the container. If `OPENCODE_ZEN_API_KEY` is absent, step MAY fallback to mock model discovery. Docker absence SHALL fail the gate (`FAILED-TO-RUN`) per `tests/e2e.rs:13`.

#### Scenario: E2E TUI gate
- **WHEN** `make e2e` runs on host with Docker
- **THEN** container executes TUI headless checks and exits 0; without Docker it exits non-zero with guidance.
