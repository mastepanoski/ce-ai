---
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-ideate
execution: code
date: 2026-08-26
feature: tui-paridad-y-estabilidad
ideation: docs/ideation/2026-08-26-tui-errores-y-paridad-funcional-ideation.md
---

# Plan: TUI Paridad y Estabilidad (fix + tab parity)

## Goal Capsule

**Objetivo:** Cada función ofrecida por el TUI haga exactamente lo que promete, verificada en `/tmp` aislado y en Docker con `opencode zen-free` headless; cerrar 9→15 paridad sin reescribir el TUI como segundo CLI.

**Medio:** Delegación thin a `capture_cli` (ya existe); añadir tabs espejo, arreglar `Upgrade` honesto, picker/modal stacking, atajos sin colisión, TTY guard + banner degradado; candado `every_tui_spawned_vector` a 15 vectores y snapshots `TestBackend`.

**Stop:** `MenuTab::all().len()` == 15 (14 func + `Exit`; `Commands` en `src/main.rs:44` = 14 func + `help` = 15) + `cargo test tui` 15/15 verde + `make e2e` con paso TUI headless verde.

## Product Contract

### Actors
- A1 Dev junior en TUI (pestañas, instala/sync sin salir del dashboard)
- A2 ce-ai CLI (contrato live en `src/main.rs:44`)
- A3 Harness (opencode, claude, pi, etc. — `HarnessKind`)

### Flows
- F1 Seleccionar harness con `◄/►`, `[Enter]` ejecuta `install/sync` con vector correcto; dry-run togglable visible.
- F2 Models: `[n/p]` slot, `[m]` abre picker catalog `opencode models` headless (zen-free fallback), `Enter` aplica.
- F3 Workflow: `status` lee `state.json`, `[1-7]` guarda checkpoint atómico.
- F4 Doctor: `[Enter]` corre `doctor`, modal scrolleable muestra findings, `CeError` → `❌` + remedio `ce-ai doctor`.
- F5 Backups: lista `backups list` filtrada por harness, `[Enter]` pide confirm `y/N` en modal, `HarnessKind::parse` + `restore_backup_by_id` con `HarnessKind` correcto, `prune_empty_dirs` tras delete.

### Requirements
- **R1 Paridad 15:** TUI debe spawnear vectores válidos para 14 comandos funcionales + `help` (15 con `help`; 10 tabs actuales = 9 func + Exit, tras U1 = 14 func + Exit = 15). Mapping: `Skills`→`skills list/resolve/doctor`, `Tools`→`tools status`, `Usage`→`usage report`, `Audit`→`audit`, `InitPrj`→`init-prj`/`deinit-prj` (toggle por estado). Cada vector validado en `every_tui_spawned_vector` 15/15.
- **R2 Upgrade honesto:** Upgrade no muestra selector harness; siempre "todos" (`run_upgrade_cmd` ignora `selected_harness_target`).
- **R3 Picker/modal correcto:** Picker captura input exclusivo; `Esc` cancela picker, no cierra modal subyacente; `output_modal` no roba `Esc` mientras picker abierto. Precedencia: picker > modal > tabs.
- **R4 Modal scrolleable:** Output >70% viewport debe ser scrolleable (`List`+`Scrollbar` en `render_modal:870`), no truncado; `PgUp/PgDn` y `j/k` con viewport.
- **R5 Atajos sin colisión:** `j/k` y `Up/Down` solo navegan tabs cuando no hay lista/modal enfocada; `n/p` solo en Models (`current_tab()==Models`).
- **R6 TTY guard + banner:** `run_interactive:220` chequea `stdout.is_terminal()` (no `stdin`) y muestra `Usage` accionable; `reload_state:148` muestra banner rojo si `State::load` falla (corrupto/permisos) en vez de `unwrap_or_default` silencioso.

### Acceptance Examples
- AE1 `cargo test` `every_tui_spawned_vector` 15/15 verde; romper `--harness` en upgrade hace fail.
- AE2 `TestBackend(80,24)` renderiza cada tab sin panic y su keyword en buffer.
- AE3 En `/tmp` aislado: `install --harness opencode` → `status` `verified 393/393` → tamper file → `sync` restaura → `uninstall` limpia.
- AE4 `make e2e` dentro Docker: `skills list/resolve` + `tools status` + `models set zen-free` headless pasan.

### Scope
- In: R1-R6, I7/I15 ya en `fix/tui-e2e-zen` (reuso), I1/I3/I5/I6/I13 + RAII guard (I8) + split F4 aquí.
- Out: Paleta `:` fuzzy (I2), footer dinámico (I14) → defer v2.

## Planning Contract

### KTDs
- **KTD1 Tabs vs palette:** Tabs espejo (costo fijo) elegidos sobre palette fuzzy (nuevo paradigma, más riesgo) — respeta sidebar existente.
- **KTD2 Delegación vs duplicación:** TUI no duplica lógica CLI; solo construye vector + captura subprocess (#72).

### Technical Design
- `src/tui.rs:40` `MenuTab` + `render_content_panel:597` + helpers `*_cmd_args` + `capture_cli:974` + `ui:432`.
- Nuevos tabs: `Skills`, `Tools`, `Usage`, `Audit`, `InitPrj` (5) — cada uno `render_*` + `run_*_cmd` thin.
- Fixes: `run_app:247` input precedence (modal→picker→tab), `render_modal:870` → `List`+`Scrollbar`, `run_interactive:220` `stdout.is_terminal()`, `reload_state:148` banner.

### Assumptions
- `opencode zen-free` disponible headless sin API key para `discover_models`; fallback mock si no.
- `ratatui TestBackend` suficiente para layout; no necesita crossterm real.

### Sequencing
- PR1 (fix/tui-e2e-zen) ya mergeable: I7+I15 base.
- PR2 aquí (feat/tui-paridad): R1 Tabs (I1) + R2 Upgrade honesto (I3) — ~200 LOC
- PR3: R3 Picker + R4 Modal scroll + R5 Atajos + R6 TTY/banner — ~180 LOC

## Implementation Units

### U1 — Tabs espejo y Upgrade honesto (~180 LOC) — R1,R2
- Extiende `MenuTab::all/title` con 5 tabs (`Skills`, `Tools`, `Usage`, `Audit`, `InitPrj` con toggle init/deinit); añade `render_skills/tools/usage/audit/init_prj_panel` y `*_cmd_args`/`run_*_cmd` thin vía `capture_cli` (validación `HarnessKind::parse` previa, no inyección).
- Quita selector harness de `MenuTab::Upgrade` panel (`tui.rs:760`); `run_upgrade_cmd:1145` ignora `app.selected_harness_target` y vector es `["upgrade"]` pinneado.
- Tests: `every_tui...` 15/15 ya hecho en `fix/tui-e2e-zen` + `headless_ui_renders_all_tabs` debe pasar con 15 tabs (80x24).

### U2 — Picker/modal stacking + scroll + atajos + TTY + RAII (~200 LOC) — R3-R6 + I8
- Reordena `run_app:247` precedence: `if picker_open` antes de `if output_modal.is_some()`; `Esc` cierra picker, `Enter` aplica `models::set`; modal `render_modal:870` pasa a `List`+`Scrollbar` con viewport 70%, `j/k`/`PgUp/PgDn` scrollea. **Split F4/F5:** `run_doctor_cmd` y `run_restore_backup_cmd` con confirm `y/N` modal y `HarnessKind` parse correcto (no `unwrap_or(Opencode)` silencioso).
- Gate `j/k` y `Up/Down` por contexto: solo tabs si `!picker_open && output_modal.is_none() && current_tab != Models` para `n/p`.
- `run_interactive:220` `stdout.is_terminal()` (no `stdin`) + `RawModeGuard` RAII (`Drop` restaura `disable_raw_mode` + `LeaveAlternateScreen` incluso si `run_app` paniquea) + `reload_state:148` banner rojo `⚠️ state.json corrupt: {err} — run ce-ai doctor` en header.
- Tests: `headless_modal_scroll` (scrolleable), `picker_precedence` (picker recibe Esc antes que modal), `tty_guard` (non-TTY devuelve `Usage`), `raw_mode_guard_restores_on_panic`.

## Verification Contract

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test` (incl. `cargo test tui -- --nocapture`)
- `make e2e` (requiere Docker daemon; fail-closed)

## Definition of Done

- Global: 15 vectores verdes, 15 tabs render headless sin panic, `/tmp` audit manual pasa, `make e2e` verde.
- U1: 15 tabs visibles, upgrade sin selector, cada `[Enter]` spawnea vector válido.
- U2: picker no roba modal, modal scrolleable, atajos no colisionan, TTY guard + banner.

## Appendix

- Ideación: `docs/ideation/2026-08-26-tui-errores-y-paridad-funcional-ideation.md`
- Fix base: `fix/tui-e2e-zen` PR #250 (branch `fix/tui-e2e-zen`)
- E2E: `Dockerfile.e2e`, `e2e_runner.sh:20`, `tests/e2e.rs:13`
