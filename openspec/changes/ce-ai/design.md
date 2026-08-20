# Design: ce-ai — CE Plugin Manager CLI

## Technical Approach

Single Rust binary (edition 2021, clap derive) that manages the compound-engineering plugin for OpenCode v1. Install/sync/upgrade use **direct file writes** (OpenCode has no headless install command — exploration §A1) driven by a **desired-manifest vs installed-manifest + filesystem hash diff** (spec SU-1..SU-3, exploration §A2c). Source comes from GitHub releases (tags `compound-engineering-v*`) with SHA256-verified local cache and `--source` override (SF-1..SF-4). Model assignments live in gentle-ai-style `state.json` and are flushed to `opencode.json` `agent.<slot>.model`/`variant`; named profiles with append-only snapshots enable rollback (MM-1..MM-4). Every mutation is planned, backed up, dry-run capable, and atomically applied. A Docker E2E gate proves install → sync → models against a real opencode install in an isolated HOME (DG-1..DG-3).

## Architecture Decisions

| # | Decision | Alternatives | Choice / Rationale |
|---|----------|--------------|--------------------|
| D1 | Install strategy | Shell-out to harness CLIs (A); direct file writes only (B); hybrid (C) | **Direct file writes for OpenCode** — only scriptable path (no native cmd); deterministic; easy snapshot/diff/rollback. Shell-out deferred with other harnesses. |
| D2 | Plugin registration | Git-URL entry `compound-engineering@git+...` (CE `INSTALL.md`); local managed loader | **Local loader file** at `<opencode-config>/compound-engineering/plugins/compound-engineering.js` referenced by absolute path in `plugin[]`. Offline-safe, no network at opencode startup, precisely uninstallable; satisfies OI-3 and OI-4 via loader's `config.skills.paths` hook plus an explicit `skills.paths` merge. |
| D3 | Managed file location | `~/.ce-ai/` central store; harness-local managed dir | **Harness-local** `<opencode-config>/compound-engineering/` (mirrors CE converter `install-manifest.json` convention, exploration §A2) — keeps opencode self-contained and uninstall manifest-driven (CC-3). |
| D4 | Config merge | Warn-and-write (CE converter); fail on invalid JSON | **Read → merge (dedup) → atomic write; hard-fail on invalid existing JSON** with fix guidance. Never clobber user config (OI-2); writing over broken config would destroy data. |
| D5 | Sync model | Re-fetch+reinstall only (a); local reconcile only (b) | **(c) Both**: desired manifest from current source tree; diff vs manifest + per-file SHA256; actions = copy/restore/remove/update-manifest; `--dry-run` plans only (SU-1..SU-4). |
| D6 | Source resolution | Raw git tags; `main` tarball always | **GitHub releases API** filtered to `compound-engineering-v*` (proposal open item 2), fallback `main` tarball (SF-2), digest recorded (SF-3), `--source <path>` bypasses network (SF-4). |
| D7 | Backup policy | Keep last N; keep all | **Keep all** under `~/.ce-ai/backups/<utc-ts>/` (proposal item 5 default). |
| D8 | E2E gate | Host-run shell script; n8500x image | **Rust `tests/e2e.rs` spawning `docker build`/`run`** on `node:22-bookworm-slim` + `npm i -g opencode-ai`; skips (exit 0) if Docker unavailable (DG-3). Uses `--source` to stay offline. |

## Data Flow

```
install:  resolve_source(SF) → extract_safe → compute desired tree
          → plan(mutations) → [dry-run? print|backup→write] → write install-manifest → update state.json

sync:     load install-manifest + hash fs files → diff vs desired → plan → apply | dry-run
upgrade:  resolve latest release → fetch+cache → sync        (SU-5)
models:   state.json model_assignments → merge agent.<slot>.model/variant into opencode.json
          → save_profile_snapshot(beforeRaw, preview)         (MM-1..MM-4)
uninstall: manifest → restore newest opencode.json backup → remove managed dir → update state.json (CC-3)
doctor:   diff + config-validity + state-consistency checks, non-zero exit on findings
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/main.rs` | Modify | clap `Cli` + global flags (`--config-dir`, `--dry-run`, `-v/-q`), dispatch, exit codes |
| `src/error.rs` | Create | `CeError` enum, `ExitCode` mapping (0 ok / 1 runtime / 2 usage) |
| `src/commands/{mod,install,sync,upgrade,models,status,uninstall,doctor}.rs` | Create | One module per subcommand: Args struct + `run(ctx)` |
| `src/opencode/{mod,config,plugins,manifest}.rs` | Create | opencode.json merge/dedup, loader copy, install-manifest I/O, path resolution (`--scope global|workspace`) |
| `src/source/{mod,release,cache,archive}.rs` | Create | GitHub release resolution, tarball cache + SHA256, safe extraction, local-tree reader |
| `src/state/{mod,state,profiles,backups}.rs` | Create | state.json, profiles + snapshots, backup/restore |
| `tests/cli.rs` | Create | Integration tests: 7 command flows via `assert_cmd` + temp config-dir |
| `tests/e2e.rs` | Create | Docker E2E gate (DG-1..DG-3) |
| `Dockerfile.e2e` | Create | node:22-bookworm-slim + opencode-ai + ce-ai binary + CE source copy |
| `Makefile` | Create | `e2e` target (release build → docker build → run) |
| `Cargo.toml` | Modify | deps: clap, serde, serde_json, sha2, reqwest(blocking, rustls), flate2, tar, chrono; dev: assert_cmd, predicates, tempfile |

## Interfaces / Contracts

```jsonc
// ~/.ce-ai/state.json  (SF-3, MM-1)
{ "version": 1,
  "installed_harnesses": [{ "name": "opencode", "version": "compound-engineering-v3.4.2",
      "source": {"kind": "github-release", "tag": "…", "sha256": "…"} | {"kind": "local", "path": "…"},
      "installed_at": "…", "last_synced_at": "…" }],
  "managed_asset_digest": { "tarball": "sha256:…", "tree": "sha256:…" },
  "model_assignments": { "<slot>": { "provider_id": "…", "model_id": "…", "effort": "high" } },
  "last_update_check": "…" }

// ~/.ce-ai/profiles/<name>.json          (MM-3)
{ "name": "…", "created_at": "…", "models": { "<slot>": "provider/model" } }

// ~/.ce-ai/profiles/versions/<name>-<utc-ts>.json — append-only (MM-4)
{ "name": "…", "created_at": "…", "beforeRaw": { "<slot>": "provider/model" }, "preview": { … } }

// <opencode-config>/compound-engineering/install-manifest.json  (OI-5, SU-1/3)
{ "version": "compound-engineering-v3.4.2", "plugin_name": "compound-engineering",
  "installed_at": "…", "source": { … },
  "files": [{ "path": "plugins/compound-engineering.js", "sha256": "…" },
            { "path": "skills/ce-brainstorm/SKILL.md", "sha256": "…" }],
  "config_mutations": [{ "file": "opencode.json", "backup": "~/.ce-ai/backups/<ts>/opencode.json",
                         "keys": ["plugin", "skills.paths", "agent.*.model"] }] }
```

Core signatures: `source::resolve_source(&SourceSpec, &Cache) -> Result<SourceTree>`; `source::extract_safe(&Path, &Path) -> Result<()>` (rejects absolute/`..` entries — zip-slip); `opencode::plan_install(tree, cfg, dry_run) -> Plan`; `state::Diff = diff(desired, manifest, fs)`; all mutators run through `Plan::execute(backup=true, dry_run)` using temp-file+rename.

## Testing Strategy

| Layer | What | Approach / spec |
|-------|------|-----------------|
| Unit | merge dedup/no-clobber, invalid-JSON fail | `opencode::config` — OI-2 |
| Unit | diff logic (missing/modified/stale), dry-run writes nothing | `state::Diff` — SU-1..SU-4 |
| Unit | state/profiles/snapshot round-trip, backup→restore | `state::*` — MM-1, MM-4, CC-3 |
| Unit | sha256 verify, tar path-traversal rejection, local source | `source::*` — SF-3/4 + safety RED test |
| Integration | `tests/cli.rs`: install → status → sync → models set/list → uninstall; re-install idempotent; unknown-slot warn; exit codes | assert_cmd + temp config-dir — CC-1..CC-3, MM scenarios |
| E2E | `tests/e2e.rs`: docker available? else skip-0 (DG-3); build image, fresh `HOME=/tmp/ce-ai-home`, `install --source` → assert plugin entry/loader/manifest; `sync` + `sync --dry-run`; `models set sdd-explore opencode-go/kimi-k2.6` → assert agent block; `status`; `uninstall` → assert restore | DG-1..DG-3 |

## Threat Matrix

`N/A` for all rows — v1 ships no shell-out, git, or PR automation; OpenCode install is pure file I/O and GitHub fetch is HTTP-only:

| Boundary | Applicability | Reason |
|---|---|---|
| Documentation-like paths | N/A | No executable-markdown handling |
| Git repository selection | N/A | No git subprocess; source = HTTP tarball or local dir |
| Commit state | N/A | No git |
| Push state | N/A | No git |
| PR commands | N/A | No PR automation |

Real boundary handled in this design (not a matrix row, carried to tasks as RED tests): **untrusted-archive extraction** — `extract_safe` rejects absolute paths and `..` traversal before writing any entry; and **Docker subprocess in tests only** — availability probe + skip per DG-3.

## Migration / Rollout

No migration (new crate). Strict TDD via `cargo test`; `cargo clippy --all-targets --all-features -- -D warnings` clean; Docker E2E gate passes before archive/PR. Rollback: `ce-ai uninstall` (manifest + backup restore) and profile snapshots.

## Open Questions

- [ ] GitHub API is unauthenticated (60 req/hr) — add `CE_AI_GITHUB_TOKEN` env passthrough? (Proposed: yes, optional.)
- [ ] `variant` field: always write empty string (gentle-ai pattern) or only when set? (Proposed: always, mirroring gentle-ai.)
- [ ] Confirm E2E uses `--source` (offline) with a separate opt-in network job for the releases path.