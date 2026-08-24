# Spec Delta
- **WHEN** a PR exceeds 400 changed lines, **THEN** it MUST carry either a
  chained split or an approved size exception, plus a Changed-Lines Forecast.
- **WHEN** corrections respond to review findings, **THEN** they MUST fit
  min(200, ceil(original/2)) lines per cycle, one bounded correction each;
  overflow becomes scoped follow-ups.
- **WHEN** OpenSpec tasks are authored, **THEN** work units carry ~200-line
  budgets that rescopes may only narrow.
- **WHEN** classifying severity, **THEN** size MUST NOT be used as a risk
  signal.
