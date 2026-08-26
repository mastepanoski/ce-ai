# Exploration

## Fuentes verificadas por harness (local probes + docs oficiales)

| Harness | Fuente | Formato | Campos usage | Estado |
|---|---|---|---|---|
| Claude Code | ~/.claude/projects/**/*.jsonl | JSONL por línea | input_tokens, cache_creation/read, output_tokens | ✅ 363MB |
| OpenCode | opencode.db SQLite WAL | tokens_* + cost + model (columnas nativas) | ✅ 11.7GB | ✅ |
| Codex | rollout-*.jsonl acumulativo | total_token_usage{...} | ✅ | ✅ |
| Pi | sessions/<workDirKey>/*.jsonl | cost, model, usage | ✅ | ✅ |
| Kimi | wire.jsonl + server API :58627 | rollup sin doc pública wire / server rollup completo | ⚠️ server-vivo only | ⚠️ |
| Antigravity | transcript.jsonl brain/<id> | modelName + interactions sin token fields | ⚠️ frágil | ⚠️ |

## Decisión de alcance v1
Claude-first. OpenCode/Codex/Pi en Fase 2 del trait. Kimi requiere server vivo; AGY sin usage fields confirmados — diferidos.
