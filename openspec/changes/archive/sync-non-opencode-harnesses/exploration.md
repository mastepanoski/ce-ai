# Exploration: Decoupling `sync` and `upgrade` from OpenCode Precondition

## Background & Code Analysis
1. `src/commands/sync.rs:37-55`:
   - `sync::run` calls `InstallManifest::load(&opencode_dir)` directly and errors if it fails.
   - It assumes `opencode` is always the primary harness.
2. `src/commands/sync.rs:78-177`:
   - `sync_with` also unconditionally loads `InstallManifest::load(&opencode_dir)?`.
   - It performs diff against `opencode_dir/compound-engineering` and writes `InstallManifest` to `opencode_dir`.
   - In lines 406-410, it pushes `opencode` to `surfaces` unconditionally.
3. `src/commands/upgrade.rs`:
   - `upgrade::run` calls `sync::sync_with`, inheriting the exact same failure when OpenCode is missing.

## Evaluated Alternatives

### Option A: Hard dependency on OpenCode (Status Quo)
- Requires documenting that OpenCode must always be installed before `ce-ai sync` or `upgrade` can be run.
- **Drawback**: Violates multi-harness architecture and user expectations. Users adopting Claude, Cursor, or Custom without OpenCode cannot run `sync` or `upgrade`.

### Option B: Harness-Neutral Source Resolution & Gated OpenCode Sync (Selected)
- In `sync::run`, detect active harnesses:
  - If no harnesses are installed (`state.installed_harnesses` is empty and no host-detected harnesses), fail fast with `no harnesses installed — run ce-ai install first`.
  - Resolve `source_json` and `version` by checking:
    1. OpenCode manifest if OpenCode is active and manifest exists.
    2. Any installed harness's manifest on disk (custom, claude, etc.).
    3. `source` and `version` recorded in `state.installed_harnesses`.
    4. `state.release_provenance`.
- In `sync_with`:
  - Determine `opencode_active`.
  - If `opencode_active`, sync OpenCode managed dir, write OpenCode manifest, and include `opencode` in matrix.
  - If `!opencode_active`, do NOT touch `opencode_dir` and do NOT include `opencode` in matrix.
  - Sync other active harnesses (Custom directory tree, table-driven companion registrations, adopted skill surfaces).
- **Benefits**:
  - Full parity across all AI tools.
  - No changes to existing OpenCode behavior.
  - Seamless support for single-harness and multi-harness setups.
