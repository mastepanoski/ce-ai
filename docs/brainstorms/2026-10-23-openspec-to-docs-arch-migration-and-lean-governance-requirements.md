---
title: "Migración de OpenSpec a docs/arch/ y Gobernanza Arquitectónica Lean en Compound Engineering"
date: "2026-10-23"
topic: "openspec-to-docs-arch-migration"
type: "requirements"
status: "ready_for_planning"
---

# Requerimientos: Migración de OpenSpec a docs/arch/ y Gobernanza Arquitectónica Lean

## 1. Resumen Ejecutivo

Este documento define la reestructuración integral de la gobernanza arquitectónica en `ce-ai`, eliminando la fricción, la sobrecarga cognitiva y el solapamiento metodológico derivados del directorio raíz heredado `openspec/`. Se establece una arquitectura documental unificada y nativa bajo `docs/arch/` (con ADRs para decisiones estructurales permanentes, especificaciones vivas para contratos canónicos, trazabilidad de constelaciones multi-repositorio y archivo consolidado), combinada con entregas ágiles de rodajas verticales (Scrum ≤ 200 LOC), TDD guiado por tríadas en `docs/plans/`, y controles de ciberseguridad anclados en estándares internacionales (ISO 27001, OWASP, NIST). Incluye un motor de migración asistida (`ce-ai init-prj --migrate-arch`) para repositorios existentes.

## 2. Contexto y Problema

1. **Colisión Cognitiva y Metodológica (SDD vs. Compound Engineering):**
   La presencia del directorio raíz `openspec/` activa en los modelos de lenguaje (LLMs) y agentes autónomos el sesgo hacia *Spec-Driven Development* (SDD) tradicional. El agente asume que debe instanciar la pesada ceremonia de 5 archivos (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`), colisionando frontalmente con las primitivas nativas de Compound Engineering (`docs/brainstorms/`, `docs/plans/`, `docs/solutions/`).
2. **Duplicación de Artefactos y Quema de Tokens:**
   El paquete de 5 archivos de OpenSpec replica el 90% de lo que ya producen `ce-brainstorm` (problema y alternativas) y `ce-plan` (diseño y checklist). Esto genera doble mantenimiento, desincronizaciones continuas entre planes y tareas, y consumo innecesario de tokens de contexto.
3. **Ausencia de Invariantes Normativos:**
   Los requisitos de seguridad frecuentemente se improvisan en lenguaje natural no estructurado, en lugar de estar anclados a normas auditables de la industria (ISO 27001/27002, OWASP Top 10 para LLM/Web/Mobile, NIST SP 800-53 y AI RMF 1.0).
4. **Acoplamiento de `archive` al Árbol Heredado:**
   Los comandos `ce-ai archive` y `ce-ai archive compact` asumen rígidamente la ruta `openspec/changes/archive/`, bloqueando la modernización hacia un árbol de documentación limpio en `docs/`.
5. **Realidad Multi-Repositorio y Flotas Desconectadas:**
   En entornos reales, los proyectos no siempre son monorepos; se distribuyen entre múltiples repositorios donde:
   - Un desarrollador tiene decenas de repositorios no relacionados en su máquina (personales, de clientes, open source).
   - Solo el desarrollador conoce qué repositorios forman parte de una misma constelación de arquitectura global.
   - El desarrollador rara vez tiene todos los repositorios clonados localmente (solo 3 o 4 de los 40 microservicios de la empresa).
   - Se añaden nuevos repositorios y componentes de forma incremental en el tiempo.

## 3. Principios de Diseño Arquitectónico

- **Un solo dominio de Arquitectura en `docs/arch/`:** Eliminar la duplicidad de directorios en la raíz. Toda la arquitectura (decisiones, contratos, constelaciones y archivo) vive dentro de `docs/arch/`.
- **Decisiones Estructurales en ADRs (`docs/arch/adr/`):** Las decisiones técnicas duraderas (elección de base de datos, arquitectura de concurrencia, protocolos) se documentan como ADRs permanentes bajo el estándar Michael Nygard.
- **Contratos Vivos del Sistema en `docs/arch/system/`:** Las especificaciones canónicas de dominio (`REQ-*`) representan el estado actual del sistema (el "esquema de base de datos en producción").
- **Constelaciones de Repositorios ("Local-First, Remote-Graceful"):** Los repositorios multi-servicio declaran su pertenencia a un sistema (`docs/arch/system.yaml`), versionando contratos locales (`docs/arch/contracts/`) para operar de forma 100% aislada sin requerir que todos los repositorios hermanos estén descargados.
- **Incrementos de Valor Verticales en `docs/plans/` (Scrum + TDD):** El trabajo diario no genera carpetas de 5 archivos. Se ejecuta como rodajas verticales delgadas (walking skeletons ≤ 200 LOC) con tríadas TDD (RED ➔ GREEN ➔ REFACTOR) en el plan ágil, siendo la **única fuente de verdad de progreso**.
- **Ciberseguridad como Invariante de Dominio:** Los controles de seguridad se definen formalmente en `docs/arch/system/security.md` referenciando ISO 27001, OWASP y NIST.
- **Promoción y Archivo Atómico:** `ce-ai archive <feature>` promueve los deltas de requerimientos a `docs/arch/system/`, registra la trazabilidad en `docs/arch/archive/README.md` y limpia el estado sin intervención manual.

## 4. Requerimientos Funcionales y Técnicos

### 4.1 Estructura Canónica de Directorios (`docs/arch/`)

- R1. El sistema adoptará `docs/arch/` como el espacio unificado de arquitectura:
  - `docs/arch/adr/`: Registros de Decisiones de Arquitectura (`YYYY-MM-DD-<slug>-adr.md` o `000X-<slug>.md`).
  - `docs/arch/system/`: Especificaciones vivas del sistema con contratos formales (`REQ-<DOMAIN>-*`).
  - `docs/arch/system/security.md`: Invariantes normativos basados en ISO/IEC 27001, OWASP (LLM/Web) y NIST.
  - `docs/arch/system.yaml`: Manifiesto declarativo de constelación de repositorios (identificador del sistema, dependencias y contratos).
  - `docs/arch/contracts/`: Caché de esquemas públicos de integración (OpenAPI, Protobuf, JSON Schemas) para desarrollo desacoplado.
  - `docs/arch/deltas/`: Deltas arquitectónicos opcionales en curso para cambios de alta complejidad.
  - `docs/arch/archive/`: Archivo histórico consolidado de incrementos cerrados con ledger cronológico `README.md` y subdirectorio `milestones/` para rollups compactados.

### 4.2 Gobernanza Multi-Repositorio y Constelaciones

- R2. El sistema permitirá al desarrollador declarar explícitamente la constelación a la que pertenece el proyecto mediante `docs/arch/system.yaml`:
  - `system`: Nombre del sistema o constelación global (ej. `fintech-core`).
  - `component`: Identificador de este servicio (ej. `payments-api`).
  - `dependencies`: Lista de servicios dependientes con su URL remota, ruta local opcional (`local_path`) y contratos requeridos (`docs/arch/contracts/*`).
- R3. El motor de flota `ce-ai fleet` soportará segmentación por constelación:
  - `ce-ai fleet status --system <id>` y `ce-ai fleet sync --system <id>` operarán exclusivamente sobre los proyectos que pertenezcan a dicha constelación, ignorando repositorios personales o no relacionados registrados en la máquina.
  - Operación en modo "Local-First, Remote-Graceful": si un repositorio hermano no está clonado localmente, las herramientas y los tests TDD ejecutan contra el contrato versionado en `docs/arch/contracts/` sin fallar ni bloquear.
  - Crecimiento incremental: la incorporación de un nuevo microservicio (`ce-ai init-prj --system <id>`) se integra de forma transparente a la constelación sin afectar a los demás repositorios.

### 4.3 Lector Dual y Compatibilidad Hacia Atrás en CLI

- R4. Todos los comandos del CLI (`ce-ai spec`, `ce-ai workflow`, `ce-ai doctor`, `ce-ai archive`) implementarán resolución de rutas en dos niveles:
  - *Nivel 1 (Preferencia Moderna):* Si existe `docs/arch/system/`, se utiliza como fuente de especificaciones vivas. Si existe `docs/arch/archive/`, se utiliza como destino de archivo.
  - *Nivel 2 (Fallback Heredado):* Si no existe `docs/arch/system/` pero existe `openspec/specs/`, el CLI opera transparentemente sobre la ruta heredada sin romper flujos de trabajo existentes.

### 4.4 Comando de Migración Asistida (`ce-ai init-prj --migrate-arch`)

- R5. Se implementará la bandera `--migrate-arch` en `ce-ai init-prj` (y subcomando `ce-ai arch migrate`):
  - Detectará si el proyecto contiene `openspec/specs/` y/o `openspec/changes/`.
  - Admitirá `--dry-run` para previsualizar todas las mutaciones sin escribir en disco.
  - Trasladará atómicamente `openspec/specs/*.md` hacia `docs/arch/system/*.md`.
  - Trasladará `openspec/changes/archive/` hacia `docs/arch/archive/` preservando el historial.
  - Actualizará las referencias en `docs/arch/archive/README.md`.
  - Eliminará de forma segura el directorio raíz `openspec/` una vez vacío, erradicando el sesgo SDD en los agentes.
  - Actualizará las directivas del bloque gestionado en `AGENTS.md` (Adoption Block v9).

### 4.5 Evolución de `ce-ai archive` y `ce-ai archive compact`

- R6. El comando `ce-ai archive <feature>`:
  - Detectará si el cambio contiene un delta formal (cláusulas `WHEN ... THEN ...` con `REQ-*`).
  - Promoverá automáticamente las cláusulas hacia la especificación viva correspondiente en `docs/arch/system/<domain>.md` con deduplicación semántica.
  - Trasladará el paquete o plan completado hacia `docs/arch/archive/`.
  - Registrará la entrada en el ledger `docs/arch/archive/README.md`.
  - Reconciliará `state.json` y dejará el FSM del repositorio en estado limpio (`100% Ready`).
- R7. El comando `ce-ai archive compact`:
  - Operará por defecto sobre `docs/arch/archive/`.
  - Empaquetará paquetes antiguos en `docs/arch/archive/milestones/archive-<milestone>.tar.gz`.
  - Generará el rollup resumen en `docs/arch/archive/milestones/<milestone>.md`.

### 4.6 Andamiaje de Rodajas Verticales y Tríadas TDD en `docs/plans/`

- R8. La única fuente de verdad (SSOT) del progreso de ejecución será el plan ágil en `docs/plans/`:
  - Cada unidad de trabajo definirá una tríada TDD explícita (≤ 200 LOC):
    1. `Unit X.1 (RED)`: Test de contrato fallido que verifica el criterio `WHEN ... THEN ...` o contrato de `docs/arch/contracts/`.
    2. `Unit X.2 (GREEN)`: Implementación mínima para pasar el test.
    3. `Unit X.3 (REFACTOR & DoD)`: Verificación de calidad, ausencia de advertencias clippy (`-D warnings`) y cumplimiento de estándares.
  - Si el incremento altera o amplía contratos del sistema, declarará en cabecera: `targets_requirement: REQ-<DOMAIN>-<ID>`.
- R9. El motor de observabilidad (`ObservableWorkflowState`) inspeccionará en tiempo real los checkboxes `- [x]` de `docs/plans/`, alimentando el auto-checkpointing de `ce-ai workflow checkpoint` y la reanudación transparente en `ce-ai workflow resume` sin requerir archivos paralelos de tareas.

### 4.7 Diagnóstico y Saneamiento en `ce-ai doctor`

- R10. `ce-ai doctor` incorporará sondas de diagnóstico arquitectónico:
  - Detectará si el repositorio contiene carpetas heredadas `openspec/` y sugerirá la migración:
    `doctor-warn: legacy 'openspec/' directory detected — run 'ce-ai init-prj --migrate-arch' to modernize to docs/arch/`.
  - Ofrecerá auto-reparación no destructiva con `ce-ai doctor --fix`.
  - Validará la integridad de enlaces `REQ-*` entre `docs/plans/` y `docs/arch/system/`.
  - Validará la consistencia de contratos en `docs/arch/contracts/` declarados en `docs/arch/system.yaml`.

## 5. Delimitación de Alcance (Scope Boundaries)

### 5.1 Dentro del Alcance (In Scope)
- Creación y estandarización del esquema de directorios `docs/arch/` (`adr/`, `system/`, `contracts/`, `deltas/`, `archive/`).
- Especificación del formato de constelación `docs/arch/system.yaml` y soporte de filtrado en `ce-ai fleet --system`.
- Implementación del lector dual en `src/commands/spec.rs`, `src/commands/workflow.rs`, `src/commands/archive_compact.rs` y `src/commands/doctor.rs`.
- Implementación de la migración asistida `ce-ai init-prj --migrate-arch` con soporte `--dry-run`.
- Actualización de `ce-ai archive` y `ce-ai archive compact` para trabajar con `docs/arch/archive/`.
- Documento normativo inicial de ciberseguridad `docs/arch/system/security.md` mapeado a ISO 27001, OWASP y NIST.
- Actualización de `AGENTS.md`, `CONCEPTS.md` y documentación de usuario (Diátaxis).

### 5.2 Fuera del Alcance (Deferred / Out of Scope)
- Sincronización automática de red Git remota (pull/push masivo) entre repositorios no clonados.
- Generación heurística de código Rust desde especificaciones (se descartó por fragilidad; TDD manual asistido es superior).
- Modificación de motores de CI/CD externos a GitHub Actions.

## 6. Criterios de Aceptación y Verificación Empírica

1. **Cero Advertencias de Compilación y Formato:**
   - `cargo fmt --check` pasa sin observaciones.
   - `cargo clippy --all-targets --all-features -- -D warnings` pasa con 0 advertencias.
2. **Cobertura de Pruebas Unitarias e Integración (`cargo test`):**
   - Nuevos tests en `src/commands/tests/spec.rs` verifican resolución de rutas en `docs/arch/system/` y fallback a `openspec/specs/`.
   - Tests de migración verifican que `--dry-run` no altera el disco y que la migración real traslada archivos y elimina `openspec/` sin pérdida de datos.
   - Tests de `archive` verifican archivado y actualización del ledger en `docs/arch/archive/README.md`.
   - Tests de `fleet` verifican el filtrado por `--system <id>`.
3. **E2E Containerizado (`make e2e`):**
   - La suite Docker E2E valida la inicialización de proyectos, migración `--migrate-arch`, y ejecución limpia de `doctor`.
4. **Verificación de Diagnóstico (`ce-ai doctor`):**
   - Un repositorio con `openspec/` emite advertencia de migración.
   - Tras ejecutar `ce-ai init-prj --migrate-arch`, `ce-ai doctor` reporta 100% de salud y cero warnings de migración.
