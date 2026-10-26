# Requirements: Deterministic Quality & Test Integrity Engine

**Date:** 2026-10-26  
**Topic:** deterministic-quality-and-test-integrity-engine  
**Origin:** Extracción de valor combinada de `unclebob/ub-mode` (`crapper` + `mutator`) vía `docs/ideation/2026-10-25-unclebob-ub-mode-value-extraction-ideation.html`  
**Harness / CLI Scope:** `ce-ai` Rust CLI (`src/commands/doctor.rs`, `src/observation/state.rs`, `src/commands/workflow.rs`)  
**Ecosystem Alignment:** 100% complementario y desacoplado de `rtk` (RTK gestiona compresión de terminal; CE-AI gestiona integridad semántica y verificación de código).

---

## 1. Problem Statement

Los modelos de lenguaje y agentes de codificación (Claude Code, OpenCode, Codex, Gemini, Antigravity) exhiben dos debilidades estructurales sistemáticas durante los ciclos de desarrollo rápido:

1. **Tests Cosméticos / Aserciones Huecas (Stage 4 TDD & Stage 5 Verification)**:
   Los agentes generan suites de tests que reportan 100% de éxito en verde, pero que en realidad no prueban comportamiento (e.g. aserciones triviales como `assert!(true)`, mocks excesivos que aíslan la lógica real, o ausencia de verificaciones en ramas de error). Si la lógica de negocio se rompe, los tests continúan pasando.
2. **Inflación Invisible de Complejidad Ciclomática (Deuda Técnica Acumulada)**:
   Al implementar parches iterativos o manejar edge cases, los agentes anidan condicionales, bloques `match` gigantes y loops sin refactorizar. Sin una métrica cuantitativa en el flujo de trabajo, estas funciones acumulan una alta probabilidad de regresión y son difíciles de auditar visualmente.

Las herramientas tradicionales de mutation testing y análisis de cobertura requieren minutos u horas para repositorios completos, lo cual destruye el ciclo interactivo del agente o quema tokens innecesarios en revisiones manuales de código.

---

## 2. Proposed Architecture: Enfoque Dual Coordinado

En lugar de duplicar herramientas de terminal como `rtk`, CE-AI integra dos motores complementarios de validación determinística en Rust:

```
┌────────────────────────────────────────────────────────────────────────┐
│               CE-AI Quality & Test Integrity Engine                   │
├───────────────────────────────────┬────────────────────────────────────┤
│   Stage 5 Verification Engine     │       Repository Health Probe      │
│     (Micro-Mutador en Diff)       │       (Métricas CRAP en Doctor)    │
├───────────────────────────────────┼────────────────────────────────────┤
│ • Scope: Líneas modificadas       │ • Scope: Funciones modificadas /   │
│   en el Git Diff (staged/branch)  │   workspace completo               │
│ • Mutaciones: 1-2 flips clave     │ • Fórmula: CC² × (1 - Cov)³ + CC   │
│   (== ➔ !=, true ➔ false, < ➔ <=) │ • Fallback: CC > 15 si no hay LCOV │
│ • Latencia: < 3 - 5 segundos      │ • Reporte: Diagnóstico en doctor   │
│ • Objetivo: Matar tests huecos    │ • Objetivo: Frenar código spaghetti│
└───────────────────────────────────┴────────────────────────────────────┘
```

---

## 3. Subsystem Specifications

### 3.1 Subsystem A: Micro-Mutador Acotado al Diff (Stage 5 Verification)

* **Superficie de Invocación:**
  * Automático en la observación de Stage 5 (`src/observation/state.rs:observe_verification`).
  * Manual vía `ce-ai verify` o `ce-ai test --mutate`.
* **Estrategia de Mutación Quirúrgica:**
  * Escanea exclusivamente el `git diff` de la tarea actual (`git diff --unified=0 main...HEAD` o archivos staged).
  * Aplica un máximo de **1 a 2 mutaciones clave por bloque modificado** (Smoke-Test ultrarrápido):
    * Operadores de igualdad: `==` $\leftrightarrow$ `!=`
    * Operadores relacionales: `>` $\leftrightarrow$ `<=`, `<` $\leftrightarrow$ `>=`
    * Booleanos directos: `true` $\leftrightarrow$ `false`
    * Retornos de guardia: Negación de guard clauses.
* **Mecánica de Ejecución Aislada:**
  * Aplica la mutación temporal en memoria o en un archivo de trabajo temporal bajo sandbox.
  * Ejecuta la suite de pruebas del proyecto.
  * **Criterio de Evaluación:**
    * Si los tests **fallan**: El mutante fue *eliminado* (Killed). Los tests validan comportamiento real $\rightarrow$ **Pasa verificación**.
    * Si los tests **siguen pasando**: El mutante *sobrevivió* (Survived). Los tests son cosméticos/huecos $\rightarrow$ **Alerta de integridad**.
* **Salida Sintética para el Agente (0 tokens en éxito, 3 líneas en fallo):**
  ```text
  ⚠️ Stage 5 Verification: 1 surviving mutant detected in src/auth/token.rs:48
     Mutación: '==' invertido a '!='
     Impacto: Los tests pasaron en verde con la condición rota. Verifica tus aserciones.
  ```

---

### 3.2 Subsystem B: Perfilado de Riesgo CRAP en `ce-ai doctor`

* **Superficie de Invocación:**
  * `ce-ai doctor` (sección de salud de código / metrics probe) y flag `ce-ai doctor --metrics`.
* **Fórmula Empírica de Riesgo:**
  $$\text{CRAP} = CC^2 \times (1 - Cov)^3 + CC$$
  *Donde $CC$ es la Complejidad Ciclomática y $Cov$ es la fracción de cobertura (0.0 a 1.0).*
* **Estrategia de Fallback Elegante (Graceful Fallback):**
  1. Si existe reporte LCOV/Coverage en disco (`target/coverage/`, `coverage/lcov.info`, etc.):
     * Extrae $Cov$ por función y calcula el score CRAP exacto.
     * Alerta diagnóstica en funciones con $\text{CRAP} > 30$ (alto riesgo / espagueti no testeado).
  2. Si **NO** existe reporte de coverage en disco:
     * Calcula $CC$ estático mediante conteo de ramas (`if`, `match`, `while`, `for`, `?`, `&&`, `||`).
     * Alerta diagnóstica si $CC > 15$ en funciones modificadas recientemente, recomendando modularización o adición de tests.
* **Salida de Diagnóstico:**
  ```text
  [CRAP Code Risk Analysis]
  ✓ 42 functions evaluated in modified modules.
  ⚠️ src/workflow/fsm.rs:184: `step_transition` (CC: 8, Cov: 15.0%, CRAP: 37.4) [HIGH RISK]
     Recomendación: Dividir la función o añadir tests para cubrir las ramas no ejercitadas.
  ```

---

## 4. Policy, Severity & Governance Alignment

1. **Principio de Cero Autoridad Semántica (AGENTS.md Invariante 6):**
   * Por defecto, las advertencias de mutantes sobrevivientes y CRAP elevado son **estrictamente diagnósticas** en tareas tácticas o menores a 200 LOC. No bloquean arbitrariamente el trabajo del desarrollador.
2. **Flag `--strict` para Integración Continua (AGENTS.md Invariante 2):**
   * En pipelines de CI (`gh pr checks`) o flujos regulados, `ce-ai doctor --strict` o `ce-ai verify --strict` retorna código de salida `CeError::Verification` (código 6) si hay mutantes sobrevivientes en código nuevo o funciones con $\text{CRAP} > 30$.
3. **Consumo de Tokens:**
   * **Cero incremento de tokens en el prompt**: No se inyectan directivas verbales redundantes en `AGENTS.md` ni en el contexto.
   * **Señal quirúrgica**: La CLI solo emite líneas de alerta cuando se detecta un problema concreto de aserción o complejidad.

---

## 5. Scope Boundaries

### In-Scope (MVP)
- Parser AST rápido en Rust para complejidad ciclomática ($CC$) sobre Rust (con regex fallback para TS/Python).
- Inversor de operadores booleanos y relacionales acotado a las líneas del git diff.
- Integración en `observe_verification` en `src/observation/state.rs`.
- Probe de salud diagnóstica en `src/commands/doctor.rs`.
- Flag `--strict` para forzar exit code 6 en fallo de verificación.

### Out-of-Scope (v1)
- Mutation testing global en repositorios completos (reservado para herramientas especializadas de CI nocturno).
- Mutaciones complejas de tipos, structs o macros.
- Generación automática de reportes HTML pesados (el resultado se mantiene como texto compacto de consola y JSON estructurado).

---

## 6. Success Criteria

- [ ] El micro-mutador evalúa el diff de una tarea típica en menos de 5 segundos.
- [ ] Si un test del agente carece de aserciones reales sobre una función modificada, el micro-mutador detecta el mutante vivo y lo reporta con `file:line`.
- [ ] `ce-ai doctor` reporta el score CRAP si hay LCOV en disco, o advierte si $CC > 15$ en ausencia de coverage.
- [ ] Cero colisión con `rtk`: la compresión de salida de terminal sigue perteneciendo exclusivamente a RTK.
- [ ] La suite completa de tests de `ce-ai` pasa en verde con 100% de cumplimiento de los invariantes de `AGENTS.md`.

---

## 7. Next Steps

- Handoff a `/ce-plan` para crear el plan de implementación técnica en `docs/plans/` con TDD triads:
  * **Unit 1**: Motor de cálculo de Complejidad Ciclomática ($CC$) y probe CRAP en `src/commands/doctor.rs`.
  * **Unit 2**: Micro-mutador sintáctico acotado a líneas de git diff en `src/verification/mutant.rs`.
  * **Unit 3**: Integración en Stage 5 Verification (`src/observation/state.rs`) y soporte del flag `--strict`.
