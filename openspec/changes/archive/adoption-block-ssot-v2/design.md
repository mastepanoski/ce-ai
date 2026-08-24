# Design: Adoption Block SSOT Guidance (v2)

Source requirements: `docs/brainstorms/2026-08-22-openspec-ssot-adoption-block-requirements.md`

## Changed Component

`src/commands/init_prj.rs` — `render_block_content(tier: AdoptionTier)` and
the block-header construction in `run()`. No other module changes.

## Data / Contract Changes

### 1. Shared version constant

```rust
pub const BLOCK_VERSION: u32 = 2;
```

Used in:
- Header: `format!("<!-- ce-ai:block begin v={} tier={} sha256={} -->", BLOCK_VERSION, tier, sha)`
- State: `ProjectAdoptionEntry { block_version: BLOCK_VERSION, ... }`

Rationale: today `v=1` and the literal `1` drift independently; one constant
makes Scenario 4 structurally guaranteed.

### 2. Full tier — appended section (verbatim)

```
### Single Source of Truth Rule
Ideation artifacts (`docs/brainstorms/*.md`, `docs/ideation/*.md`) are disposable inputs, NOT parallel specifications. Distill their conclusions into the OpenSpec files above (`proposal.md`, `exploration.md`) and reference the source doc instead of copying content. Never maintain brainstorm/ideation documents in sync with OpenSpec. Skip ideation skills entirely when requirements and approach are already clear.
```

Mirrors the root `AGENTS.md` wording so both surfaces teach one model.

### 3. Orchestrator tier — appended single line (verbatim)

```
- Ideation outputs (`docs/brainstorms/`, `docs/ideation/`) are disposable inputs: distill them into the specs before delegation; never maintain them in parallel.
```

### 4. Minimal tier — untouched (byte-equality guard test).

## Invariants Preserved

- Markers `<!-- ce-ai:block begin` / `end -->` unchanged → `deinit-prj`
  restore and replacement logic work across versions.
- Atomic write path stays `crate::state::write_atomic`.
- No schema change in `state.json` (field types unchanged).

## Test Plan Mapping (spec ➔ tests)

| Spec scenario | Test |
| :--- | :--- |
| 1 | assert rendered full block contains SSOT strings |
| 2 | orchestrator contains distillation line exactly once |
| 3 | byte-equality of minimal string vs pinned literal |
| 4 | header + state entry both derive from `BLOCK_VERSION` |
| 5 | hand-written v1 block + CRLF + user content; run; assert in-place v2 |
| 6 | double-run idempotence (existing behavior, now locked by test) |

## Docs

`docs/user-guide/project-adoption-guide.md`: tier table gains "v2" note for
full/orchestrator contents and an "Upgrading adopted projects" subsection:
re-run `ce-ai init-prj <path> --tier <t>` after binary upgrade.
