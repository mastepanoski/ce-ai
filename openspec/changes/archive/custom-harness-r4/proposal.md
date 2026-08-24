# Proposal: `custom-harness-r4`

## Why

Audit of the multi-harness series (target 11/11) found Requirement 4 of
`multi_harness_support` unimplemented: `ce-ai install --harness custom`
announces and parses the mode but performs no real work.

Verified failure evidence on `main` (d8f4763):

1. **Dead adapter**: `CustomAdapter::new` is annotated `#[allow(dead_code)]`;
   no command ever constructs it.
2. **Fictional write path**: with no dedicated branch, `install --harness
   custom` falls through the generic OpenCode-format branch and writes a
   `{plugin: [...], skills: {paths: [...]}}` JSON into `~/.config/custom/
   custom.json` — a directory no known harness reads. The same fallthrough
   exists in `sync.rs`. This is the exact invariant-#5 violation class
   ("No Dummy Fallbacks") that justified the DeepSeek de-scope.
3. **Three divergent defaults**: `mod.rs` hardcodes
   `~/.config/custom/custom.json`; the disconnected adapter points at
   `~/.ce-ai/custom_harness.json`; the dead `GenericJsonAdapter` points at
   `~/.custom/config.json`.

Meanwhile README advertises "plus custom fallback mode" and gates pass only
because no test exercises the path.

## What Changes

- Implement R4 for real: `--plugins-dir`, `--skills-dir`, `--rules-file`
  CLI flags plus a persisted config file at `~/.ce-ai/custom_harness.json`.
- Route `install`, `uninstall`, and `sync` for `HarnessKind::Custom` through
  a dedicated branch backed by `CustomAdapter` / `CustomHarnessConfig`.
- Adopt a **single contract** for custom-mode paths:
  `~/.ce-ai/custom_harness.json`; remove `~/.config/custom`,
  `~/.custom/config.json`, and the dead `generic_json.rs` module.
- Uninstall becomes surgical for custom mode: removes exactly the
  manifest-recorded CE files and strips managed blocks; never deletes user
  content in shared directories.
- Sync gains hash-verified custom surfaces in its verification matrix.

## In Scope

- `src/harness/custom.rs`, `src/harness/mod.rs` (Custom arms),
  deletion of `src/harness/generic_json.rs`.
- `install.rs`, `uninstall.rs`, `sync.rs`: Custom branches, new flags,
  state-entry `custom` field.
- CLI integration tests (`tests/cli.rs`) and unit tests.
- Docs: README claim now true, `docs/user-guide/harness-matrix.md`.

## Out of Scope

- Interactive (`inquire`) prompts for custom paths; scriptable flags only.
- MCP server registration for custom harnesses (no known config format).
- Model assignment / agent maps for custom harnesses (already rejected via
  `supports_agent_definitions`).
- The pre-existing sync fallthrough affecting native harnesses Pi/Kimi/Agy/Fx
  (recorded in exploration.md as a separate follow-up finding).
- Rewriting the frozen `openspec/changes/multi_harness_support` artifacts;
  this change supersedes their Task 2.6 status.

## Risks

| Risk | Mitigation |
| --- | --- |
| Custom dirs are user-owned; bulk deletion would destroy data | Manifest-driven surgical removal only |
| State entry loses resolved paths between commands | Persist resolved `custom` object inside the state entry at install time |
| Config-file schema drift | Schema is versioned by this spec; unknown keys preserved, missing required keys = Usage error |

## Success Criteria

- `install --harness custom --plugins-dir P --skills-dir S [--rules-file R]`
  copies managed plugin/skill assets into the given directories, records a
  manifest + state entry, and never writes any fabricated config file.
- Without configuration the command fails fast with `CeError::Usage`
  (exit code 2) and actionable guidance.
- `uninstall --harness custom` removes exactly what install recorded,
  strips managed blocks from the rules file, preserves all other user files.
- `sync` re-copies and SHA256-verifies custom assets; drift fails with
  exit code 6.
- Zero references remain to `~/.config/custom` or `~/.custom/config.json`.
