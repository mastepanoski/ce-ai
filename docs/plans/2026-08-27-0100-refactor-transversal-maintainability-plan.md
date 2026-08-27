---
title: Refactor Transversal Mantenibilidad - Plan
type: refactor
date: 2026-08-27
topic: refactor-transversal-maintainability
artifact_contract: ce-unified-plan/v1
artifact_readiness: implementation-ready
product_contract_source: ce-brainstorm
execution: code
---

# Refactor Transversal Mantenibilidad - Plan

## Goal Capsule

**Objective:** `ce-ai` gana mantenibilidad medible sin romper compatibilidad — el código deja de penalizar cambios con if-else anidados, módulos monolíticos y duplicación entre harnesses; añadir un comando o harness nuevo es incremental y testeable.

**Means:** Refactor híbrido incremental con patterns objetivo diseñados upfront pero migrados slice a slice, empezando por Commands Strategy (KTD1).

**Authority:** Product Contract de este plan > Planning Contract > Implementation Units. Compatibilidad 100% CLI/state/opencode es invariante (R1).

**Stop conditions:** Todos los U completados con Verification Contract verde; ningún breaking change en CLI binario, `state.json`/`opencode.json`, ni `scripts/install.sh|ps1`.

**Execution profile:** `code` — Rust 2021, `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `make e2e` en contenedor. Cambios atómicos vía `crate::state::write_atomic` para `state.json`/`opencode.json`.

## Product Contract

### Summary

Refactor transversal del repo `ce-ai` (~16.5k líneas, 35+ archivos) con compatibilidad 100%: eliminar cadenas if-else en commands vía Strategy/Command, descomponer `tui.rs` monolítico (1.791 líneas), unificar harnesses duplicados bajo Factory/Trait y aislar boundaries `state`/`opencode`/`source` con Ports-Adapters — entregado de forma incremental empezando por el slice de Commands.

### Problem Frame

El brainstorm transversal identificó tres dolores concretos verificados contra código: `src/main.rs:100-116` y `src/commands/tools.rs:28-323` usan `match`/`if-else` repetidos por harness (7 harnesses con registro MCP copiado ~40 líneas x 8), `src/tui.rs` concentra rendering, estado y eventos en un solo archivo de 1.791 líneas sin separación container/presentational, y `src/state`, `src/opencode`, `src/source` mezclan I/O con lógica sin ports testeables. Clippy está verde hoy, pero el costo de añadir un harness/comando es alto y los tests de drift CLI (`tui.rs:1523 every_tui_spawned_vector…`) son el único guard contra regresiones de contrato. Sin refactor, cada feature nueva amplifica duplicación y el onboarding requiere entender el archivo monolítico.

### Requirements

**Compatibilidad y contrato**

- R1. Compatibilidad 100% preservada — ningún cambio rompe CLI binario (`ce-ai --help`), exit codes de `CeError` (0/1/2/3/4/5/6), schema `state.json`, `opencode.json` (preservación de plugins/skills no gestionados) ni `scripts/install.sh`/`scripts/install.ps1`.
- R2. `crate::state::write_atomic` permanece obligatorio para toda mutación de `state.json`/`opencode.json` (hard-gate invariante 3).

**Maintainability target**

- R3. Eliminar cadenas if-else de dispatch en `src/main.rs` y `src/commands/*` sustituyéndolas por registry Strategy/Command donde el añadir un comando no exige editar un `match` central.
- R4. Eliminar duplicación inter-harness — el registro MCP por-harness en `src/commands/tools.rs:128-315` y similares debe colapsar a una ruta común parametrizada por `HarnessKind`, sin 7 bloques copiados.
- R5. Descomponer `src/tui.rs` monolítico en módulos con boundary claro (p.ej. `tui/render`, `tui/state`, `tui/events`, `tui/tabs`) sin cambiar comportamiento visible.
- R6. Aislar `state`/`opencode`/`source` con boundaries testeables (trait/port + adapter) de modo que lógica de negocio sea testeable sin filesystem real.

**Calidad y patrones**

- R7. Patrones aplicados donde aportan mantenibilidad — Strategy/Command para dispatch, Factory para harnesses, Ports-Adapters para state/opencode; no introducir Chain of Responsibility u otros patrones sin justificar carrying cost vs beneficio.
- R8. YAGNI estricto — ningún patrón, crate split o abstracción sin caso de uso concreto y un test que la ejercite.

**Verificación**

- R9. CI 100% verde es la señal primaria de éxito — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test` (unit + `tests/e2e.rs` + `tests/security.rs`), `make e2e` y matriz GitHub Actions (ubuntu/macos/windows) sin regresión.
- R10. Preservar y ampliar el anti-drift net `tui.rs:every_tui_spawned_vector_satisfies_its_cli_contract` — todo vector spawneado por TUI debe seguir parseando contra su superficie clap viva.

**Documentación**

- R11. Capturar aprendizajes no triviales en `docs/solutions/` con frontmatter (`module`, `tags`, `problem_type`) siguiendo el patrón existente; no dejar refactor sin rastro de decisión.

### Key Decisions

- **D1. Híbrido incremental sobre big-bang** — diseñar traits/patterns objetivo upfront pero migrar slice a slice empezando por Commands Strategy (slice mínimo valioso elegido) — evita ventana larga de código medio-refactorizado sin entrega. Governs R3, R9.
- **D2. Compatibilidad 100% como invariante no negociable** (session-settled: user-directed — chosen over migración con breaking changes: permite limpieza profunda inmediata pero rompe usuarios). Governs R1, R10.
- **D3. CI verde como señal primaria de éxito sobre métricas de líneas** — cyclomatic/duplicación son secundarias; si CI no está verde el refactor no está hecho. Governs R9.
- **D4. Patrones solo donde reducen duplicación medible** — Strategy/Factory/Ports-Adapters sí; Chain of Responsibility y split en crates separados deferred hasta que un caso lo exija. Governs R7, R8.

### Scope Boundaries

**En scope:**
- Refactor interno de `src/main.rs`, `src/commands/*`, `src/harness/*`, `src/state/*`, `src/opencode/*`, `src/source/*`, `src/tui.rs` manteniendo contrato externo.
- Unificación de duplicación verificada (`src/commands/tools.rs` por-harness blocks, `src/harness/*.rs` adapters).
- Tests y harnesses de verificación para los nuevos boundaries.

**Fuera de scope (no hacer):**
- Nueva funcionalidad de producto o cambios visibles de CLI/flags.
- Breaking changes de schema, exit codes o comportamiento de `install.sh`/`install.ps1`.
- Split en crates separadas (`ce-core`/`ce-harness`/`ce-tui`) — deferred.
- Reescritura de `docs/` más allá de `docs/solutions` del refactor.

### Success Criteria

- SC1. `cargo test` + `make e2e` + CI matrix 3 OS verde sin flaky nuevo; `cargo clippy -D warnings` cero warnings tras cada U.
- SC2. 0 duplicaciones estructurales >30 líneas entre harnesses detectables por inspección (`tools.rs` colapso verificable).
- SC3. `tui.rs` deja de ser monolito — ningún archivo en `src/tui/**` excede ~500 líneas y boundaries render/state/events son testeables en isolation.
- SC4. Añadir un comando dummy de prueba requiere tocar solo registry + su módulo, sin editar `match` central (demostrable en PR de verificación).

### Actors

- Mantenedor humano y agentes AI que extienden `ce-ai` — ambos consumen los mismos boundaries y tests; no hay actor multi-party diferenciado.

## Planning Contract

### Key Technical Decisions

- KTD1. **Command Registry con trait `CeCommand`** — `src/commands/mod.rs` expone `trait CeCommand { fn run(ctx, args) }` + `CommandRegistry` que registra subcomandos vía `clap::Command` augmentation; `src/main.rs:100-116` delega a `registry.dispatch(cli.command)` en lugar de `match` manual. (session-settled: user-directed — chosen over match central extendido: cada comando nuevo hoy exige editar el match y duplicar boilerplate).
- KTD2. **Harness Factory + `HarnessAdapter` trait unificado** — `src/harness/mod.rs` consolida `HarnessKind::adapter()` factory que retorna `Box<dyn HarnessAdapter>`; `src/commands/tools.rs` colapsa 7 bloques `if *_installed` a un loop `for harness in state.installed_harnesses { adapter.register_mcp(...) }`. Preserva `home_dir_from_ctx` y `register_*_mcp_server` como adapters.
- KTD3. **TUI descomposición por responsabilidad** — `src/tui.rs` (1.791 líneas) se parte en `src/tui/mod.rs` (orquestación), `src/tui/render.rs`, `src/tui/state.rs`, `src/tui/events.rs`, `src/tui/tabs/*.rs`; mantiene `ratatui`/`crossterm` y el test anti-drift `every_tui_spawned_vector…` intacto y ampliado. Sin cambio visual.
- KTD4. **Ports-Adapters para `state`/`opencode`/`source`** — `src/state/mod.rs` y `src/opencode/config.rs` exponen traits `StateStore`/`ConfigStore` con `FsStateStore`/`FsConfigStore` (usa `write_atomic`) e `InMemoryStore` para tests; `src/source/registry.rs` y `src/commands/*` dependen del port, no de `PathBuf` directo.
- KTD5. **Consolidación de error y dry-run** — `src/error.rs` `CeError` como única fuente de exit codes; `Context { dry_run, quiet, verbose }` threading uniforme; ningún `unwrap()` en paths de producción fuera de tests — clippy `unwrap_used` deny en `src/**` salvo `#[cfg(test)]`.
- KTD6. **No Chain of Responsibility ni crate split en este plan** — el dispatch es 1:1 comando→handler, no cadena; crate split requiere RFC separado. Evita sobre-ingeniería (YAGNI, R8).

### High-Level Technical Design

```
src/main.rs (thin) -> CommandRegistry (KTD1) -> CeCommand impls (src/commands/*)
                |
                +-> HarnessFactory (KTD2) -> dyn HarnessAdapter (src/harness/*)
                +-> TuiApp (KTD3) -> render/state/events/tabs
                +-> StateStore / ConfigStore ports (KTD4) -> Fs adapters (write_atomic)

Verification spine: clippy + cargo test + anti-drift test (KTD1/KTD3) + make e2e
```

Secuencia incremental: U1 (registry) desbloquea U2 (harness loop consume registry context) y U3 (TUI consume CeCommand vectors vía registry); U4 (ports) puede ir en paralelo a U3 tras U1; U5 cross-cutting cierra; U6 verificación final.

### Assumptions

- A1. `cargo clippy --all-targets --all-features -- -D warnings` ya verde (verificado 2026-08-27) — refactor no parte de deuda clippy.
- A2. Matriz CI 3 OS y `make e2e` disponibles en repo — no se añade runner nuevo.
- A3. `tui.rs:1523` anti-drift test es contract test suficiente para CLI surface — no se añade golden-file extra en esta fase.

### Sequencing

- Fase 1: U1 Command Registry (desbloquea resto)
- Fase 2: U2 Harness Factory || U3 TUI descomposición (paralelizables tras U1)
- Fase 3: U4 Ports-Adapters (paralelizable con Fase 2 si U1 hecho)
- Fase 4: U5 Error/dry-run cross-cutting (tras U1-U4)
- Fase 5: U6 Cleanup + Solutions + gates finales (cierra)

### Risks & Dependencies

- Riesgo: sobre-patrón (Strategy god object) — mitigación: KTD1 registry es solo dispatch, sin lógica de negocio; review límite 400 líneas por PR (CONTRIBUTING).
- Riesgo: TUI regresión visual — mitigación: anti-drift test + snapshot manual antes/después.
- Dependencia: `crate::state::write_atomic` y `PathBuf::join` cross-platform deben permanecer invariantes.

## Implementation Units

### U1. Command Registry Strategy — eliminar match central

**Goal:** Dispatch de comandos vía registry sin `match` manual en `main.rs`.

**Requirements:** R1, R3, R9, R10.

**Files:** `src/main.rs`, `src/commands/mod.rs`, `src/commands/install.rs`, `src/commands/sync.rs` (ejemplo de migración de 2-3 comandos), `src/tui.rs` (vectores spawn).

**Approach:**
- Definir `trait CeCommand` + `struct CommandRegistry { map: BTreeMap<&'static str, Box<dyn CeCommand>> }` en `src/commands/mod.rs`.
- Migrar `Commands` enum a registro dinámico o mantener enum pero delegar ejecución a `registry.get(name).run(ctx, args)` — elegir en implementación la forma que preserve `clap` derive sin duplicar help text (evaluar `clap::Subcommand` vs manual registration; KTD1).
- `src/main.rs:100-116` queda en ~15 líneas: parse + `registry.dispatch(cli.command, &ctx)`.
- Migrar primero `install`, `sync`, `status` como pilotos; resto queda para iterar dentro de U1 sin nueva U.
- Preservar `tui.rs` vectores — anti-drift test debe seguir verde.

**Test Scenarios:**
- TS1.1: `ce-ai --help` y `ce-ai install --help` idénticos antes/después (snapshot help text).
- TS1.2: `every_tui_spawned_vector_satisfies_its_cli_contract` verde.
- TS1.3: Añadir comando dummy `__probe` vía registry sin tocar `main.rs` — test unit lo registra y `dispatch` lo resuelve.
- TS1.4: `cargo test` + `cargo clippy -D warnings` verde.

**Verification:** `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`; `ce-ai install --dry-run` y `ce-ai sync --dry-run` sin regresión.

### U2. Harness Factory — colapsar duplicación por-harness

**Goal:** Un único loop parametrizado reemplaza 7 bloques copiados de registro MCP.

**Requirements:** R1, R4, R7, R8.

**Files:** `src/harness/mod.rs`, `src/harness/claude.rs`, `src/harness/cursor.rs`, `src/harness/codex.rs`, `src/harness/copilot.rs`, `src/harness/kimi.rs`, `src/harness/grok.rs`, `src/harness/agy.rs`, `src/harness/fx.rs`, `src/commands/tools.rs`, `src/harness/registration.rs`.

**Approach:**
- Extender `HarnessKind` con `fn adapter(&self) -> Box<dyn HarnessAdapter>` y `trait HarnessAdapter { fn default_config_path, fn register_mcp_server, fn is_installed }`.
- `src/commands/tools.rs:128-322` colapsa a `for entry in &state.installed_harnesses { let adapter = HarnessKind::from_name(&entry.name).adapter(); adapter.register_mcp(...) }` — elimina 7 `if *_installed` blocks.
- Mantener `install_tool` match de `tool_lower` (context7/engram/rtk/codegraph) — es dominio pequeño y legible, no forzar Strategy ahí.
- Verificar que `pi` sigue con mensaje `does not support native MCP` sin registro.

**Test Scenarios:**
- TS2.1: `cargo test --test security` verde (no path traversal regresión).
- TS2.2: Dry-run `ce-ai tools install codegraph` produce 1 log por harness instalado vs 7 bloques previos.
- TS2.3: Añadir harness ficticio `test-harness` requiere solo nuevo `HarnessKind` + adapter, sin editar `tools.rs` dispatch.

**Verification:** `cargo test && cargo clippy`; `ce-ai tools status` output idéntico salvo orden determinista.

### U3. TUI Descomposición — romper monolito 1.791 líneas

**Goal:** `tui.rs` deja de ser monolito sin cambio visual ni de contrato CLI.

**Requirements:** R1, R5, R9, R10.

**Files:** `src/tui.rs` → `src/tui/mod.rs`, `src/tui/render.rs`, `src/tui/state.rs`, `src/tui/events.rs`, `src/tui/tabs/*`, `src/main.rs` (llamada `tui::run_interactive`).

**Approach:**
- Extraer módulos por responsabilidad: `state.rs` (WorkflowState, AppState), `events.rs` (crossterm event loop), `render.rs` (ratatui widgets), `tabs/*.rs` (cada tab).
- `src/tui/mod.rs` mantiene `pub fn run_interactive(ctx: &Context) -> Result` como entry point.
- Mover helpers `install_cmd_args`, `sync_cmd_args`, etc. a `src/tui/tabs/*.rs` junto a su tab.
- Preservar `every_tui_spawned_vector…` moviéndolo a `src/tui/mod.rs` o `tests/tui_contract.rs`.

**Test Scenarios:**
- TS3.1: `cargo test every_tui_spawned_vector_satisfies_its_cli_contract` verde tras move.
- TS3.2: Ningún archivo en `src/tui/**` >500 líneas (`wc -l` check).
- TS3.3: Manual smoke: `ce-ai` TUI abre, navega tabs, `q` sale sin panic (CI no rompe).

**Verification:** `cargo fmt && cargo clippy && cargo test`; `make e2e` incluye `ce-ai status` tras TUI path (no regresión).

### U4. Ports-Adapters para State/Opencode/Source

**Goal:** Lógica testeable sin filesystem real; I/O aislado en adapters.

**Requirements:** R1, R2, R6, R9.

**Files:** `src/state/mod.rs`, `src/state/state.rs`, `src/opencode/config.rs`, `src/source/registry.rs`, `src/commands/sync.rs`, `src/commands/install.rs`, `tests/**`, `src/state/backups.rs`.

**Approach:**
- Traits `StateStore { load, save_atomic }`, `ConfigStore { read, write_atomic }` en `src/state/mod.rs` / `src/opencode/config.rs`.
- `FsStateStore`/`FsConfigStore` usan `write_atomic` + `PathBuf::join`; `InMemoryStore` para tests.
- Inyectar `&dyn StateStore` en `sync`, `install`, `registry::verify` — constructores toman `Context` que ya lleva `config_dir`/`opencode_config_dir`, añadir `ctx.state_store()` accessor que por defecto retorna `FsStateStore`.
- No cambiar schema `state.json` — solo indirection.

**Test Scenarios:**
- TS4.1: `sync` y `install` tests usan `InMemoryStore` sin `tempdir` — al menos 2 tests nuevos por comando migrado.
- TS4.2: `write_atomic` zero-residue test (`tests/security.rs:atomic_write_guarantees…`) sigue verde con `InMemoryStore` + `FsStore`.
- TS4.3: Workspace override `.ce-ai.json` merge sigue funcionando con port.

**Verification:** `cargo test && cargo clippy`; `ce-ai doctor` con state corrupto retorna `CeError::State` (exit 3) sin panic.

### U5. Cross-cutting Error, Dry-run y Unwrap Hardening

**Goal:** `CeError` uniforme, `dry_run` threading consistente, cero `unwrap` en prod.

**Requirements:** R1, R2, R9.

**Files:** `src/error.rs`, `src/commands/mod.rs` (`Context`), `src/source/cache.rs`, `src/source/registry.rs`, `src/harness/*.rs`, `src/state/*.rs`.

**Approach:**
- Auditoría `grep -rn "\.unwrap()\|\.expect(" src --include="*.rs" | grep -v "#[cfg(test)]"` — reemplazar por `?` con `CeError::Io|State|Network` mapeado.
- `clippy::unwrap_used` deny en `src/**` (allow en tests) — añadir a `Cargo.toml` lints o `clippy.toml`.
- Unificar `dry_run` guard: todo `if ctx.dry_run { return Ok(()) }` antes de cualquier `write_atomic`/`fs::write` — verificar con grep que no hay write sin guard.
- `result_exit_code` mapping ya existe — preservar.

**Test Scenarios:**
- TS5.1: `cargo clippy -- -D clippy::unwrap_used` pasa en `src/**`.
- TS5.2: `ce-ai install --dry-run --harness claude` no escribe `state.json` ni `opencode.json` (hash antes/después igual).
- TS5.3: Error de red en `source::github::fetch` mapea a exit 5, no panic.

**Verification:** `cargo clippy -- -D warnings -D clippy::unwrap_used && cargo test`.

### U6. Cleanup, Métricas y Knowledge Capture

**Goal:** Cierre verificable del refactor con métricas y `docs/solutions` trail.

**Requirements:** R8, R9, R11, SC1-SC4.

**Files:** `docs/solutions/**`, `README.md` (si aplica), `Cargo.toml` (version bump si corresponde), `CHANGELOG.md`, todos los `src/**` para dead_code audit.

**Approach:**
- `cargo fix --allow-dirty` + `cargo clippy` + `cargo audit` si disponible; `cargo test -- --include-ignored` no requerido (e2e vía make).
- `grep -rn "allow(dead_code\|allow(unused" src` justificar o eliminar.
- Métricas: `cargo test` tiempo antes/después, `wc -l src/tui/**`, duplicación `jscpd` o `rg` count opcional — registrar en `docs/solutions`.
- Escribir `docs/solutions/refactor-transversal-maintainability.md` con frontmatter `module: refactor`, `tags: [strategy, factory, ports-adapters, tui]`, `problem_type: architecture`.
- Verificar SC4 con PR de prueba `__probe` comando (revert antes de merge).

**Test Scenarios:**
- TS6.1: `cargo fmt --check && cargo clippy -D warnings && cargo test && make e2e` verde en local y CI.
- TS6.2: `docs/solutions/refactor-transversal-maintainability.md` existe y frontmatter válido.
- TS6.3: SC4 smoke: registrar `__probe` sin tocar `main.rs` demostrado en test U1.

**Verification:** Full gate `make e2e` + CI matrix.

## Verification Contract

**Comandos repo-específicos:**
- `cargo fmt --check` — formato.
- `cargo clippy --all-targets --all-features -- -D warnings` + `-D clippy::unwrap_used` tras U5.
- `cargo test` — unit + `tests/security.rs` (4 tests) + `tests/e2e.rs` (ignorado salvo `-- --ignored`); incluye `every_tui_spawned_vector…`.
- `make e2e` — e2e Docker gate (estado limpio).
- CI matrix: `ubuntu-latest`, `macos-latest`, `windows-latest` + `install.ps1` en `windows-latest`.

**Gates por U:** cada U exige `cargo test` + `cargo clippy` verde antes de siguiente U. U6 exige `make e2e` completo.

## Definition of Done

**Global:**
- [ ] R1-R11 satisfechos sin breaking CLI/state/opencode.
- [ ] `cargo fmt --check` + `cargo clippy -D warnings` cero warnings.
- [ ] `cargo test` verde y `make e2e` verde.
- [ ] CI matrix 3 OS verde.
- [ ] `docs/solutions/refactor-transversal-maintainability.md` creado.
- [ ] Código de intentos abandonados eliminado (no dead branches).
- [ ] SemVer bump y `CHANGELOG.md` si el refactor se shippea como minor/patch según impacto.

**Por U:** cada U marca su Verification + Test Scenarios verdes y review <400 líneas por PR.

## Appendix

**Fuentes:**
- `src/main.rs:100-116` dispatch match — origen if-else a colapsar.
- `src/commands/tools.rs:28-323` duplicación por-harness — evidencia SC2.
- `src/tui.rs:1-1791` monolito — métrica SC3.
- `src/state/state.rs` + `src/state/mod.rs:write_atomic` — invariante R2.
- `src/error.rs` CeError exit codes — invariante R1.
- `CONCEPTS.md` vocabulario Harness/Managed Assets — autoridad producto.
- `docs/solutions/*` — patrón previo para capture (ej. `skill-registry-code-review-refactorings…`).

**Estimación líneas por U:** U1 ~180, U2 ~220, U3 ~300 (movimiento + split), U4 ~250, U5 ~150, U6 ~80. Total ~1.180 LOC cambiadas, respetando PRs <400 líneas vía 4-5 PRs encadenados.
