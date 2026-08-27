---
module: src/tui.rs
tags: [tui, ratatui, harness-parity, headless, zen, e2e, raw-mode]
problem_type: architecture
---

# TUI Paridad 9→15 y Estabilidad Input/Modal/TTY

## Problem
El TUI ofrecía 9 tabs funcionales (`MenuTab::all` 10 con `Exit`) pero el CLI expone 15 subcomandos (`src/main.rs:44`). `every_tui_spawned_vector_satisfies_its_cli_contract` pinneaba solo 5 vectores, dando 8/8 verde falso. `Upgrade` renderizaba selector de harness que `run_upgrade_cmd` ignoraba (mentira UI). Picker y modal colisionaban (`Esc` cerraba modal antes que picker), modal sin scroll truncaba `doctor`/`skills list`, atajos `j/k` movían tabs aunque el foco estaba en lista, `stdin.is_terminal()` fallaba en pipes/CI, y `disable_raw_mode` no se restauraba si `run_app` paniqueaba. Faltaban screenshots headless para probar overflow visual.

## Solution
**Paridad (U1):** `MenuTab` 10→15 — 5 tabs espejo `Skills/Tools/Usage/Audit/InitPrj` con `render_*` + `run_*_cmd` thin vía `capture_cli` (`src/tui.rs:86`, `render_content_panel:612`). Upgrade honesto sin selector (`tui.rs:760`).

**Estabilidad (U2):** `App` añade `output_scroll` + `state_error` banner rojo; `RawModeGuard` con `Drop` restaura `disable_raw_mode` + `LeaveAlternateScreen` incluso en panic; `stdout.is_terminal()` guard; input precedence `picker > modal > tabs`; modal `List+Scrollbar` con `j/k/PgUp/PgDn` y `Esc/Enter/q` close; gate `j/k` context-aware; split `F4 Doctor` / `F5 Backups` con confirm `y/N` y `HarnessKind::parse`.

**E2E zen (fix/tui-e2e-zen):** `every_tui...` 5→15 vectores, `headless_ui_renders_all_tabs` + `headless_screenshots_no_overflow` (`TestBackend 80×24`, dump a `tui-screenshots/` gitignored), `e2e_runner.sh:76` TUI headless checks con `opencode/zen-free` fallback y `cargo test tui` soft, `Dockerfile.e2e` mantiene `opencode-ai`.

## Verification
- `cargo test tui` 10/10 (15 vectores + 15 tabs headless + screenshots), `cargo test` 189/189, `cargo fmt`/`clippy -D warnings` ok.
- `/tmp/ce-ai-tui-audit.*` aislado: `install` → `verified 393/393` → tamper → `sync` restaura → `uninstall` limpia; `make e2e` verde con `size:exception` (400 líneas límite).

## Gotchas
- Rust `println!` paniquea con `Broken pipe` si `| grep -q` cierra pipe temprano — e2e debe redirigir a `/tmp/*.txt` antes de `grep`.
- `ratatui` emoji ocupa 2 celdas pero 1 char — overflow check debe permitir slop 84, no 80 estricto.
- `reinstall` en E2E sobrescribe backup y `uninstall` deja `plugin entry` — no reinstalar en E2E 8.
