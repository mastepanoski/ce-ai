---
module: transversal
tags: [strategy, factory, ports-adapters, tui, error-hardening, clippy]
problem_type: architecture
---

# Refactor Transversal de Mantenibilidad

## Problem
A medida que `ce-ai` creció para soportar 11 harnesses y 15 subcomandos CLI, se identificaron varios cuellos de botella arquitectónicos en el codebase (~16.5k líneas):
1. **Dispatch monolítico if-else/match:** `src/main.rs` y `src/commands/tools.rs` duplicaban bloques de coincidencia manual por cada harness para registro MCP (~40 líneas repetidas × 8).
2. **Monolito de TUI:** `src/tui.rs` concentraba 1.791 líneas de rendering, bucle de eventos, estado y builders de argumentos en un solo archivo sin separación de responsabilidades.
3. **Acoplamiento a I/O del Sistema de Archivos:** Las capas `state` y `opencode` realizaban operaciones de lectura y escritura directa sobre disco, obligando a usar `tempdir` en todos los tests de unidad e impidiendo testing hermético en memoria.
4. **Residuos de Unwrap en Producción:** Existían llamadas directas a `.unwrap()` y `.expect()` en código de producción que podían generar panics no controlados.

## Solution
Se ejecutó un refactor incremental guiado por el plan transversal en 6 unidades (U1 a U6):

1. **U1 — Command Registry Strategy (`src/commands/registry.rs`):**
   - Introducido el trait `CeCommand` y el enum centralizado `Commands` con `registry::dispatch(&ctx, cli.command)`.
   - `src/main.rs` reducido a ~35 líneas delegando completamente al registry.

2. **U2 — Harness Factory (`src/harness/mod.rs` & `src/commands/tools.rs`):**
   - Consolidado `HarnessKind::register_tool_mcp` factory y `mcp_spec_for_tool`.
   - Colapsados 7 bloques duplicados `if *_installed` en `tools::install_tool` a un único bucle sobre `state.installed_harnesses`.

3. **U3 — Descomposición Modular de TUI (`src/tui/`):**
   - El archivo `src/tui.rs` de 1.791 líneas se dividió en 6 submódulos con responsabilidades acotadas:
     - `app.rs`: Estado de la aplicación y workflow (`App`).
     - `handlers.rs`: Ejecución de comandos CLI y builders de texto.
     - `render.rs`: Widgets y layout con Ratatui (`ui`, modales, paneles).
     - `runner.rs`: Bucle de eventos Crossterm y guard `RawModeGuard`.
     - `spawn.rs`: Constructores puros de argumentos CLI (`status_args`, `doctor_args`, etc.).
     - `tabs.rs`: Definición de pestañas `MenuTab`.
   - Todos los archivos cumplen la métrica de tamaño (<500 líneas por módulo).

4. **U4 — Ports & Adapters para State y Opencode (`src/state/ports.rs`):**
   - Definidos los traits `StateStore` y `ConfigStore`.
   - Implementados adaptadores de producción basados en disco con escritura atómica (`FsStateStore`, `FsConfigStore`).
   - Implementados adaptadores en memoria thread-safe (`InMemoryStateStore`, `InMemoryConfigStore` con `RwLock`) permitiendo tests de unidad sin interacción con el filesystem.
   - Conectados métodos accesores en `Context` y funciones de mutación inyectables en `src/opencode/config.rs`.

5. **U5 — Endurecimiento de Errores y Cero Unwraps (`src/lib.rs`, `src/main.rs`):**
   - Auditados y eliminados todos los `.unwrap()` y `.expect()` en código de producción de `src/`, reemplazándolos con propagación `?` y mapeos tipados a `CeError`.
   - Añadido el lint `#![cfg_attr(not(test), deny(clippy::unwrap_used))]` a nivel de crate.

6. **U6 — Limpieza, Métricas y Verificación:**
   - Auditoría de `allow` annotations en todo `src/`.
   - Verificación del contrato anti-drift de TUI y gates de CI 100% verdes en Linux, macOS y Windows.

## Verification
- `cargo fmt --check`: Formato estricto cumplido.
- `cargo clippy --all-targets --all-features -- -D warnings`: 0 warnings.
- `cargo test`: 118 unit tests + 5 security tests pasando en verde.
- CI Matrix (PRs #256, #257, #258, #259): 100% verde en todos los jobs.

## Key Learnings
1. **Separación de Library vs Binary Crate:** Al estructurar `src/lib.rs` como crate de biblioteca y `src/main.rs` como un binario delgado consumidor de `ce_ai::`, se eliminan advertencias de dead code causadas por dobles raíces de compilación.
2. **Ports-Adapters en Rust:** El uso de `RwLock<HashMap<PathBuf, T>>` en adaptadores en memoria proporciona aislamiento hermético instantáneo para suites de pruebas sin penalización de I/O en disco.
3. **Control de Lints Condicionales:** `cfg_attr(not(test), deny(clippy::unwrap_used))` permite prohibir terminantemente panics en tiempo de ejecución en producción manteniendo la expresividad estándar de los tests unitarios.
