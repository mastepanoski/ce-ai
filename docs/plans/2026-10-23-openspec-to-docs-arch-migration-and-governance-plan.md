---
title: "Plan de Implementación: Migración de OpenSpec a docs/arch/, Gobernanza Lean y Alineación Documental"
date: "2026-10-23"
type: plan
origin: "docs/brainstorms/2026-10-23-openspec-to-docs-arch-migration-and-lean-governance-requirements.md"
status: "ready_for_execution"
---

# Plan: Migración de OpenSpec a docs/arch/, Gobernanza Lean y Alineación Documental

## Summary

Este plan define la ejecución técnica para erradicar el sesgo cognitivo de *Spec-Driven Development* (SDD) tradicional provocado por el directorio raíz `openspec/`, centralizando la gobernanza arquitectónica en `docs/arch/` (con ADRs, contratos canónicos vivos, constelaciones multi-repositorio y archivo unificado). Establece a `docs/plans/` como la única fuente de verdad (SSOT) del progreso mediante tríadas TDD (≤ 200 LOC), añade migración asistida no destructiva (`ce-ai init-prj --migrate-arch`), conecta la seguridad con normas auditables (ISO 27001, OWASP, NIST) y alinea toda la documentación del repositorio (`README.md`, `CONCEPTS.md`, `AGENTS.md`, `docs/user-guide/`) con la arquitectura lean de Compound Engineering.

## Problem Frame

La presencia del directorio `openspec/` en la raíz de los proyectos host induce a los modelos de lenguaje a generar paquetes ceremoniales de 5 archivos (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`), duplicando los artefactos nativos de Compound Engineering (`docs/brainstorms/` y `docs/plans/`). Esto produce quema innecesaria de tokens, desincronizaciones de checklists y fricción en flujos de desarrollo dinámicos. Asimismo, la arquitectura actual carece de soporte declarativo para proyectos distribuidos en múltiples repositorios heterogéneos donde el desarrollador no tiene descargada toda la flota. Por último, la documentación pública y las guías de usuario mantienen referencias obsoletas a comandos y flujos SDD que deben ser depuradas y alineadas bajo el estándar Diátaxis.

## Requirements Traceability

- R1. Estructura canónica de directorios unificada bajo `docs/arch/` (`adr/`, `system/`, `contracts/`, `deltas/`, `archive/`).
- R2. Declaración de constelaciones multi-repositorio en `docs/arch/system.yaml` con arquitectura "Local-First, Remote-Graceful", validación de URLs sin credenciales embebidas (OWASP LLM-02) y caché de contratos en `docs/arch/contracts/`.
- R3. Soporte de filtrado por constelación en `ce-ai fleet` (`--system <id>`).
- R4. Lector dual retrocompatible en CLI (`docs/arch/` preferido con fallback a `openspec/`).
- R5. Comando de migración asistida `ce-ai init-prj --migrate-arch` con previsualización `--dry-run` y eliminación segura de `openspec/`.
- R6. Evolución del ciclo de vida de `ce-ai archive` hacia `docs/arch/archive/` (para deltas de arquitectura y contratos) y `docs/plans/archive/` (para planes ágiles de sprint completados), con promoción automática de deltas a `docs/arch/system/` y actualización del ledger.
- R7. Compactación generacional `ce-ai archive compact` operando sobre `docs/arch/archive/milestones/`.
- R8. Única fuente de verdad (SSOT) de progreso en `docs/plans/` estructurada en tríadas TDD (RED ➔ GREEN ➔ REFACTOR ≤ 200 LOC) con observabilidad en tiempo real.
- R9. Sondas de diagnóstico, prevención y auto-reparación en `ce-ai doctor`.
- R10. Invariantes de ciberseguridad normativa en `docs/arch/system/security.md` (ISO 27001, OWASP, NIST).
- R11. Alineación transversal de documentación: `README.md` (≤ 100 líneas), `CONCEPTS.md` (accreción monotónica), `AGENTS.md` (bloque v9), `CONTRIBUTING.md` y guías en `docs/user-guide/`.

## Key Technical Decisions (KTDs)

- KTD-1 (Dominio Único de Arquitectura): Centralizar todos los artefactos de arquitectura dentro de `docs/arch/` en lugar de carpetas hermanas dispersas. `docs/arch/adr/` para decisiones estructurales (estándar Michael Nygard), `docs/arch/system/` para contratos canónicos vivos (`REQ-*`) y `docs/arch/archive/` para trazabilidad histórica.
- KTD-2 (Constelaciones Multi-Repositorio Soberanas): El desarrollador declara explícitamente las relaciones entre proyectos mediante `docs/arch/system.yaml`. `ce-ai` nunca asume que repositorios no relacionados forman un sistema conjunto ni requiere que repositorios hermanos estén descargados localmente.
- KTD-3 (Lector Dual sin Migración Forzosa): Para garantizar cero regresiones en repositorios existentes, las funciones de inspección (`probe_openspec_context_in`, `list_domain_specs`, `observe_plan`) comprobarán primero la ruta moderna en `docs/arch/` y utilizarán `openspec/` únicamente como fallback transparente.
- KTD-4 (SSOT de Progreso en docs/plans/): Se elimina la verificación de checkboxes en `tasks.md`. `ObservableWorkflowState` escanea exclusivamente el plan activo en `docs/plans/`, alimentando el auto-checkpointing de `ce-ai workflow checkpoint` y el banner de reanudación `ce-ai workflow resume`.
- KTD-5 (Extirpación de Raíz para Desactivar Sesgo SDD): Tras la migración exitosa de `openspec/specs/` y `openspec/changes/archive/`, el directorio raíz `openspec/` se elimina de forma limpia. Su ausencia física impide que los agentes activen el modo SDD.
- KTD-6 (Alineación Documental Diátaxis Estricta): Se actualizan los manuales de usuario y el PRD de arquitectura para reflejar la gobernanza lean, manteniendo `README.md` por debajo de 100 líneas y actualizando `CONCEPTS.md` con validación estricta (`scripts/validate-concepts.py`).

## High-Level Technical Design

```
src/
├── commands/
│   ├── spec.rs             # Adaptación Dual-Path (docs/arch/system vs openspec/specs)
│   ├── workflow.rs         # Adaptación de archive a docs/arch/archive y SSOT en docs/plans
│   ├── archive_compact.rs  # Compactación generacional en docs/arch/archive/milestones
│   ├── init_prj.rs         # Implementación de --migrate-arch y soporte --system
│   ├── fleet.rs            # Filtrado de proyectos por constelación (--system <id>)
│   └── doctor.rs           # Sondas de detección de openspec/ y verificación de system.yaml
├── observation/
│   └── state.rs            # Observador de planes y contratos sin dependencia de tasks.md
└── compat/
    └── mod.rs              # Resolutor canónico de rutas de arquitectura (arch_dir, system_specs_dir)
```

## Implementation Units

### U1. Módulo de Resolución de Rutas de Arquitectura y Lector Dual

- Files to change:
  - `src/compat/mod.rs`
  - `src/commands/spec.rs`
  - `src/commands/tests/spec.rs`
- Traceability: R1, R4
- Scope: ~180 LOC
- Description:
  Implementar helpers canónicos en `src/compat/` para resolver dinámicamente las rutas de arquitectura:
  - `arch_dir(repo_root)` ➔ `repo_root.join("docs").join("arch")`
  - `system_specs_dir(repo_root)` ➔ comprueba `docs/arch/system/`, fallback a `openspec/specs/`
  - `archive_dir(repo_root)` ➔ comprueba `docs/arch/archive/`, fallback a `openspec/changes/archive/`
  Actualizar `src/commands/spec.rs` para utilizar estos helpers en `list_domain_specs`, `get_domain_spec_path`, `validate_domain_specs` y `promote_delta_to_domain_spec`.
- Test Scenarios:
  - T1.1: `test_system_specs_dir_prefers_docs_arch_system`: Verifica que si existen `docs/arch/system` y `openspec/specs`, se devuelve `docs/arch/system`.
  - T1.2: `test_system_specs_dir_fallbacks_to_openspec`: Verifica que si solo existe `openspec/specs`, se devuelve dicha ruta.
  - T1.3: `test_list_domain_specs_from_docs_arch`: Verifica el listado correcto de especificaciones vivas ubicadas en `docs/arch/system/*.md`.

### U2. Soporte de Constelaciones Multi-Repositorio y Caché de Contratos

- Files to change:
  - `src/state/state.rs`
  - `src/commands/fleet.rs`
  - `src/commands/tests/fleet.rs`
- Traceability: R2, R3
- Scope: ~190 LOC
- Description:
  Definir la estructura de datos para el manifiesto de constelación `docs/arch/system.yaml` (`SystemManifest`: `system`, `component`, `role`, `dependencies`).
  Implementar validación estricta de URLs de repositorio remoto para prohibir credenciales embebidas (tokens, passwords) mitigando riesgos de fuga de credenciales (OWASP LLM-02).
  Añadir el argumento `--system <id>` a `ce-ai fleet status` y `ce-ai fleet sync` en `src/commands/fleet.rs`, filtrando los repositorios registrados en `state.json` según el `system` declarado en su `docs/arch/system.yaml`. Si un repositorio dependiente no está clonado localmente, se marca como `(remote-contract: cached)` sin interrumpir la ejecución.
- Test Scenarios:
  - T2.1: `test_parse_system_manifest_valid`: Parsea correctamente un archivo `system.yaml` con dependencias remotas y locales.
  - T2.2: `test_fleet_filter_by_system`: Verifica que `ce-ai fleet status --system payments` solo inspecciona proyectos pertenecientes a `payments`.
  - T2.3: `test_fleet_graceful_missing_local_dependency`: Comprueba que una dependencia sin clon local no produce errores de ejecución.

### U3. Reestructuración de `ce-ai archive` y Compactación en `docs/arch/archive/`

- Files to change:
  - `src/commands/workflow.rs`
  - `src/commands/archive_compact.rs`
  - `src/commands/tests/archive_compact.rs`
- Traceability: R6, R7
- Scope: ~195 LOC
- Description:
  Actualizar `validate_and_archive_feature`, `sync_archive_readme_ledger` y `run_archive` en `src/commands/workflow.rs` para que el destino de archivo distinga claramente los artefactos: los deltas de arquitectura y especificaciones se archivan en `docs/arch/archive/`, mientras que los planes ágiles de sprint completados se mueven a `docs/plans/archive/`.
  Actualizar `src/commands/archive_compact.rs` (`run_archive_compact`, `probe_archive_compaction`) para compactar paquetes históricos desde `docs/arch/archive/` hacia `docs/arch/archive/milestones/archive-<milestone>.tar.gz`.
  Implementar la promoción automática de requisitos a `docs/arch/system/<domain>.md` durante el archivado sin requerir flags adicionales.
- Test Scenarios:
  - T3.1: `test_archive_moves_to_docs_arch_archive`: Verifica que archivar un cambio completado lo traslada a `docs/arch/archive/` y actualiza `docs/arch/archive/README.md`.
  - T3.2: `test_archive_auto_promotes_requirements`: Comprueba que las cláusulas `WHEN ... THEN ...` se incorporan al archivo de dominio en `docs/arch/system/`.
  - T3.3: `test_archive_compact_in_docs_arch`: Verifica la compactación de paquetes loose en `docs/arch/archive/milestones/`.

### U4. Observabilidad de Progreso SSOT en `docs/plans/` y Checkpointing

- Files to change:
  - `src/observation/state.rs`
  - `src/commands/workflow.rs`
  - `src/observation/tests/state.rs`
- Traceability: R8
- Scope: ~170 LOC
- Description:
  Depurar el observador de flujo de trabajo en `src/observation/state.rs` para retirar la dependencia de `tasks.md`. El observador calculará el progreso exclusivamente inspeccionando los checkboxes `- [x]` del plan activo en `docs/plans/`.
  Actualizar `probe_openspec_context_in` y las comprobaciones de desincronización de tareas en `src/commands/workflow.rs` para que el cálculo de progreso e inhibición de FSM se guíe por el plan de Compound Engineering.
- Test Scenarios:
  - T4.1: `test_observe_plan_computes_progress_without_tasks_md`: Verifica el cálculo del 100% de progreso con checkboxes de tríada TDD en `docs/plans/*.md`.
  - T4.2: `test_workflow_resume_reads_plan_progress`: Comprueba que `ce-ai workflow resume` emite el banner correcto derivado de `docs/plans/`.
  - T4.3: `test_checkpoint_records_stage4_with_plan`: Verifica que el auto-checkpointing infiere la etapa 4 basado en `docs/plans/` y modificaciones en git.

### U5. Comando de Migración Asistida (`ce-ai init-prj --migrate-arch`)

- Files to change:
  - `src/commands/init_prj.rs`
  - `src/commands/tests/init_prj.rs`
- Traceability: R5
- Scope: ~200 LOC
- Description:
  Añadir el flag `--migrate-arch` a `init_prj`:
  1. Verifica que el árbol de trabajo de Git esté limpio (`git status --porcelain`), abortando con error si hay cambios sin commitear salvo que se especifique `--force` (prevención de pérdida de datos).
  2. Detecta la existencia de `openspec/specs/` y `openspec/changes/`.
  3. Soporta `--dry-run` para emitir el plan de mutaciones sin tocar disco.
  4. Crea `docs/arch/system/`, `docs/arch/adr/` y `docs/arch/archive/`.
  5. Mueve `openspec/specs/*.md` a `docs/arch/system/`.
  6. Mueve `openspec/changes/archive/` a `docs/arch/archive/`.
  7. Elimina de forma segura el directorio raíz `openspec/`.
  8. Actualiza el bloque gestionado en `AGENTS.md` a la versión v9 (modelo lean sin menciones a SDD de 5 archivos).
- Test Scenarios:
  - T5.1: `test_migrate_arch_dry_run`: Verifica que `--dry-run` previsualiza las operaciones sin alterar el sistema de archivos.
  - T5.2: `test_migrate_arch_execution`: Comprueba la migración atómica completa y la eliminación del directorio `openspec/`.
  - T5.3: `test_migrate_arch_updates_agents_block`: Verifica que `AGENTS.md` queda actualizado a las directivas lean de `docs/arch/`.
  - T5.4: `test_migrate_arch_aborts_on_dirty_working_tree`: Comprueba que `--migrate-arch` aborta de forma segura si existen cambios no commiteados sin `--force`.

### U6. Sondas de Diagnóstico y Auto-Reparación en `ce-ai doctor`

- Files to change:
  - `src/commands/doctor.rs`
  - `src/commands/tests/doctor.rs`
- Traceability: R9
- Scope: ~180 LOC
- Description:
  Incorporar sondas de salud arquitectónica en `src/commands/doctor.rs`:
  - `probe_legacy_openspec_dir`: Detecta si existe `openspec/` en la raíz y emite `doctor-warn: legacy 'openspec/' directory detected — run 'ce-ai init-prj --migrate-arch'`.
  - `probe_system_manifest_integrity`: Valida la coherencia de `docs/arch/system.yaml` y la presencia de contratos requeridos en `docs/arch/contracts/`.
  - Integrar auto-reparación en `ce-ai doctor --fix` para ejecutar la migración asistida automáticamente con confirmación.
- Test Scenarios:
  - T6.1: `test_doctor_warns_on_legacy_openspec_dir`: Emite advertencia si existe `openspec/`.
  - T6.2: `test_doctor_fix_repairs_openspec_migration`: Ejecuta `--migrate-arch` y resuelve la advertencia de salud.
  - T6.3: `test_doctor_validates_system_manifest`: Detecta incoherencias sintácticas en `system.yaml`.

### U7. Especificación de Seguridad Normativa y Gobernanza de Invariantes

- Files to change:
  - `docs/arch/system/security.md`
  - `docs/arch/adr/0001-lean-architecture-and-docs-arch-unification.md`
- Traceability: R1, R10
- Scope: ~160 LOC
- Description:
  Crear la especificación canónica `docs/arch/system/security.md` conteniendo los invariantes normativos auditables del sistema:
  - ISO/IEC 27001 & 27002 (Integridad criptográfica, SHA256, mínimo privilegio).
  - OWASP Top 10 para LLMs (Sanitización de entradas, prevención de inyecciones y control de fuga de tokens/secretos).
  - NIST SP 800-53 / NIST AI RMF 1.0 (Escritura atómica `write_atomic`, contención de procesos y resiliencia).
  Registrar el primer ADR formal en `docs/arch/adr/0001-lean-architecture-and-docs-arch-unification.md` documentando el paso de OpenSpec SDD a `docs/arch/` con rodajas verticales Scrum y TDD.
- Test Scenarios:
  - T7.1: `test_security_spec_schema_valid`: `ce-ai spec validate` verifica que `docs/arch/system/security.md` cumple las reglas de especificación canónica.

### U8. Alineación Transversal de Documentación y Guías de Usuario

- Files to change:
  - `README.md`
  - `AGENTS.md`
  - `CONCEPTS.md`
  - `CONTRIBUTING.md`
  - `docs/user-guide/cli-reference.md`
  - `docs/user-guide/project-adoption-guide.md`
  - `docs/user-guide/compound-engineering-workflow-explained.md`
  - `docs/user-guide/spec-driven-vs-chat-tdd-explained.md`
  - `docs/user-guide/architectural-and-conceptual-guide.md`
- Traceability: R11
- Scope: ~190 LOC
- Description:
  Auditoría y alineación completa de la documentación para eliminar cualquier mención residual a ceremonias de 5 archivos de OpenSpec y reflejar la arquitectura lean en `docs/arch/`:
  - `README.md`: Mantener estricto ≤ 100 líneas con mapa Diátaxis actualizado.
  - `CONCEPTS.md`: Accretar monotónicamente los nuevos conceptos (*Repository Constellations*, *Lean Architecture Domain*, *Normative Cybersecurity Invariants*, *Dual-Path Reader*) y validar con `scripts/validate-concepts.py`.
  - `AGENTS.md`: Reflejar el Invariante 6 modernizado, eliminando la recomendación del paquete de 5 archivos y consagrando las rodajas verticales en `docs/plans/` con tríadas TDD.
  - `docs/user-guide/`: Modificaciones quirúrgicas (sustituciones puntuales de rutas y flags obsoletos, ~15-20 LOC por documento) para actualizar guías de adopción, flujo Compound Engineering y referencia CLI con `--migrate-arch` y `--system`, respetando estrictamente el presupuesto de cambios.
- Test Scenarios:
  - T8.1: `test_readme_line_count_within_limit`: Verifica que `README.md` no excede 100 líneas.
  - T8.2: `test_concepts_monotonic_accretion`: Ejecuta `python3 scripts/validate-concepts.py` comprobando que ningún término fue destruido y las nuevas definiciones cumplen el formato.
  - T8.3: `test_doc_lint_strict`: Ejecuta `cargo run -- doc lint --strict` garantizando cero enlaces rotos y formato Diátaxis válido.

## Dependencies and Sequencing

1. **Fase 1 (Fundación y Compatibilidad):**
   - Ejecutar U1 (Módulo de resolución de rutas y lector dual).
   - Ejecutar U2 (Soporte de constelaciones multi-repo en fleet).
2. **Fase 2 (Ciclo de Vida de Ejecución y Archivo):**
   - Ejecutar U3 (Reestructuración de archive hacia `docs/arch/archive/`).
   - Ejecutar U4 (Observabilidad de progreso SSOT en `docs/plans/`).
3. **Fase 3 (Herramientas de Migración y Diagnóstico):**
   - Ejecutar U5 (Comando de migración asistida `--migrate-arch`).
   - Ejecutar U6 (Sondas de diagnóstico en `ce-ai doctor`).
4. **Fase 4 (Especificaciones Canónicas y Gobernanza):**
   - Ejecutar U7 (Especificación de seguridad ISO/OWASP/NIST y ADR-0001).
5. **Fase 5 (Alineación Documental y Cierre):**
   - Ejecutar U8 (Alineación de documentación, `CONCEPTS.md` y `README.md`).

## Risk Analysis and Mitigation

- **Riesgo 1: Rotura de scripts existentes que dependen de la ruta `openspec/`.**
  - *Mitigación:* El lector dual implementado en U1 garantiza que si un repositorio aún no ha ejecutado `--migrate-arch`, todos los comandos CLI continúan funcionando con `openspec/` sin fallar.
- **Riesgo 2: Falsos positivos en `ce-ai doctor` para repositorios no migrados.**
  - *Mitigación:* La advertencia de `doctor` se clasifica como informativa/advisoria (`doctor-warn`) y no bloquea operaciones normales ni salidas exitosas (exit code 0).
- **Riesgo 3: Pérdida de historial de cambios archivados durante la migración.**
  - *Mitigación:* `--migrate-arch` incluye comprobación obligatoria de espacio, soporte `--dry-run` y traslado atómico sin truncado de contenido.
- **Riesgo 4: Deriva de conceptos en `CONCEPTS.md`.**
  - *Mitigación:* Cumplimiento estricto del Invariante 12 de `AGENTS.md` mediante ediciones quirúrgicas in-place y validación con `scripts/validate-concepts.py`.
