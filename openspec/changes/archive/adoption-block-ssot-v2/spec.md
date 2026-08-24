# Spec: Adoption Block SSOT Guidance (v2)

## ADDED Requirements

### Scenario 1: Full tier carries the Single Source of Truth rule
WHEN `ce-ai init-prj <path> --tier full` renders the managed block
THEN the block MUST contain a "Single Source of Truth Rule" section stating
that ideation artifacts are disposable inputs to be distilled into OpenSpec,
and that ideation skills are skipped when requirements and approach are clear.

### Scenario 2: Orchestrator tier carries one distillation directive
WHEN `ce-ai init-prj <path> --tier orchestrator` renders the managed block
THEN it MUST contain exactly one line stating that ideation outputs are
disposable inputs to be distilled before delegation, never maintained in
parallel.

### Scenario 3: Minimal tier is unchanged
WHEN `ce-ai init-prj <path> --tier minimal` renders the managed block
THEN its content MUST be byte-identical to the v1 minimal block.

### Scenario 4: Header and state version cannot drift
WHEN any adoption block is written
THEN the header `v=` field and `ProjectAdoptionEntry.block_version` in
`state.json` MUST both equal the same `BLOCK_VERSION` constant (currently 2).

### Scenario 5: In-place upgrade from v1
WHEN `init-prj` runs against a project whose `AGENTS.md` contains a v1 managed
block plus surrounding user content with CRLF line endings
THEN only the text between the begin/end markers is replaced with the v2
block, surrounding user content and CRLF endings are preserved, and
`state.json` records `block_version == 2`.

### Scenario 6: Idempotent re-run
WHEN `init-prj` runs twice consecutively on the same project with the same
tier
THEN the second run reports the block as up-to-date and leaves `AGENTS.md`
byte-for-byte unchanged.

## Acceptance Criteria

- All scenarios covered by integration tests in `tests/cli.rs`.
- Full DoD gates pass (`fmt`, `clippy -D warnings`, `cargo test`, `make e2e`).
