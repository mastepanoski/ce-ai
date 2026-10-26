# Requirements: Terminal Stream Token Compressor (`ce-ai run --compact`)

**Date:** 2026-10-25  
**Topic:** terminal-stream-token-compressor  
**Origin:** Extracción de valor de `unclebob/ub-mode` (Principio "Guard the Context Window") vía `docs/ideation/2026-10-25-unclebob-ub-mode-value-extraction-ideation.html`  
**Harness / CLI Scope:** `ce-ai` Rust CLI  

---

## 1. Problem Statement

Durante las fases de implementación y verificación de Compound Engineering (Stage 4 `ce-work` y Stage 5 `verification`), los agentes de IA ejecutan repetidamente comandos de prueba y compilación como `cargo test`, `npm test`, `vitest` o `pytest`.

Estos comandos producen rutinariamente entre 100 y 800 líneas de salida estándar (logs de compilación, listados de pruebas pasadas una a una, advertencias secundarias y metadatos de entorno). En cada ejecución exitosa, esto inyecta de 2,000 a 6,000 tokens innecesarios directamente en la ventana de contexto del agente. En sesiones con 5 a 10 iteraciones de prueba, entre el 40% y el 70% del consumo de tokens y del presupuesto de contexto se desperdicia en confirmar lo obvio: que los tests pasaron.

Por otro lado, cuando un comando falla, el error crítico suele quedar sepultado en medio de cientos de líneas de salida irrelevante, forzando al modelo a re-leer un buffer saturado para aislar el fallo.

---

## 2. Proposed Solution: `ce-ai run --compact`

Crear un subcomando nativo en `ce-ai` (`ce-ai run --compact <comando...>` o flag abreviado `ce-ai run -c <comando...>`) que actúe como un ejecutor y buffer de protección de contexto:

1. **Captura y Redirección Total a Disco**: Ejecuta el comando en un subproceso, volcando el stream completo (stdout + stderr) en tiempo real a un archivo rotativo en disco.
2. **Generación de Tarjeta de Estado Sintética en Éxito (Exit Code 0)**: Emite en consola exactamente 3 líneas concisas con el recuento de tests, tiempo de ejecución y enlace al log.
3. **Slicing Quirúrgico de Errores en Fallo (Exit Code != 0)**: Aísla y muestra únicamente el bloque de error relevante (aserciones fallidas, pánicos de Rust, trazas de excepción de Python/Node) y omite los tests que pasaron y los logs de compilación.
4. **Cero Dependencias Externas**: Implementado nativamente en Rust como parte del binario de `ce-ai`.

---

## 3. User & Agent Journey

### 3.1 Escenario Exitoso (Compresión Radical)

**Comando ejecutado por el agente:**
```bash
ce-ai run --compact cargo test
```

**Salida entregada al contexto del agente (Solo 3 líneas ~50 tokens en vez de 3,000):**
```text
✓ Command succeeded: cargo test (124 tests passed, 0 failed) in 1.42s
💾 Full log (312 lines): target/ce-logs/run-20261025-214012.log (symlink: target/ce-logs/latest.log)
⚡ Token compression: ~2,800 tokens saved (96.5% reduction)
```

### 3.2 Escenario de Fallo (Slicing Inteligente de Error)

**Comando ejecutado por el agente:**
```bash
ce-ai run --compact cargo test --test unit_tests
```

**Salida entregada al contexto del agente (~180 tokens enfocados):**
```text
🛑 Command failed (exit code 101): cargo test --test unit_tests in 0.86s
--------------------------------------------------------------------------------
FAILURES:
---- test_atomic_write_preserves_permissions stdout ----
thread 'test_atomic_write_preserves_permissions' panicked at src/state/mod.rs:142:9:
assertion `left == right` failed: expected mode 0o644, got 0o600
  left: 420
 right: 384
stack backtrace:
   0: ce_ai::state::write_atomic
   1: ce_ai::state::tests::test_atomic_write_preserves_permissions
--------------------------------------------------------------------------------
💾 Full execution log (189 lines): target/ce-logs/latest.log
```

---

## 4. Scope Boundaries

### In-Scope (MVP)
- **Subcomando CLI**: `ce-ai run [--compact|-c] [--raw] <command> [args...]`.
- **Estructura de Logs y Rotación**:
  - Directorio preferido en repos Cargo: `target/ce-logs/`.
  - Directorio en otros proyectos: `.ce-ai/logs/` (con entrada automática en `.gitignore`).
  - Conservación de los últimos 10 logs con nombrado por timestamp (`run-YYYYMMDD-HHMMSS.log`) y symlink `latest.log`.
- **Parsers de Slicing**:
  - **Cargo (Rust)**: Detecta resúmenes (`test result: ok. X passed; 0 failed`), bloques de `failures:`, panic messages y compiler errors (`error[E0...]:`).
  - **Vitest / Jest (Node/TypeScript)**: Detecta bloques `FAIL`, assertions `Expected / Received` y stack traces.
  - **Pytest (Python)**: Detecta bloques `FAILURES`, `AssertionError` y traceback.
  - **Fallback Universal**: Heurística basada en regex para cualquier comando desconocido buscando palabras clave de fallo (`FAILED`, `error:`, `fatal:`, `panic:`, `exception`).
- **Bypass Flag**: Flag `--raw` para deshabilitar el compactado en caso de requerir streaming tradicional.

### Out-of-Scope (v1)
- Daemon interactivo de terminal (TUI) para inspeccionar logs en vivo.
- Intercepción invasiva del shell del usuario mediante LD_PRELOAD o ptrace.
- Modificación automática de los scripts de `package.json` o `Makefile` sin acción explícita del usuario.

---

## 5. Architectural Alignment & Compliance

1. **Compound Engineering Alignment**:
   - Fortalece el principio de **Context Guarding** presente en `AGENTS.md` y `ce-work`.
   - Permite a los subagentes iterar en TDD (Stage 4) sin saturar la ventana de contexto antes de alcanzar Stage 5 (Verification) o Stage 6 (Knowledge Capture).
2. **Arquitectura Actual de CE-AI**:
   - Integrado en `src/commands/run.rs`.
   - Reutiliza `crate::error::CeError` y la resolución de workspace de `crate::commands::Context`.
   - Utiliza `std::process::Command` con pipes estándar para streaming concurrente a disco y buffer en memoria.
3. **Consumo de Tokens**:
   - **Gasto de tokens:** Cero tokens de inferencia (ejecución 100% determinística en Rust local).
   - **Ahorro de tokens:** **Reducción neta estimada del 40% al 70%** de los tokens quemados por terminal en sesiones típicas de codificación.

---

## 6. Success Criteria

- [ ] `ce-ai run --compact cargo test` devuelve un máximo de 5 líneas en ejecuciones exitosas.
- [ ] En caso de fallo, la salida contiene el mensaje de error exacto y el stacktrace relevante, omitiendo tests verdes y líneas de compilación superadas.
- [ ] El log completo se conserva de forma determinística en `target/ce-logs/latest.log` (o `.ce-ai/logs/latest.log`).
- [ ] Rotación automática garantizada: no más de 10 archivos de log persisten en disco simultáneamente.
- [ ] Soporte verificado para Cargo, Vitest/Jest, Pytest y fallback universal por regex.
- [ ] 100% verificado con tests unitarios en Rust y pruebas de integración.

---

## 7. Next Steps

- Handoff a `/ce-plan` para descomponer en unidades de implementación Scrum (`Unit 1: Subcommand & Log Rotating Engine`, `Unit 2: Slicing Parsers`, `Unit 3: CLI Integration & Docs`).
