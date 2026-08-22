# Backup & Uninstall

> **Intent**: How-to — back up, restore, and cleanly remove `ce-ai` and the Compound Engineering plugin without data loss. For installation, see the [Installation & Coexistence Guide](installation-and-coexistence-mechanisms.md).

`ce-ai` is built with a zero-data-loss guarantee for host harness configurations across all supported harnesses (see [Harness Matrix](harness-matrix.md)).

## Automatic Pre-Mutation Backups

Before any file write or configuration update during `install` or `models set`, `ce-ai`:

1. Checks whether pre-existing harness configs exist (e.g. `opencode.json`, `.claude.json`, `.cursorrules`).
2. Creates a timestamped backup copy inside `~/.ce-ai/backups/<utc-timestamp>/`.
3. Registers the backup path in the managed manifest (`install-manifest.json`).

## Atomic Uninstall (`ce-ai uninstall`)

Running `ce-ai uninstall --harness <name>`:

1. **Restores original config**: locates the latest backup in `~/.ce-ai/backups/` and atomically restores the target config to its exact pre-install content.
2. **Removes created files**: configuration entries or Markdown blocks created by `ce-ai install` are cleanly deleted or stripped.
3. **Purges managed directory**: deletes `<harness-config>/compound-engineering/` containing loaders, installed skills, and `install-manifest.json`.
4. **Cleans state**: updates `~/.ce-ai/state.json` to reflect that the harness is uninstalled.

```bash
ce-ai uninstall --harness claude
ce-ai uninstall --harness opencode
```

## Install Manifest (`install-manifest.json`)

Every managed harness directory contains `install-manifest.json` recording:

- Installed plugin version and source (release tag or local path).
- Per-file SHA256 hashes for drift detection.
- Config mutation log linking to the exact pre-install backup path.

## Model Profile Snapshots

- Saving a profile (`ce-ai models profile save <name>`) writes an append-only snapshot under `~/.ce-ai/profiles/versions/<name>-<timestamp>.json`.
- Loading a profile (`ce-ai models profile load <name>`) restores previous assignments while preserving historical snapshots.

## Complete Removal

To fully remove `ce-ai` from a machine:

```bash
# 1. Restore each installed harness config
ce-ai uninstall --harness claude
ce-ai uninstall --harness opencode   # repeat per installed harness

# 2. Remove the binary
cargo uninstall ce-ai

# 3. Purge cached tarballs, model profiles, and backups (optional)
rm -rf ~/.ce-ai
```
