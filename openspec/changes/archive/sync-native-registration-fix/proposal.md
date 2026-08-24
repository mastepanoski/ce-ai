# Proposal: `sync-native-registration-fix`

## Why

The multi-harness audit series surfaced follow-up debt while implementing
`custom-harness-r4`: `sync_with()` routes every harness without an explicit
arm into the generic OpenCode branch:

```rust
} else {
    let _ = crate::opencode::config::ensure_plugin_and_skills(&target_config, ...);
}
```

Consequences per affected harness:

| Harness | `target_config` | Effect of the fallthrough |
| --- | --- | --- |
| `kimi` | `<dir>/mcp.json` | When the file exists (install created it), `plugin` and `skills.paths` keys are injected into the native MCP config — real corruption |
| `agy` | `<dir>/config/mcp_config.json` | Same injection into the native Antigravity config |
| `fx` | `<dir>/mcp.json` | Same injection into the native Vercel Labs fx config |
| `pi` | `<dir>/skills` (a directory path) | `read_config` fails and the error is silently swallowed (`let _ =`) — a latent landmine plus an invariant-#5 smell |

Additionally, these four harnesses are excluded from the post-sync hash
verification matrix even though install copies full skill trees for them, so
their surfaces report "not verified" despite being verifiable.

## What Changes

- Give Kimi, Agy, Fx, and Pi explicit re-registration arms in `sync_with()`
  mirroring their install behavior (MCP registration where applicable +
  managed-skills copy).
- Replace the silent generic fallthrough with a hard `CeError::Runtime`
  naming any unsupported harness found in `state.json`.
- Extract the duplicated managed-skills copy into one helper that propagates
  IO errors (invariant #5); adopt it in the existing Claude/Codex/Copilot/
  Grok arms (Claude previously ignored copy failures).
- Extend the verification matrix to hash-check the skill trees of all eight
  directory-copying harnesses (Claude, Codex, Copilot, Grok, Kimi, Agy,
  Pi, Fx), with Agy's `config/skills` root handled explicitly.
- Bookkeeping annotations in `openspec/changes/multi_harness_support/
  tasks.md` (Task 2.5 references the removed `generic_json.rs` and a
  de-scoped DeepSeek adapter; Task 2.6 cites a test name that never
  existed) — docs-only.

## In Scope

- `src/commands/sync.rs` only, plus its CLI integration tests.
- The two tasks.md reality-note annotations above.

## Out of Scope

- `install.rs` behavior (already correct per-harness).
- fx adapter heuristics (`fx.rs:21-28`): re-verified as deterministic and
  fully unit-tested by `fx-adapter-audit-refinements`; no change needed.
- Claude arm's historical error-swallowing beyond adopting the shared
  helper (no behavioral change beyond propagating real IO failures).

## Risks & Mitigations

| Risk | Mitigation |
| --- | --- |
| Sync now fails loudly on unexpected state entries | Only reachable via hand-edited state.json; error names the entry |
| Verification extension flags pre-existing drift | Matches user expectation: drift SHOULD fail sync with exit 6 |

## Success Criteria

- After `install --harness <kind>` + `sync`, none of the four native config
  files gains OpenCode-format keys; bytes stay identical.
- Deleted managed skill files under each of the four roots are restored by
  sync, and the verification matrix reports `✓ <kind>`.
- Full gate suite green (fmt, clippy `-D warnings`, cargo test, make e2e).
