# Spec Delta: `openspec-hygiene-and-error-transparency`

## Requirement: Ledger truthfulness

- **WHEN** a change folder's feature is verifiably shipped,
  **THEN** its `tasks.md` MUST either have every box checked or carry a
  STATUS header citing the ship evidence, with any residual open boxes
  declared unaudited.
- **WHEN** a change folder is completed under criterion (a) or (b),
  **THEN** it MUST live under `openspec/changes/archive/`.
- **WHEN** a folder stays active with open boxes,
  **THEN** it MUST appear in the triage table of
  `archive/README.md` with its open-task count.

## Requirement: Best-effort transparency

- **WHEN** a best-effort cleanup operation succeeds,
  **THEN** no output MUST be produced.
- **WHEN** a best-effort removal targets an absent path (`NotFound`),
  **THEN** no output MUST be produced.
- **WHEN** a best-effort operation fails with any other error,
  **THEN** a stderr warning MUST name the affected path and the cause.
- **WHEN** de-adoption processes multiple vendor stubs,
  **THEN** one vendor's cleanup failure MUST NOT abort the others.
