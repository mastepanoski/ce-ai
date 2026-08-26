---
date: 2026-08-24
topic: usage-analytics-sources
focus: per-harness local persistence research before scoping usage analytics (#78)
mode: repo-grounded
---

# Ideation: Usage Analytics — Fuentes Locales por Harness

## Grounding Context

Codebase context: ce-ai orquesta agentes vía `src/harness/` (`HarnessKind`, patrón adapter). El issue #78 propone extender el patrón con un usage-source adapter y horas humanas manuales. Verificado en esta máquina: Claude 363MB de transcripts JSONL, OpenCode `opencode.db` SQLite presente (11.7GB + WAL), Codex rollouts por año/mes/día, Pi sessions por proyecto, Kimi y Antigravity instalados con persistencia parcial. Herramienta existente del usuario: ccusage.

## Topic Axes

- Fuente por harness (formato/localización/riqueza de campos)
- Quién llena el gap (adapter ce-ai vs tool existente vs nada hoy)
- Superficie de consulta v1 (report CLI / TUI / storage intermedio)

## Ranked Ideas

### 1. Trait `UsageSource` con adapters Tier-A: claude, opencode, codex, pi
**Description:** Adaptadores read-only sobre persistencia local ya confirmada; `ce-ai usage report [--from --to] [--by session|day|project|model]` lee-through sin almacenamiento intermedio en v1. Kimi/agy quedan detrás del trait para v2.
**Axis:** Fuente por harness
**Basis:** direct: probes locales — Claude JSONL (`input_tokens`, `cache_creation/read_input_tokens`, `output_tokens`); OpenCode SQLite `opencode.db` con columnas nativas `tokens_input/output/reasoning/cache_read/cache_write`, `cost`, `model`, `directory` (schema drizzle documentado; leer con conexión readonly por WAL activo); Codex rollout JSONL anual `total_token_usage` acumulativo → derivar deltas; Pi JSONL con `cost`/`model`/`usage`.
**Rationale:** Cuatro de seis harness tienen datos ricos HOY; el valor del issue se demuestra sin tocar kimi/agy.
**Downsides:** Schema drift por vendor (OpenCode migra su DB entre versiones); Codex acumulativo requiere lógica de deltas por timestamp.
**Confidence:** 85%
**Complexity:** Medium
**Status:** Unexplored

### 2. Kimi vía server API local; AGY diferido hasta esquema público
**Description:** Kimi Code expone rollup `{input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, total_cost_usd}` vía su server local (`127.0.0.1:58627/api/...`) pero solo con server vivo; su wire.jsonl no documenta campos de tokens. Antigravity persiste transcripts JSONL (`~/.gemini/antigravity*/brain/<id>/.system_generated/logs/transcript.jsonl`) con modelName pero sin usage fields confirmados + historia frágil (bug conocido de índice reset).
**Axis:** Quién llena el gap
**Basis:** external: Kimi Code Docs (Server API / Data locations / Sessions), Antigravity SDK lifecycle docs + hooks payload (`transcriptPath`, `modelName`).
**Rationale:** No bloquear el trait: ambos entran después como adapters que lean esas fuentes cuando los vendors las estabilicen o documenten.
**Downsides:** Kimi requiere server corriendo; AGY sin garantía de campos.
**Confidence:** 70%
**Complexity:** High
**Status:** Unexplored

### 3. Lean-on-tools donde existan; ce-ai aporta la capa cross-harness
**Description:** ccusage (Claude) y futuros CLIs por vendor cubren single-harness; el gap real que ce-ai llena es normalización multi-harness + correlación con proyectos/sesiones de state.json + horas humanas — nada existente hace eso.
**Axis:** Quién llena el gap
**Basis:** reasoned: ccusage/opencode-usage-cli son single-harness y no cruzan con state.json; el issue ya rechazó shell-out como primary pero su parsing informa los adapters.
**Rationale:** Evita duplicar ccusage; posiciona ce-ai como agregador.
**Downsides:** Mantener parsers al día por vendor.
**Confidence:** 80%
**Complexity:** Low (decisión) / Medium (implementación)
**Status:** Unexplored

## Rejection Summary

| # | Idea | Reason Rejected |
|---|------|-----------------|
| 1 | Instrumentar agentes vía plugins/hooks para capturar usage live | Rechazado en el issue: invasivo; persistencia local ya cubre |
| 2 | Shell-out a ccusage/opencode-usage-cli como primary | Rechazado en el issue: single-harness + dependencia runtime |
| 3 | Storage intermedio indexado en v1 | YAGNI: read-through sobre fuentes locales alcanza para las consultas pedidas |
| 4 | Adapter OpenCode antes de confirmar DB local | La DB no existía al primer probe; ahora confirmada (11.7GB) — entra a Tier-A |
