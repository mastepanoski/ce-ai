# Tasks: Non-OpenCode Harness Sync and Upgrade

- [x] Unit 1: Source & Version Resolution in `src/commands/sync.rs` (~45 LOC)
  - [x] Add `resolve_sync_source` helper in `src/commands/sync.rs` resolving source from OpenCode, other installed harness manifests, `state.installed_harnesses`, or `release_provenance`.
  - [x] Return clear error `no harnesses installed — run ce-ai install first` when no harness exists.

- [x] Unit 2: Gated OpenCode Sync in `sync_with` in `src/commands/sync.rs` (~65 LOC)
  - [x] Determine `opencode_active` upfront in `sync_with`.
  - [x] Only diff, copy/restore, and write OpenCode manifest when `opencode_active` is true.
  - [x] Only include `opencode` in `surfaces` when `opencode_active` is true.
  - [x] For custom and table-driven harnesses, preserve their own config mutations and manifests.

- [x] Unit 3: Tests in `tests/cli.rs` and `src/commands/tests/sync.rs` (~80 LOC)
  - [x] In `tests/cli.rs`, add integration test installing only `claude` (no opencode), running `sync`, asserting exit 0, `claude: registered`, and absence of `opencode: verified`.
  - [x] In `tests/cli.rs`, add integration test installing only `claude`, running `upgrade --source ...`, asserting exit 0.
  - [x] In `tests/cli.rs`, add test running `sync` with no harnesses installed, asserting exit 1 and `no harnesses installed`.

- [x] Unit 4: Documentation Update in `docs/user-guide/sync-and-upgrade-mechanisms.md` (~15 LOC)
  - [x] Clarify what `sync` verifies across native companions vs managed file trees vs adopted surfaces.
