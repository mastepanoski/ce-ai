# Spec Delta: `transactional-ops`

## Requirement 1 — Journal durability

- **WHEN** install or sync performs any tracked filesystem mutation,
  **THEN** the prior content (or absence) of the target path MUST be
  persisted in `<config_dir>/install-journal.json` **before** the mutation.
- **WHEN** the command finishes successfully, **THEN** the journal MUST be
  removed; `state.json` MUST be the final persisted mutation.

## Requirement 2 — Deterministic recovery

- **WHEN** a stale journal exists at the start of install/sync,
  **THEN** applied mutations MUST be rolled back in reverse (restore prior
  bytes / delete created files) with a stderr warning, after which the
  command proceeds fresh.
- **WHEN** the journal is corrupt, **THEN** recovery MUST warn and treat it
  as absent.

## Requirement 3 — Diagnosis

- **WHEN** `ce-ai doctor` runs with a journal present,
  **THEN** it MUST report an `install-journal:` finding naming the command.

## Requirement 4 — Fault injection

- **WHEN** `CE_AI_FAIL_AFTER_WRITES=<N>` is set,
  **THEN** tracked write N+1 MUST fail while earlier writes remain journaled.
- **WHEN** such an injected failure occurs between any two steps,
  **THEN** the next successful run MUST restore user-visible pre-command
  content without data loss and complete cleanly.
