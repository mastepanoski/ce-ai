# Spec Delta: `usage-capture-report`

## Requirement 1 — Deterministic capture
- **WHEN** pre-commit hook se ejecuta con hooks instalados, **THEN** el usage acumulado en fuentes Tier-A locales desde el último marcador se registra en `.ce-ai/usage/<dev>.jsonl` con author/timestamp/session/model/tokens.
- **WHEN** no hay TTY interactivo, **THEN** solo tokens se registran (sin prompt de horas).
- **WHEN** una fuente es ilegible, **THEN** warning non-fatal y el período queda marcado `uncaptured` para esa fuente.

## Requirement 2 — Reporting
- **WHEN** `ce-ai usage report [--from --to] [--by day|session|model]` corre, **THEN** agrega el ledger con coverage markers (`captured`/`uncaptured`) y salida tabla humana + `--json`.

## Requirement 3 — Human hours
- **WHEN** commit interactivo, **THEN** prompt minutos+activity; HourRecord atribuido al git user anclado al commit padre.
- **WHEN** `hours log` manual, **THEN** backfill/corrección con los mismos campos.
