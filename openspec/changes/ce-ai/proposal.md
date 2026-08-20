# Proposal: ce-ai — CE Plugin Manager CLI

## Intent

Provide a single Rust CLI, `ce-ai`, that installs, syncs, upgrades, and configures the `compound-engineering-plugin` (CE) for AI coding harnesses. Today users must run harness-specific install steps and manually edit config files to assign models to subagents. `ce-ai` automates that toil, starting with OpenCode, using a `gentle-ai`-style state file and reproducible profiles.

## Scope

### In Scope
- OpenCode-only install, sync, upgrade, models, status, uninstall, doctor.
- Fetch CE source from GitHub releases by default; cache under `~/.ce-ai/cache`; `--source <local path>` override.
- Canonical state: `~/.ce-ai/state.json` + `~/.ce-ai/profiles/*.json` + append-only snapshots under `~/.ce-ai/profiles/versions/`.
- Model assignments for OpenCode subagents (`agent.<name>.model` + `variant`) in `~/.config/opencode/opencode.json`.
- Functional Docker E2E gate with fresh HOME and opencode installed.

### Out of Scope
- Codex, Pi, AGY, Claude, Cursor, Kimi installs in v1.
- Managing CE companion packages (`pi-subagents`, etc.).
- TUI (CLI only in v1).
- Multi-harness model writes (OpenCode only in v1).

## Capabilities

### New Capabilities
- `ce-source-fetching`: discover, download, and cache CE release tarballs; fallback to pinned ref tarball; local path override.
- `opencode-install`: direct file writes for OpenCode (`opencode.json` plugin entry + plugin loader + managed files).
- `sync-upgrade`: compute desired manifest from cached CE source, diff against installed files and `install-manifest.json`, repair/drift-detect, and upgrade by fetching latest release first.
- `models-management`: persist model assignments in `state.json`, apply to OpenCode `agent.<name>.model`/`variant`, and manage named profiles with append-only snapshots.
- `cli-commands`: `install`, `sync`, `upgrade`, `models`, `status`, `uninstall`, `doctor` with `--dry-run`, `--config-dir`, `--verbose`/`--quiet`.
- `e2e-docker-gate`: Rust integration test that builds a Docker image with opencode and asserts real install → sync → model config flow.

### Modified Capabilities
- None.

## Approach

Hybrid install strategy: OpenCode has no native install command, so `ce-ai` writes the plugin entry in `opencode.json`, copies the CE plugin loader, and registers skills/agents directly. `sync` reconciles the desired CE manifest against the installed manifest and filesystem. `upgrade` fetches the latest GitHub release, caches it, then runs `sync`. Models are kept in `~/.ce-ai/state.json` and flushed to OpenCode config; profiles/snapshots enable rollback and sharing.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/main.rs` | New | CLI entry point with clap subcommands. |
| `src/commands/` | New | install, sync, upgrade, models, status, uninstall, doctor modules. |
| `src/opencode/` | New | OpenCode config merge, plugin loader placement, manifest writer. |
| `src/source/` | New | GitHub release/tag fetching, tarball cache, local source override. |
| `src/state/` | New | `state.json`, profiles, snapshots, install-manifest I/O. |
| `tests/e2e.rs` | New | Docker-based E2E test gate. |
| `Makefile`/`Cargo.toml` | Modified | E2E target and test dependencies. |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| OpenCode config format changes | Med | Pin supported CE release in tests; treat format drift as breaking change. |
| E2E flakiness from network/npm | Med | Cache CE tarball; support `--source` local path; Docker E2E uses offline assertions after install. |
| User config overwritten | Low | Backup `opencode.json` and managed files before mutation; dry-run by default awareness. |
| Release tag naming ambiguity | Low | Prefer latest GitHub release asset; fallback to `main` or pinned ref if release metadata missing. |

## Rollback Plan

- Backup `opencode.json` and any CE-managed file before writing; backups live in `~/.ce-ai/backups/<timestamp>/`.
- `uninstall` reads `install-manifest.json` and restores the pre-install backup of `opencode.json`, removes plugin loader, skills, agents, and managed blocks.
- `sync` supports `--dry-run` so users can preview changes.
- State and profiles are append-only; snapshots allow manual reversion via `ce-ai models profile load <name>`.

## Dependencies

- `docker` for E2E gate.
- Network access to GitHub releases (or `--source` local path).
- OpenCode installed via npm for E2E (`opencode-ai`).

## Success Criteria

- [ ] `cargo test` passes unit/integration tests.
- [ ] Docker E2E passes: install CE into OpenCode, sync, set model for a subagent, verify `opencode.json` and plugin loader.
- [ ] `--dry-run` shows exact file mutations without writing.
- [ ] `uninstall` restores pre-install state and removes managed files.
- [ ] `state.json` + profiles + snapshots persist and round-trip.

## Open Items / Proposal Question Round

Assumptions needing confirmation before spec/design:
1. Release source: default to latest GitHub release tarball; if release API/tag missing, fallback to `https://github.com/everyinc/compound-engineering-plugin/archive/refs/heads/main.tar.gz` (or a pinned ref).
2. Tag naming: observed tags include `v2.x.x` (git) and `compound-engineering-v3.x.x` (GitHub releases). The implementation will resolve the latest release via GitHub API/asset, not raw git tags.
3. OpenCode plugin loader: CE ships `.opencode/plugins/compound-engineering.js`; `ce-ai` will place it under the user’s OpenCode config/plugins directory. Exact destination and skill registration path to be finalized in design.
4. Merge semantics for `opencode.json`: how to add/remove the CE plugin entry and agent model keys without clobbering unrelated user config.
5. Backup retention: keep last N backups or all backups; default to keep all.
