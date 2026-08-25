# Proposal: `usage-capture-report`

## Why
Issue #78: ce-ai orchestra harnesses pero no hay visibilidad de tokens consumidos ni horas humanas — bloqueando estimación y facturación tokens+horas por persona.

## What Changes
- Módulo `src/usage/`: ledger shard-per-author (`.ce-ai/usage/<dev>.jsonl`), marcador por-proyecto local-only, adaptador Claude JSONL (Tier-A, mayor volumen real: 363MB verificados).
- Comandos: `ce-ai usage sync` (captura determinística desde último marcador), `ce-ai usage report` (agregación con filtros y coverage markers), `ce-ai hours log/list`.
- Integración pre-commit hook: captura automática + prompt de horas interactivo.
- Consent gate por visibilidad del remote.

## Out of Scope
- Adapters opencode/codex/pi/kimi/agy (Fase 2+, detrás del trait).
- Costos en el ledger (granularidad = tokens + modelo).
- WakaTime/zeittracker (enum `source` listo, integración post-v1).
