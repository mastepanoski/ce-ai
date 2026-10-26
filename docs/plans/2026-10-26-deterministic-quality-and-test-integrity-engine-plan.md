---
title: "Plan de Implementación: Deterministic Quality & Test Integrity Engine"
date: "2026-10-26"
type: plan
origin: "docs/brainstorms/2026-10-26-deterministic-quality-and-test-integrity-engine-requirements.md"
status: "draft"
---

# Plan: Deterministic Quality & Test Integrity Engine

## Summary

Este plan implementa el **Deterministic Quality & Test Integrity Engine** en `ce-ai`, extrayendo los principios mecánicos de `unclebob/ub-mode` (`crapper` y `mutator`) como herramientas locales determinísticas en Rust, garantizando **cero consumo de tokens de inferencia** y **desacoplamiento total de RTK** (el cual permanece como el dueño exclusivo de la compresión de salida stdio de terminal).

El sistema consta de un enfoque dual coordinado:
1. **Micro-Mutador Acotado al Diff (Stage 5 Verification)**: Inyecta 1-2 mutaciones booleanas y relacionales en las líneas del git diff para detectar tests cosméticos / aserciones huecas en menos de 5 segundos.
2. **Perfilado de Riesgo de Código CRAP en `ce-ai doctor`**: Calcula la métrica empírica $\text{CRAP} = CC^2 \times (1 - Cov)^3 + CC$ si existen reportes LCOV en disco, con un *graceful fallback* a Complejidad Ciclomática ($CC > 15$) cuando no hay cobertura.

Ambas capacidades operan en modo diagnóstico no bloqueante por defecto (preservando el fast-path lean), con soporte para el flag `--strict` en matrices de CI (`gh pr checks`).

---

## Problem Frame

Durante el ciclo de Compound Engineering, los agentes de IA cometen dos errores habituales que pasan desapercibidos:
* **Tests Cosméticos (Aserciones Huecas)**: En Stage 4 (TDD) y Stage 5 (Verification), los agentes escriben tests que reportan 100% verde pero no verifican el comportamiento real (debido a mocks laxos o ausencia de aserciones críticas).
* **Espaguetización Oculta (Deuda de Complejidad)**: Funciones que acumulan condicionales anidados y ramas no testeadas sin que el agente las refactorice.

El mutation testing tradicional en repositorios enteros es inviable por latencia (minutos u horas). Al acotar la mutación exclusivamente a las líneas modificadas en el git diff y calcular CRAP como diagnóstico estático en `doctor`, CE-AI dota al desarrollador y al agente de feedback matemático inmediato sin consumir tokens ni añadir dependencias externas.

---

## Requirements Traceability

- **R1 (Cálculo AST de Complejidad Ciclomática)**: Parser estático en Rust para contar decisiones estructurales (`if`, `match`, loops, `?`, `&&`, `||`).
- **R2 (Fórmula de Riesgo CRAP con Graceful Fallback)**: Aplicar $\text{CRAP} = CC^2 \times (1 - Cov)^3 + CC$ si hay LCOV en disco; fallback a $CC > 15$ si no hay datos de cobertura.
- **R3 (Sonda Diagnóstica en `ce-ai doctor`)**: Integrar el análisis CRAP en `ce-ai doctor` y comando `ce-ai doctor --metrics`.
- **R4 (Micro-Mutador Quirúrgico en Git Diff)**: Generar y ejecutar 1-2 mutaciones clave (`==` $\leftrightarrow$ `!=`, `>` $\leftrightarrow$ `<=`, booleanos) exclusivamente sobre líneas modificadas en el diff de la tarea.
- **R5 (Integración en Stage 5 Verification)**: Reflejar la integridad de tests en `observe_verification` (`src/observation/state.rs`) y exponer el subcomando `ce-ai verify` (o `ce-ai workflow verify`).
- **R6 (Gobernanza & Flag `--strict`)**: Diagnóstico no bloqueante por defecto; retorno de exit code `CeError::Verification` (código 6) cuando se use `--strict` y existan mutantes vivos o $\text{CRAP} > 30$.
- **R7 (Desacoplamiento Estricto con RTK)**: Cero solapamiento con wrappers de terminal; RTK continúa siendo el companion exclusivo para compresión de stdio.
- **R8 (Alineación Documental & Monotonic Accretion)**: Accreción estricta de conceptos en `CONCEPTS.md` validada con `scripts/validate-concepts.py`.

---

## Key Technical Decisions (KTDs)

- **KTD-1 (Diff-Only Mutation Scope)**: En lugar de mutar archivos enteros, el generador de mutantes lee únicamente los chunks del `git diff --unified=0` contra la rama base (`main` o HEAD staged). Esto reduce el espacio de mutación a 1-5 mutaciones por tarea, ejecutándose en < 3 segundos.
- **KTD-2 (Sandboxed In-Memory / Shadow Overlay)**: La aplicación de mutaciones se realiza sobre una copia temporal en `target/ce-mutants/` o reemplazo atómico revertido inmediatamente después de correr la suite de pruebas del proyecto, garantizando que el árbol de trabajo nunca quede en estado corrupto ante interrupciones.
- **KTD-3 (Graceful Fallback para Cobertura)**: No forzar la instalación ni ejecución de generadores de coverage (`llvm-cov`, `tarpaulin`). Si `target/coverage/lcov.info` o `coverage/lcov.info` existe en disco, se usa; si no, se reporta $CC$ pura, evitando fricción o fallos en entornos mínimos.
- **KTD-4 (Formato de Salida Zero-Token)**: Si todos los mutantes son eliminados (tests verdes y reales) y el CRAP es saludable, la salida de consola es de exactamente 1 línea. Solo se emite un bloque de 3 líneas cuando un mutante sobrevive o una función excede el umbral de riesgo.
- **KTD-5 (Zero Semantic Authority Invariant)**: Cumpliendo con el Invariante 6 de `AGENTS.md`, estos chequeos son meramente consultivos por defecto en tareas pequeñas (< 200 LOC), activando el modo estricto únicamente con `--strict` en pipelines de CI.

---

## High-Level Technical Design

```
src/
├── quality/                     # [NUEVO] Motor de Métricas y Análisis Estático
│   ├── mod.rs                   # Exportaciones públicas de calidad
│   ├── complexity.rs            # Parser AST de Complejidad Ciclomática (CC)
│   ├── coverage.rs              # Lector y extractor de cobertura LCOV
│   └── crap.rs                  # Evaluador de la fórmula CRAP (CC & Cov)
├── verification/                # [NUEVO] Motor de Verificación de Integridad
│   ├── mod.rs                   # Interfaz de verificación y evaluación
│   ├── diff.rs                  # Extractor de líneas modificadas desde git diff
│   ├── mutator.rs               # Inversor sintáctico de operadores (==, !=, bool)
│   └── runner.rs                # Ejecutor aislado de test suite con mutantes
├── observation/
│   └── state.rs                 # Integración de MutantSmoke en observe_verification
├── commands/
│   ├── doctor.rs                # Sonda de diagnóstico CRAP / flag --metrics
│   └── verify.rs                # Subcomando CLI ce-ai verify
└── main.rs / registry.rs        # Registro de comandos y argumentos CLI
```

---

## Implementation Units

- [x] **U1: Motor de Análisis de Complejidad Ciclomática (CC) y LCOV (`src/quality/`)**
- [x] **U2: Evaluador de Riesgo CRAP y Diagnóstico en `ce-ai doctor`**
- [x] **U3: Extractor de Git Diff y Generador Sintáctico de Micro-Mutaciones**
- [x] **U4: Ejecutor Aislado de Mutantes y Subcomando `ce-ai verify`**
- [x] **U5: Integración en Stage 5 Verification (`src/observation/state.rs`) y Soporte `--strict`**
- [x] **U6: Accreción en `CONCEPTS.md`, Documentación de Usuario y Puerta CI**

---

### U1: Motor de Análisis de Complejidad Ciclomática (CC) y LCOV (`src/quality/`)

- **Files to create/modify**:
  - `src/quality/mod.rs`
  - `src/quality/complexity.rs`
  - `src/quality/coverage.rs`
  - `src/quality/tests/complexity.rs`
- **Traceability**: R1, R2
- **Estimated Scope**: ~170 LOC
- **Description**:
  Implementar el cálculo de Complejidad Ciclomática ($CC$) para código fuente en Rust mediante análisis de tokens sintácticos (`syn` o escáner léxico optimizado de palabras clave: `if`, `match` arms, `while`, `for`, `loop`, `?`, `&&`, `||`).
  Implementar el extractor de LCOV que lee archivos de reporte existentes (`lcov.info`) y extrae el porcentaje de cobertura por función o archivo.
- **Test Triad**:
  - **Red**: Crear test unitario en `src/quality/tests/complexity.rs` con funciones de prueba (lineal $CC=1$, con bifurcaciones $CC=4$, con match complejo $CC=8$) que fallen al no existir el parser.
  - **Green**: Implementar `calculate_complexity(source: &str) -> usize` y `parse_lcov(path: &Path) -> HashMap<String, f64>` haciendo pasar los tests.
  - **Refactor**: Optimizar para no procesar comentarios o literales de string, garantizando ejecución en < 1ms por archivo.

---

### U2: Evaluador de Riesgo CRAP y Diagnóstico en `ce-ai doctor`

- **Files to create/modify**:
  - `src/quality/crap.rs`
  - `src/commands/doctor.rs`
  - `src/commands/tests/doctor.rs`
- **Traceability**: R2, R3, R6
- **Estimated Scope**: ~180 LOC
- **Description**:
  Implementar la fórmula matemática de CRAP:
  $$\text{CRAP}(CC, Cov) = CC^2 \times (1 - Cov)^3 + CC$$
  Integrar una nueva sonda diagnóstica en `ce-ai doctor` que analice las funciones modificadas o del workspace:
  - Si hay LCOV en disco, calcula CRAP y emite advertencia si $\text{CRAP} > 30$.
  - Si no hay LCOV, reporta $CC$ pura y emite advertencia si $CC > 15$.
  - Añadir soporte para el flag `--metrics` en `doctor::Args`.
- **Test Triad**:
  - **Red**: Escribir test que ejecute `doctor::run` sobre un workspace simulado con una función con $CC=16$ y espere encontrar el warning de complejidad en `doctor`.
  - **Green**: Implementar la sonda en `src/commands/doctor.rs` conectando con `src/quality/crap.rs`.
  - **Refactor**: Asegurar formato limpio y conciso en consola (máximo 2 líneas de advertencia por función detectada).

---

### U3: Extractor de Git Diff y Generador Sintáctico de Micro-Mutaciones

- **Files to create/modify**:
  - `src/verification/diff.rs`
  - `src/verification/mutator.rs`
  - `src/verification/tests/mutator.rs`
- **Traceability**: R4, KTD-1
- **Estimated Scope**: ~190 LOC
- **Description**:
  Implementar el módulo que extrae los rangos de líneas modificadas a partir de `git diff --unified=0`.
  Implementar el micro-mutador de código que escanea esas líneas y genera mutaciones determinísticas puntuales:
  - `==` $\leftrightarrow$ `!=`
  - `>` $\leftrightarrow$ `<=` y `<` $\leftrightarrow$ `>=`
  - `true` $\leftrightarrow$ `false`
  Limitar a 1-2 mutaciones candidatas por bloque modificado, produciendo un diff mutado en memoria listo para aplicar temporalmente.
- **Test Triad**:
  - **Red**: Test unitario que pase un fragmento de código con `if status == Status::Active && flag == true` y espere 2 candidatos de mutación identificados por línea y offset.
  - **Green**: Implementar `generate_diff_mutations(repo_root: &Path) -> Vec<MutationCandidate>`.
  - **Refactor**: Filtrar líneas que correspondan a tests o comentarios para mutar únicamente código de implementación.

---

### U4: Ejecutor Aislado de Mutantes y Subcomando `ce-ai verify`

- **Files to create/modify**:
  - `src/verification/runner.rs`
  - `src/commands/verify.rs`
  - `src/commands/registry.rs`
  - `src/commands/tests/verify.rs`
- **Traceability**: R4, R5, R6, KTD-2
- **Estimated Scope**: ~190 LOC
- **Description**:
  Implementar el runner de pruebas que toma cada candidato de mutación:
  1. Aplica la mutación en el archivo fuente de forma atómica.
  2. Ejecuta la suite de tests (vía comando configurado o fallback a `cargo test`).
  3. Reestablece inmediatamente el archivo original (mediante bloque guardián RAII / Drop que garantiza restauración incluso ante pánicos o señales).
  4. Registra si el mutante fue eliminado (tests fallaron) o sobrevivió (tests pasaron).
  Registrar el subcomando `ce-ai verify` en el enum `Commands` de `registry.rs`.
- **Test Triad**:
  - **Red**: Test de integración que ejecute `verify` sobre un repositorio fixture donde un test tiene una aserción vacía; debe reportar 1 mutante sobreviviente.
  - **Green**: Implementar `run_mutation_verification` y el comando `verify::run`.
  - **Refactor**: Incorporar time-out para que un mutante que cause un loop infinito sea cortado a los 5 segundos y computado como eliminado (Killed).

---

### U5: Integración en Stage 5 Verification (`src/observation/state.rs`) y Soporte `--strict`

- **Files to create/modify**:
  - `src/observation/state.rs`
  - `src/observation/tests/state.rs`
  - `src/commands/doctor.rs`
- **Traceability**: R5, R6, KTD-5
- **Estimated Scope**: ~160 LOC
- **Description**:
  Extender `VerificationObservation` en `src/observation/state.rs` para incluir:
  ```rust
  pub struct VerificationObservation {
      pub has_evidence: bool,
      pub uncommitted_changes: usize,
      pub review_receipt_stamped: bool,
      pub mutant_integrity_passed: Option<bool>,
      pub max_crap_score: Option<f64>,
  }
  ```
  Conectar `observe_verification` para que detecte recibos de mutación (`.mutant-receipt.json`) generados por `ce-ai verify`.
  En `doctor` y `verify`, implementar el comportamiento del flag `--strict`: si hay mutantes sobrevivientes o $\text{CRAP} > 30$, retornar `CeError::Verification("test integrity gate failed: surviving mutants detected")`.
- **Test Triad**:
  - **Red**: Test unitario en `src/observation/tests/state.rs` que compruebe que `observe_verification` detecte y refleje el recibo de mutación.
  - **Green**: Actualizar `observe_verification` y el chequeo `--strict` en los comandos.
  - **Refactor**: Asegurar que la serialización JSON del estado de workflow FSM mantenga compatibilidad hacia atrás.

---

### U6: Accreción en `CONCEPTS.md`, Documentación de Usuario y Puerta CI

- **Files to modify**:
  - `CONCEPTS.md`
  - `docs/user-guide/cli-reference.md`
  - `docs/user-guide/spec-driven-vs-chat-tdd-explained.md`
- **Traceability**: R7, R8
- **Estimated Scope**: ~110 LOC
- **Description**:
  1. Accretar quirúrgicamente a `CONCEPTS.md` (sin borrado destructivo) los términos:
     - `CRAP Code Risk Metric (Change Risk Analyzer and Predictor)`
     - `Diff-Scoped Mutation Smoke-Test`
     - `Hollow Test Detection (Cosmetic Assertions)`
  2. Validar accreción monotónica estricta con `python3 scripts/validate-concepts.py`.
  3. Documentar `ce-ai verify` y `ce-ai doctor --metrics` en `docs/user-guide/cli-reference.md`.
  4. Ejecutar la suite completa de calidad:
     ```bash
     cargo fmt --check
     cargo clippy --all-targets --all-features -- -D warnings
     cargo test
     ```
- **Test Triad**:
  - **Red**: Ejecutar `python3 scripts/validate-concepts.py` antes de editar para registrar el baseline de términos.
  - **Green**: Editar `CONCEPTS.md` quirúrgicamente y validar que el contador crezca estrictamente de forma monotónica.
  - **Refactor**: Verificar enlaces Diátaxis y referencias relativas sin enlaces absolutos.

---

## Risks & Mitigations

| Riesgo | Impacto | Mitigación |
|---|---|---|
| **Archivo fuente corrupto si el proceso se interrumpe durante la mutación** | Alto | Patrón RAII con `Drop` guard: el archivo original se respalda en memoria/disco antes de mutar y el Drop handler restaura el contenido original sin importar si el test falla, hace panic o recibe SIGINT. |
| **Falsos positivos de mutantes en macros complejas** | Medio | Limitar la mutación de operadores a código regular (`fn`, `impl`, bloques de lógica de negocio) y saltar código generado por macros o dependencias externas. |
| **Tiempo excesivo si la suite de tests es muy grande** | Medio | Acotado a máximo 2 mutaciones clave sobre líneas modificadas en el git diff y límite estricto de timeout de 5 segundos por ejecución de mutante. |

---

## Verification & Definition of Done Gate

- [x] `cargo fmt --check` pasa sin observaciones.
- [x] `cargo clippy --all-targets --all-features -- -D warnings` pasa limpio (cero warnings).
- [x] Todos los tests unitarios y de integración pasan (`cargo test`).
- [x] `ce-ai doctor --metrics` calcula CRAP si hay LCOV o advierte si $CC > 15$.
- [x] `ce-ai verify` inyecta mutaciones sobre el diff, detecta tests huecos y restaura el código fuente al 100%.
- [x] Flag `--strict` bloquea con exit code 6 en fallo de verificación y sale con código 0 en éxito.
- [x] `CONCEPTS.md` validado con `python3 scripts/validate-concepts.py` con accreción monotónica.
