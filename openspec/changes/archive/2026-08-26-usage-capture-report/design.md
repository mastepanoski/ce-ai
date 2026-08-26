# Design: `usage-capture-report`

## Architecture

```
pre-commit hook (.githooks/pre-commit)
    │
    ├── ce-ai usage sync --json     ← captura determinística (tokens)
    └── ce-ai hours prompt          ← solo si TTY interactivo

ce-ai usage report [--from --to] [--by day|session|model] [--json]
    │
    ├── lee .ce-ai/usage/<dev>.jsonl (shard-per-author)
    ├── agrega por filtros
    └── coverage markers por fuente/período
```

## Marker
Per-project local-only: `<config_dir>/markers/<hash(cwd)>.json` con `{last_captured: timestamp}` por fuente. Bootstrap = ahora (sin backfill histórico en hooks).

## Ledger schema (por línea JSONL)
```json
{"author":"...","timestamp":"...","harness":"...","session_id":"...","cwd_basename":"...","model":"...","input_tokens":N,"output_tokens":N,"cache_read":N,"cache_write":N,"reasoning_tokens":N}
```

## Hours
HourRecord con author=git user, minutes, activity enum, commit padre. Prompt TTY-gated.
