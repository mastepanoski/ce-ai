---
date: 2026-08-26
topic: tui-errores-y-paridad-funcional
focus: TUI dashboard — errores funcionales y paridad con CLI (cada función debe funcionar)
artifacts_root: docs
ideation_mode: repo-grounded
generated_by: ce-ideate
---

# TUI Errores y Paridad Funcional — Ideación Grounded

## Resumen Ejecutivo

El TUI (`src/tui.rs:1`, `src/main.rs:116`) ofrece 9 tabs pero el CLI expone 15 subcomandos (`ce-ai --help`). 6 comandos quedan huérfanos (skills, tools, usage, audit, init-prj, deinit-prj) y 3 tabs tienen semántica divergente (upgrade ignora harness, sync dry-run parcial, backup restore parsea mal harness). Los 8 tests de `tui.rs:1211` están verdes porque `every_tui_spawned_vector_satisfies_its_cli_contract:1359` solo pinnea 5 vectores — el net anti-drift no cubre los comandos añadidos post-1.23. Idear aquí significa cerrar paridad sin reescribir el TUI como un segundo CLI.

## Grounding Consolidado

### Codebase context
- **Shape:** `src/tui.rs:71` `App` guarda `selected_tab`, `selected_harness_idx`, `model_slots`, `output_modal`, `model_picker_open`; `run_app:247` dibuja `ui:432` y despacha `capture_cli:974` spawnando el binario actual como subprocess (evita corromper alternate screen, #72).
- **Patrones:** `MenuTab::all:40` 10 entries fijas; `render_content_panel:597` texto estático por tab; `reload_state:148` lee `HOME`, `state.json` y `crate::commands::models::config_assignments`; tests pinnean upgrade sin flags (`upgrade_dead_flags_stay_rejected:1398`) y vetores TUI válidos.
- **Pain points:** Tabs faltantes; `upgrade` aún renderiza `harnessTargets` aunque `run_upgrade_cmd:1145` lo ignora; `install/sync` comparten harness picker pero `upgrade` no; `capture_cli` mezcla stdout+stderr sin truncar ni scroll; `run_interactive:219` chequea `stdin.is_terminal()` pero TUI corre sobre `stdout`; `App::reload_state` hace `unwrap_or_default` silencioso; teclado colisiona (Up/Down mueve tab siempre, incluso sobre picker; `n/p` vs `j/k` duplicado).
- **Leverage:** Todo TUI ya delega a CLI vía `capture_cli` — paridad es añadir vector + tab + test, no lógica duplicada; `HarnessKind::detect_installed_harnesses` y `state.json` ya existen.

### Past learnings (`docs/solutions/`)
- `fix(tui): upgrade panel spawns current CLI contract; anti-drift net` — el precedente para pinnear vectores y evitar Issue #161 (upgrade --harness removido).
- `fix(determinism): pin release sources` — el TUI debe propagar errores con `CeError` y mostrar remedio, no `unwrap`.

### External context
- Ratatui + Crossterm: alternate screen + raw mode deben restaurarse en `Drop` incluso si `run_app` paniquea; 100ms poll es estándar; modal con `Clear` + `centered_rect` funciona pero sin scroll pierde contenido largo (doctor/backups).

---

## Topic axes (descomposición ortogonal)

1. **Navegación y entrada** — cómo el usuario llega a cada función sin memorizar atajos colisionados.
2. **Paridad de comandos** — qué subcomandos faltan y cómo exponerlos sin duplicar CLI.
3. **Seguridad/preview** — dry-run, confirmaciones y atomicidad visibles en TUI.
4. **Observabilidad y recuperación** — qué ve el usuario cuando algo falla y cómo vuelve.

---

## Candidatos Divergentes (15 ideas — generar todas antes de criticar)

### I1. Tabs faltantes como páginas espejo (skills/tools/usage/audit/init-prj)
Añadir 5 tabs que solo renderizan ayuda + capturan el subcomando (`skills list`, `tools status`, `usage report`, `audit`, `init-prj`). Sin duplicar lógica.

### I2. Paleta de comandos (`:`) en vez de más tabs
Un `:` abre fuzzy-palette que lista TODOS los subcomandos por nombre; no crece `MenuTab`.

### I3. Upgrade honesto sin selector de harness
Quitar el selector `< [harness] >` del panel Upgrade; el panel muestra "todos los harnesses activos" y veta selección.

### I4. Dry-run como toggle global con badge por comando
`d` alterna `app.dry_run`; cada panel que soporta `--dry-run` muestra `PREVIEW`/`APPLY` y el comando spawned lo incluye; los que no, muestran "n/a".

### I5. Scroll y búsqueda en modal de salida
Modal con viewport scrolleable (`j/k`, `PgUp/PgDn`, `/` buscar) en vez de `any key closes` + `Paragraph` sin scroll.

### I6. Picker de modelos con precedencia y stacking arreglado
Modal picker captura todas las teclas; `Esc` cancela, `Enter` aplica; el `output_modal` no roba input mientras picker esté abierto; `Up/Down` no propaga a tabs.

### I7. Matriz de paridad testeada: every_spawned_vector para TODOS los comandos
Extender `every_tui_spawned_vector_satisfies_its_cli_contract` a 15 vectores (skills adopt, tools install, usage sync, audit, backups, init-prj...), con `with_cli_globals` donde toca.

### I8. Raw-mode guard con RAII (Drop) y manejo de panic
`RawModeGuard` que restaura terminal en `Drop`; `run_interactive` lo usa en vez de `ok()` silencioso.

### I9. IsTerminal sobre stdout + non-TTY early-exit con mensaje accionable
Cambiar `stdin.is_terminal()` a `stdout.is_terminal()` y `stderr`, y devolver `Usage` con `ce-ai <subcommand> --help` si no hay TTY.

### I10. Harness picker con deshabilitados y conteo real
Si `detect_installed_harnesses` está vacío, mostrar `(none)` y deshabilitar `install --harness all`; `next_harness/prev_harness` salta deshabilitados.

### I11. Backup restore con confirmación y TisTerminal seguro
`run_restore_backup_cmd:1178` pide `y/N` dentro del modal, parsea `HarnessKind` con fallback + muestra `target_path`; no `unwrap_or(Opencode)` silencioso.

### I12. Estado degradado visible (state.json corrupto / HOME ausente)
`reload_state` propaga `Result` a un `banner` rojo en header en vez de `unwrap_or_default`; header muestra `config_dir: <path> (stale)` si `State::load` falla.

### I13. Atajos sin colisión: vim-keys solo en contexto
`j/k` solo navega listas cuando el foco es lista; tabs se mueven solo con `Up/Down` o `Tab`; `n/p` queda reservado a Models y no se intercepta globalmente.

### I14. Footer dinámico por tab
Footer muestra atajos relevantes al tab actual (ej. Models: `[m] pick · [n/p] slot`, Backups: `[r] restore · [◄/►] filter`).

### I15. E2E TUI headless con `cargo test --test tui` + snapshot de frames
Test que instancia `App`, llama `ui` con `TestBackend` y snapshottea `Buffer` por tab (golden), detecta overflow y panic de layout.

---

## Crítica — cada idea pasa por el filtro

| # | Veredicto | Razón técnica / evidencia |
|---|-----------|---------------------------|
| I1 | **KEEP** | Repo-grounded: 6 comandos huérfanos probados con `ce-ai --help`; el patrón `capture_cli` ya existe, costo ~30 LOC por tab + test. Riesgo bajo. |
| I2 | REJECT | Reemplaza tabs por paradigma nuevo; contradice aprendizaje de TUI actual (sidebar fijo) y añade fuzzy engine sin necesidad inmediata; útil como I1-followup, no como fix de errores. |
| I3 | **KEEP** | Bug real: `tui.rs:760` renderiza selector pero `upgrade_cmd_args:953` lo ignora; confunde al junior que cree estar haciendo upgrade parcial. Fix <10 LOC. |
| I4 | **KEEP** | `App.dry_run:72` ya existe pero `run_upgrade_cmd` y `run_doctor_cmd` lo ignoran; el test `with_cli_globals` cubre `--dry-run` solo para install/sync. Paridad de preview exige badge. |
| I5 | **KEEP** | Dolor observado: `render_modal:870` trunca con `Wrap` y cierra con cualquier tecla; doctor/backups con muchas líneas pierden info. Ratatui `List` + `Scrollbar` es estándar. |
| I6 | **KEEP** | Bug de input: `run_app:263` `if output_modal.is_some()` roba cualquier tecla antes de picker; `270` picker maneja `Up/Down` pero `349` tabs también los manejan si picker cerrado mal; stacking roto reportado en QA manual. |
| I7 | **KEEP** | El net actual miente: 8/8 verde pero 10 vectores sin pinnear. Extenderlo es la forma más barata de garantizar que cada función "funcione" tras cada cambio de CLI (ya evitó #161). |
| I8 | KEEP (defer) | Correcto pero no arregla ningún tab roto hoy; el `ok()` silencioso en `disable_raw_mode` ya es tolerable. Prioridad menor que paridad. |
| I9 | **KEEP** | `run_interactive:220` chequea stdin pero TUI dibuja en stdout; en `cargo test` y pipes el guard falla o no falla cuando debe. Fix trivial, evita "error: raw mode" críptico. |
| I10 | REJECT | `HarnessKind::detect_installed_harnesses` casi siempre devuelve algo (al menos opencode); el caso vacío ya muestra `(No host agent harnesses…)` en Status. Over-engineering. |
| I11 | KEEP (merge con I1) | Útil pero pequeño; se absorbe en I1 Backups + confirm modal. No merece idea separada. |
| I12 | **KEEP** | `reload_state:148` silencia errores; si `state.json` está corrupto el dashboard sigue mostrando "0 Active" sin explicar por qué. Banner degrado es 15 LOC. |
| I13 | **KEEP** | Colisión real: `k` sube tab pero también sube picker; `n` en Models colisiona con navegación global. Context-gating es fix de usabilidad de 20 LOC. |
| I14 | KEEP (defer) | Nice-to-have que depende de I13; sin I13 el footer dinámico mentiría. |
| I15 | **KEEP** | Sin snapshots, cada nuevo tab puede romper layout (`Constraint::Percentage` overflow) y nadie lo nota hasta manual. `TestBackend` existe en ratatui, costo medio pero paga. |

**Sobrevivientes:** I1, I3, I4, I5, I6, I7, I9, I12, I13, I15 (10 → ranking prioriza 6 para v1, 4 defer).

---

## Ranking Final — qué construir primero

### 🥇 1. I7 — Matriz de paridad testeada (every_spawned_vector completo)
**Por qué primero:** sin esto, cualquier fix de paridad puede re-romperse en el siguiente PR y los 8 tests seguirán verdes. Es el candado que hace que "cada función funcione" sea verificable en CI. ~40 LOC, cero UI, unblocka I1/I3/I4.

### 🥈 2. I1 — Tabs faltantes espejo (skills/tools/usage/audit/init-prj) + I3 upgrade honesto
**Juntos:** cierran el gap 9→15 comandos. Cada tab es `render_*` + `*_cmd_args` + `capture_cli`. I3 va pegado porque Upgrade es el único tab que miente sobre harness. Estimado ~200 LOC total (CONTRIBUTING §4 work-unit).

### 🥉 3. I6 — Picker con stacking y precedencia fija
**Bug funcional crítico:** hoy puedes dejar un modal abierto y el picker no recibe `Esc`; o cerrar modal sin querer y perder selección. Afecta a la función "Models & Profiles" que es la más usada tras install. ~25 LOC.

### 4. I5 — Modal scrolleable y buscable
**Sin esto, doctor/backups/skills list son inútiles con datos reales** (más de 70% de viewport). Es el fix de observabilidad que hace que las otras tabs sirvan. ~50 LOC con `List` + `Scrollbar`.

### 5. I13 — Atajos sin colisión (context-gated)
**Costo de confusión para juniors:** `j/k` hace dos cosas a la vez. Este fix baja la tasa de "apreté algo y el dashboard saltó de tab". ~20 LOC, sin riesgo.

### 6. I9 + I12 — TTY guard honesto y banner de estado degradado
**Paquete de resiliencia:** evita el falso `Usage` en CI y explica por qué el header dice "0 Active" cuando `state.json` está roto. ~25 LOC.

**Defer a v2:** I8 RAII guard, I14 footer dinámico, I15 snapshots E2E (aunque I15 debería entrar en cuanto v1 estabilice, para que v2 no rompa layout).

---

## Métrica de éxito

- `cargo test tui` pasa con 15 vectores pinneados (I7) y `make e2e` sigue verde.
- Manual QA headless: recorrer 15 tabs con `TestBackend`, cada `[Enter]` devuelve `exit 0` o modal con `❌` + remedio, y `doctor` trunca con scroll en vez de cortar.
- `ce-ai --help` y `MenuTab::all().len()` difieren solo en `Exit` (único tab sin comando).

---

## Artefacto y próximos pasos

- **Artefacto:** este doc `docs/ideation/2026-08-26-tui-errores-y-paridad-funcional-ideation.md` (output:md).
- **Next:** `/ce-brainstorm tui-errores-y-paridad-funcional` tomando **I7 + I1/I3** como dirección elegida. Ce-brainstorm refinará requisitos (R1..R6), flujos y acceptance (AE1..AE4) antes de `/ce-plan`.
- **Cost line:** v1 (I7+I1/I3+I6+I5+I13+I9/I12) ~360 LOC → 2 PRs encadenados (~200 LOC c/u) por CONTRIBUTING §4; cada PR con `fmt`/`clippy -D warnings`/`cargo test`/`make e2e`.

