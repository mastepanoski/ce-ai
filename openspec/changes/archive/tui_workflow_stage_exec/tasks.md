# Tasks: TUI Workflow Panel — Native Action Execution

TDD order per unit; verify each with `cargo test` before moving on.

## U1 — Workflow commands return output lines
- [x] RED: unit tests in src/commands/workflow.rs — status_lines returns real state-derived lines; checkpoint_lines writes formatted `last_update_check` and returns confirmation lines; corrupt state maps to `CeError::Io`/`CeError::State` without panic.
- [x] GREEN: implement per-action functions returning `Result<Vec<String>, CeError>`; keep `run()` signature, printing lines via `println!`.
- [x] Integration test: CLI `ce-ai workflow status` output matches the returned lines (single source of truth).

## U2 — TUI renders real output with failure class
- [x] RED: pure-helper tests — Ok(lines) → success block; Err(CeError) → ❌ failure block with actionable copy; `[1-7]` regression guard.
- [x] GREEN: rewire run_workflow_cmd through execute_action using U1 functions; wire checkpoint action through checkpoint_lines.

## U3 — Panel guide content rework
- [x] RED: rendered-lines tests — one marker (`[run]`/`skill:`) per stage row; Verify row toolchain-free; footer lists `[Enter]` + `[1-7]`; no resume hint.
- [x] GREEN: rework MenuTab::Workflow render block (src/tui.rs:552-582).

## U4 — Teacher-style documentation
- [x] Write docs/user-guide/workflow-panel.md (explanation intent) satisfying the AE5 checklist.
- [x] Add README doc-map line only if ≤100-line budget holds.
- [x] Verify against docs/references/docs-styling.md.

## U5 — Governance & verification
- [x] Bump SemVer (MINOR) in Cargo.toml and Formula/ce-ai.rb; update CHANGELOG.md.
- [x] Full gate: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e`.
- [x] Tick this file's boxes as units land.
