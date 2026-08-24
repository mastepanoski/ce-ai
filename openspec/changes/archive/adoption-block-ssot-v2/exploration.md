# Exploration: Adoption Block SSOT Guidance (v2)

## Options Evaluated

### Tier scope

| Option | Assessment |
| :--- | :--- |
| 1. `full` only | Simplest, but leaves the confusion vector alive where `orchestrator` explicitly tells agents to run `ce-brainstorm`. |
| 2. `full` section + one line in `orchestrator` (**chosen**) | Covers both places that reference ideation skills; keeps cost near zero. |
| 3. All three tiers | Inflates `minimal`, which is deliberately short — violates KISS/YAGNI. |

### Delivery mechanism

| Option | Assessment |
| :--- | :--- |
| Extend `render_block_content` strings (**chosen**) | Static text, zero runtime cost, sha-based idempotent update already handles rollout. |
| New `migrate-prj` command | Rejected (YAGNI): re-running `init-prj` already replaces the block between markers; a second command doubles surface for no capability gain. |
| External template file loaded at runtime | Rejected: breaks single-binary distribution and adds I/O failure modes to a pure-string path. |

## Version Bump Mechanics (verified against source)

- The header literal `v=1` lives inline in `run()`'s `format!`
  (`src/commands/init_prj.rs`); `ProjectAdoptionEntry.block_version` is set to
  the literal `1`. Both must move to a shared `BLOCK_VERSION` constant so they
  cannot drift apart again.
- Block detection is marker-based (`BLOCK_BEGIN_MARKER` /
  `BLOCK_END_MARKER`), not version-based: re-running `init-prj` compares the
  full rendered block (sha256) and replaces when different — v1 blocks are
  replaced with no migration code.
- `deinit-prj` restores pre-block content by markers only; version-agnostic,
  unaffected.

## Trade-off Accepted

Projects adopted before this change keep v1 blocks until the operator re-runs
`init-prj`. Accepted because adoption is opt-in per project and silent remote
mutation of user instruction files would violate the preserve-user-configs
invariant.
