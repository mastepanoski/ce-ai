---
date: 2026-08-24
topic: usage-analytics-and-human-hours
---

# Usage Analytics & Human Hours

## Summary

`ce-ai` gana captura determinística de consumo (tokens por sesión/modelo/día/commit-rango) vía pre-commit hook y registro asistido de horas humanas, consolidados en un ledger commiteado particionado por usuario (`.ce-ai/usage/<dev>.jsonl`) con atribución por persona — habilitando estimación de trabajo futuro y facturación en tokens (billing-grade) más horas (estimación self-report). Entrega por fases: Claude-first, luego resto de Tier-A, luego horas.

---

## Problem Frame

`ce-ai` orquesta agentes en múltiples harnesses pero nadie ve qué cuesta esa orquestación: cada harness persiste usage localmente en formatos propios y dispersos (Claude: 363MB de JSONL verificados; OpenCode: SQLite con columnas nativas de tokens; Codex: rollouts acumulativos; Pi: JSONL con cost/model), herramientas como ccusage cubren un solo harness, y el tiempo humano de revisión entre commits no se registra en ningún lado. Cuando varias personas trabajan el mismo repositorio para un cliente, tampoco hay forma de atribuir quién consumió qué — bloqueando tanto la estimación de trabajos futuros como la facturación defendible del trabajo ya hecho.

---

## Actors

- A1. **Developer**: corre harnesses y commitea; su HOME local es la fuente de tokens; su git user atribuye horas y commits del ledger.
- A2. **Client/Stakeholder**: recibe reportes tokens+horas por feature/período para facturación o estimación.
- A3. **Pre-commit hook (ce-ai)**: pipeline actor que ejecuta captura determinística y prompt de horas.

---

## Key Flows

- F1. **Capture at commit**
  - **Trigger:** `git commit` con hooks instalados
  - **Actors:** A3, A1
  - **Steps:** hook invoca capture → parsea fuentes Tier-A desde el último marcador → appendea records normalizados al ledger → si TTY interactivo, prompt de minutos/actividad
  - **Outcome:** ledger actualizado con tokens+horas de esa unidad de trabajo; commit continúa
  - **Covered by:** R1, R2, R3, R4, R5
- F2. **Report generation**
  - **Trigger:** `ce-ai usage report [filtros]`
  - **Actors:** A1/A2
  - **Steps:** lee ledger commiteado (+ registros locales pendientes) → agrega según filtros → tabla humana o --json
  - **Outcome:** totales reproducibles tokens+horas
  - **Covered by:** R8, R9
- F3. **Manual hours backfill**
  - **Trigger:** `ce-ai hours log`
  - **Actors:** A1
  - **Outcome:** HourRecord corregido/backfill con autor git
  - **Covered by:** R6, R7

---

## Requirements

**Capture (hook-driven, deterministic)**
- R1. El pre-commit hook captura usage de fuentes Tier-A locales desde el último marcador y appendea records normalizados al ledger antes de completar el commit. El marcador es **por-proyecto y local-only** (nunca commiteado); el bootstrap inicial captura "desde ahora" (sin backfill histórico en el hook); la dedup usa clave determinística `harness+session_id+ventana` (inmune a --amend/rebase).
- R2. La captura no depende de memoria del usuario; fallos de lectura de una fuente son warning no-fatal.
- R3. Cada record snapshot-ea **solo campos whitelisted**: author (git user), timestamp, harness, session_id, cwd sanitizado (basename del proyecto), model, tokens (input/output/reasoning/cache_read/cache_write), schema_version. Jamás contenido de conversaciones ni texto libre; validación estricta de esquema pre-write (línea inválida => reject + warning).
- R4. Entornos no interactivos: solo tokens; sin prompts. Todo período/fuente sin cobertura queda **marcado como `uncaptured`** en reportes (distinguiendo cero-real de no-capturado).

**Human hours**
- R5. En commits interactivos, prompt de minutos + activity; HourRecord con author=git user y anclaje al commit.
- R6. `hours log` manual para backfill/correcciones; `hours list` para consulta.
- R7. Activity enum: review/prompting/debugging/other; source enum: manual (wakatime/zeittracker futuros).

**Reporting**
- R8. `usage report` agrega el ledger por fecha | commit-range | project | model | user | activity; salida tabla humana + `--json`; incluye **coverage markers** (`captured` / `uncaptured`) por fuente y período (R4).
- R9. Tokens+modelo son reproducibles desde el ledger commiteado alone; los montos de costo requieren price-source externo versionado (fuera del repo).

**Sources**
- R10. Tier-A adapters: claude (JSONL), opencode (SQLite readonly), codex (rollout deltas), pi (JSONL cost/model/usage).
- R11. Kimi/Antigravity diferidos detrás del trait.

**Privacy & layout**
- R12. Consent gate: la primera captura en un repo requiere opt-in explícito del developer; repos con remote público ⇒ destino anonimizado/agregado o sink alternativo (DB remota/servicio) — jamás atribución cruda en history público.
- R13. Allowlist exhaustiva: únicamente los campos declarados arriba pueden persistirse; esquema desconocido => record dropeado + warning (fail-closed).
- R14. Ledger particionado shard-per-author: `.ce-ai/usage/<dev>.jsonl` — merges disjuntos por diseño.
- R15. Correcciones post-commit mediante entradas compensatorias (append-only; sin rewrite de history).

---

## Acceptance Examples

- AE1. **Covers R1, R3.** Given hooks instalados y una sesión previa de Claude, when se commitea, then el ledger gana records con author/timestamp/model/tokens de esa sesión.
- AE2. **Covers R4, R5.** Given un commit interactivo tras trabajo del agente, then se promptea horas una vez y se registran con autor; given un commit en CI/scripted, then sin prompt y tokens igual registrados.
- AE3. **Covers R9.** Given los transcripts originales borrados, when se regenera el mismo reporte, then los números son idénticos desde el ledger.
- AE4. **Covers R7, R6.** Given `hours log a1..a2 --minutes 90 --activity review`, then queda HourRecord atribuido al git user visible en reportes.

---

## Success Criteria

- Mauro genera un resumen cliente-facing: **tokens+modelos billing-grade** y **horas como estimación self-report explícitamente etiquetada**, multi-usuario.
- El mismo ledger produce idénticos números de tokens/modelo en cualquier máquina; los gaps de captura (`uncaptured`) son visibles, no silenciosos.

---

## Scope Boundaries

**Entrega por fases (M6):**
- **Fase 1**: ledger shard-per-author + hook capture + `usage report` sobre Claude (mayor volumen real).
- **Fase 2**: adapters restantes Tier-A (opencode/codex/pi).
- **Fase 3**: horas prompt/log interactivas.
Cada fase satisface independientemente una porción de los success criteria.

**Posicionamiento (M7)**: usage analytics es un **módulo opt-in** — NO forma parte del install de gobernanza por defecto de init-prj; se activa por decisión explícita de repo (coherente con consent gate R12).

- Sin server ni sink externo; consolidación exclusivamente vía git.
- Sin instrumentación live de agentes (se leen persistencias locales).
- Kimi/Antigravity adapters diferidos (detrás del trait).
- Integraciones WakaTime/zeittracker post-v1 (enum `source` listo).
- Sin panel TUI en v1.
- El ledger queda excluido del counting contract de #108.
- NO se commitean montos de costo — granularidad = tokens + modelo (el costo lo calcula un sistema externo).

---

## Key Decisions

- **Ledger commiteado vs sink externo**: consolidación multi-usuario nativa vía git merge, auditable por diseño, sin infraestructura.
- **Snapshot-at-capture vs re-parse**: facturación defendible ante transcripts mutables (compactación/borrado/bugs de vendors).
- **Tokens determinísticos (hook) / horas semi-asistidas (prompt en commit)**: los segundos no son medibles pasivamente; el commit es el punto real de unidad de trabajo.
- **Tokens sí, costos no en el ledger**: granularidad por modelo alcanza para calcular costos externamente sin exponer tarifas en el historial compartido.
- **Destino según visibilidad (M1)**: privado => ledger atribuido tras consent; público => anonimizado/agregado o sink alternativo.
- **Marker por-proyecto + local-only + bootstrap "ahora" (M3)**: aísla clientes entre sí, evita parseos masivos en hooks, y la dedup determinística neutraliza --amend/rebase.
- **Shard-per-author (M4)**: merges disjuntos por diseño; sin union-drivers ni .gitattributes.

---

## Dependencies / Assumptions

- Una persona por máquina/HOME (atribución por autor de commit); pair-programming en una misma sesión no es distinguible.
- Los formatos de harness pueden derivar; los adapters toleran versiones recientes y se testean contra fixtures.
- El ledger queda excluido del LOC counting contract (#108).
- Rollout del hook extendido a repos existentes (vehículo asumido: `make hooks` re-run o init-prj installer — diseño exacto diferido a Planning).

---

## Outstanding Questions

### Resolve Before Planning

*(ninguno)*

### Deferred to Planning

- Mecanismo exacto de rollout del hook extendido a repos ya creados.
- Diseño del marcador "último punto capturado" por fuente (global por usuario vs por proyecto).
