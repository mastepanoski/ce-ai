# Exploration

## Findings

All top-level variants are wired through `commands::registry::Commands`.
The defect is therefore semantic alignment, not absent dispatch.

`usage::report` receives `from`, `to`, and `by` as underscore-prefixed
arguments and prints every ledger record. The Phase-1 usage boundary is
Claude-only (`~/.claude/projects`), so adding other adapters to make the
description true would expand this corrective change substantially.

`gate check` defaults to `GateMode::Enforce` and returns `CeError::Usage` when
Stage 4 lacks its required OpenSpec files. Changing the code to observe-only
would weaken an existing enforcement invariant; the correct narrow repair is
to document the actual default and `--mode observe` alternative.

`guard` stores a single `GuardrailState`; it is displayed by status and doctor
but is not consulted by the workflow or gate policy. A single stored harness
cannot support a meaningful per-harness disable operation. Validation avoids a
silent wrong-scope action while the public wording describes configuration,
not enforcement.

`tools install` registers MCP definitions and intentionally does not download
or execute third-party installers. This preserves separate companion ownership
and avoids a platform-specific installer/security expansion.

## Alternatives rejected

1. **Implement every advertised future capability now:** rejected as a
   multi-adapter/enforcement project outside an audit correction.
2. **Change gate default to observe:** rejected because it changes existing
   safety behavior; truthful help is the smaller compatible repair.
3. **Remove guard or its harness flag:** rejected because validation preserves
   CLI compatibility and makes its current state model explicit.
