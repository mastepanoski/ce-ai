# Proposal: tui-e2e-zen — TUI headless E2E con Opencode Zen free en Docker

## Why
El TUI pasa 8/8 tests pero `every_tui_spawned_vector_satisfies_its_cli_contract` (src/tui.rs:1359) solo pinnea 5/15 vectores; los 6 comandos huérfanos (skills/tools/usage/audit/init-prj) nunca se ejecutan en CI y `Dockerfile.e2e` no prueba TUI. Un junior puede romper paridad y CI sigue verde. Se requiere gate fail-closed que pruebe cada función TUI headless con modelo free, aislado en Docker, antes del brainstorm de paridad completa.

## What Changes
- Extiende `every_tui_spawned_vector_satisfies_its_cli_contract` a 15 vectores (install, sync, upgrade, models list/set, skills list/resolve/doctor/adopt, status, uninstall, doctor, backups list, tools status, usage report, workflow status, audit, init-prj).
- Añade `TestBackend` snapshots headless para `ui()` por tab (no requiere TTY) — detecta overflow/panic de layout.
- Extiende `Dockerfile.e2e` + `e2e_runner.sh` con paso TUI: `opencode` zen free headless (`opencode run --model opencode/zen-free` o mock si no hay creds), `ce-ai skills resolve` headless, y `cargo test tui` dentro del contenedor.
- Mantiene `make e2e` fail-closed: Docker ausente = hard failure.

## Scope
- In-scope: solo harness E2E/TUI, no tabs nuevas (eso va en brainstorm posterior).
- Out: tabs faltantes (I1), picker/modal fixes (I5/I6) — quedan para `tui-errores-y-paridad-funcional`.

## Risks
- Zen creds ausentes en CI → fallback a mock model list; E2E no flakyea por red.
- `TestBackend` añade dep `ratatui --features test` solo en `dev-dependencies` — no bin bloat.

## Success Criteria
- `cargo test` pinnea 15 vectores; `cargo test tui` con TestBackend pasa.
- `make e2e` en host con Docker pasa incluyendo paso TUI headless.
