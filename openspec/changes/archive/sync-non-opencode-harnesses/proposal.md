# Proposal: Enable `ce-ai sync` and `upgrade` for Non-OpenCode Harnesses

## Problem Statement
Currently, `ce-ai sync` and `ce-ai upgrade` treat OpenCode's install manifest (`<opencode_dir>/compound-engineering/install-manifest.json`) as an unconditional precondition for execution:
```rust
let manifest = InstallManifest::load(&opencode_dir)
    .map_err(|_| CeError::Runtime("no install-manifest.json — run install first".into()))?;
```
And inside `sync_with`, OpenCode's managed directory is unconditionally diffed, written, and asserted.

When a user installs a native harness (such as `claude`, `cursor`, `pi`, `copilot`) or a `custom` harness without installing `opencode`, running `ce-ai sync` fails immediately with:
```
error: runtime error: no install-manifest.json — run install first
```
This contradicts `docs/user-guide/sync-and-upgrade-mechanisms.md:34`, which promises synchronization across all supported AI tools without OpenCode being a mandatory dependency.

## Scope Boundaries
- **In-Scope**:
  - Contextually resolve source tree and version for `sync` and `upgrade` from installed harnesses, manifest files, or `state.installed_harnesses`.
  - In `sync_with`, gate OpenCode directory sync, diffing, and manifest generation to only execute when OpenCode is an active harness.
  - When OpenCode is absent but other harnesses are installed, report the active harnesses in the sync verification matrix (e.g. `registered` or `verified` for custom) without an erroneous `opencode` row.
  - If no harnesses are installed at all, produce a clear error: `no harnesses installed — run ce-ai install first`.
  - Verify that `upgrade` successfully resolves new releases and syncs installed non-OpenCode harnesses.
  - Document the behavior in `docs/user-guide/sync-and-upgrade-mechanisms.md`.
- **Out-of-Scope**:
  - Changing companion registration contracts for table-driven harnesses.
  - Redefining adoption semantics for skills.

## Success Criteria
- `ce-ai install --harness claude` followed by `ce-ai sync` exits with code 0 and reports `claude: registered`.
- `ce-ai install --harness custom ...` followed by `ce-ai sync` exits with code 0 and reports `custom: verified`.
- `ce-ai upgrade` works when only non-OpenCode harnesses are installed.
- Running `ce-ai sync` with nothing installed reports `no harnesses installed — run ce-ai install first`.
- All integration tests, unit tests, and CI matrix pass cleanly.
