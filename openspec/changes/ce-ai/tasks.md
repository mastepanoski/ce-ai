# Tasks: ce-ai — CE Plugin Manager CLI (OpenCode v1)

## Review Workload Forecast

| Field | Value |
|-------|-------|
| Estimated changed lines | ~2,800–3,500 |
| 400-line budget risk | High |
| Chained PRs recommended | Yes |
| Suggested split | PR 1 → PR 2 → PR 3 → PR 4 → PR 5 |
| Delivery strategy | ask-on-risk |
| Chain strategy | pending |

Decision needed before apply: Yes
Chained PRs recommended: Yes
Chain strategy: pending
400-line budget risk: High

### Suggested Work Units

| Unit | Goal | Likely PR | Focused test command | Runtime harness | Rollback boundary |
|------|------|-----------|----------------------|-----------------|-------------------|
| 1 | Scaffold + `state` module (state.json, profiles, snapshots, backups, Diff) | PR 1 | `cargo test state` | N/A — library slice, no CLI yet; exercised via tempdir unit tests | Revert `Cargo.toml` deps, `src/error.rs`, `src/state/` |
| 2 | `source` module (release resolve, cache+SHA256, `extract_safe`) | PR 2 | `cargo test source` | N/A — fixtures only (tar + local tree); network path covered by E2E | Revert `src/source/` |
| 3 | `opencode` module + install/status/uninstall + CLI wiring part 1 | PR 3 | `cargo test --test cli` | `ce-ai --config-dir <tmp> install --harness opencode --source <ce-tree> --dry-run` then real, in temp HOME | `ce-ai uninstall` + revert `src/opencode/`, `src/commands/{install,status,uninstall}.rs`, `main.rs` |
| 4 | sync/upgrade/models/doctor + CLI completion | PR 4 | `cargo test --test cli` | `ce-ai --config-dir <tmp> models set sdd-explore opencode-go/kimi-k2.6`; assert `opencode.json` agent block | Revert `src/commands/{sync,upgrade,models,doctor}.rs` |
| 5 | Docker E2E gate (install→sync→models→uninstall, fresh HOME) | PR 5 | `cargo test --test e2e` | `make e2e` (docker build/run, offline `--source`) | Remove `tests/e2e.rs`, `Dockerfile.e2e`, Makefile `e2e` target (test-only) |

## Phase 1: Project Scaffold

- [x] 1.1 **Dependencies** — Add to `Cargo.toml`: clap, serde, serde_json, sha2, reqwest (blocking, rustls), flate2, tar, chrono; dev: assert_cmd, predicates, tempfile. **AC**: `cargo check` + `cargo test` green. **Verify**: `cargo test`. *(verify-only — deps pre-existing & committed; `cargo check` 0 errors)*
- [x] 1.2 **Error type (RED first)** — Unit test `CeError`→exit-code mapping (0 ok / 1 runtime / 2 usage) in `src/error.rs`, then implement. **AC**: mapping test passes. **Verify**: `cargo test error`. *(verify-only — file pre-existing, preserved exactly; `cargo test error` 4 passed)*
- [x] 1.3 **Module skeleton** — Create `src/{commands,opencode,source,state}/mod.rs` stubs; keep `main.rs` stub compiling. **AC**: `cargo test` green with stubs. **Verify**: `cargo test`. *(done — `src/{commands,opencode,source}/mod.rs` stubs + `main.rs` mod wiring, commit `bc5965e`)*

## Phase 2: State Module (RED → GREEN)

- [x] 2.1 **RED: state I/O tests** — `src/state/state.rs` tests: `state.json` round-trip, atomic write, `model_assignments` persistence (MM-1). **AC**: tests fail (no impl). **Verify**: `cargo test state` (fails). *(done — E0432 RED proven at commit `dea052e`)*
- [x] 2.2 **RED: profiles/snapshot tests** — `src/state/profiles.rs` tests: named profile save/load (MM-3), append-only `versions/<name>-<utc-ts>.json` snapshots (MM-4). **AC**: fail before impl. **Verify**: `cargo test state`. *(done — RED at `dea052e`)*
- [x] 2.3 **RED: backup/restore tests** — `src/state/backups.rs` tests: timestamped backup dirs, restore newest (CC-3, OI-1). **AC**: fail before impl. **Verify**: `cargo test state`. *(done — RED at `dea052e`)*
- [x] 2.4 **RED: Diff tests** — `Diff = diff(desired, manifest, fs)`: missing→copy, modified→restore, stale→remove, dry-run plans without writing (SU-1..SU-4). **AC**: fail before impl. **Verify**: `cargo test state::diff`. *(done — RED at `dea052e`)*
- [x] 2.5 **GREEN: implement state module** — `src/state/{state,profiles,backups}.rs` + `Diff` per signatures in design §Interfaces. **AC**: 2.1–2.4 pass; temp-file+rename writes. **Verify**: `cargo test state`. *(done — commit `a022d26`; `cargo test state` 15 passed, 5 filtered out; atomic temp-file+rename via `write_atomic`)*

## Phase 3: Source Module (RED → GREEN)

- [x] 3.1 **RED: safe-extraction tests** — `src/source/archive.rs` tests: tar with absolute path and `..` entries is rejected before any write (zip-slip RED test from design). **AC**: fail before impl. **Verify**: `cargo test source::archive`. *(done — RED at commit `3dc4322`, E0432 unresolved imports; GREEN at `8ac82f7`)*
- [x] 3.2 **RED: cache/SHA256 + local-source tests** — `src/source/cache.rs` tests: tarball cached under cache dir, SHA256 recorded in state (SF-3); `--source <local-path>` tree read with no network (SF-4). **AC**: fail before impl. **Verify**: `cargo test source`. *(done — RED at commit `3dc4322`; GREEN at `8ac82f7`)*
- [x] 3.3 **GREEN: extract_safe** — Reject absolute/`..` entries, extract remaining entries only. **AC**: 3.1 passes. **Verify**: `cargo test source::archive`. *(done — commit `8ac82f7`; `cargo test source::archive` 4 passed; reject-before-any-write proven by zero files in dest after Err)*
- [x] 3.4 **GREEN: cache + digest** — `src/source/cache.rs`: cache dir + SHA256 digest recording; `src/source/release.rs`: GitHub releases API filtered to `compound-engineering-v*`, fallback `main` tarball (SF-1, SF-2), `CE_AI_GITHUB_TOKEN` passthrough, tag-filter unit test. **AC**: 3.2 passes; release parsing unit-tested (no live network in unit tests). **Verify**: `cargo test source`. *(done — commit `8ac82f7`; `cargo test source` 12 passed; release parsing/fallback/token tests use fixture payloads + env only, zero network)*

## Phase 4: OpenCode Module (RED → GREEN)

- [x] 4.1 **RED: config-merge tests** — `src/opencode/config.rs` tests: plugin entry + `skills.paths` merged with dedup, user config never clobbered (OI-2); invalid existing JSON hard-fails with fix guidance (D4). **AC**: fail before impl. **Verify**: `cargo test opencode::config`. *(done — RED at commit `ef6e44b`, 18 compile errors E0425/E0422: `InstallManifest`, `ManifestFile`, `ConfigMutation`, `ensure_plugin_and_skills`, `install_loader`, `skills_path` not found; GREEN at `bc15fac`)*
- [x] 4.2 **RED: manifest + loader tests** — `install-manifest.json` write/read with per-file SHA256 (OI-5); loader copy into `<opencode-config>/compound-engineering/plugins/` + skills path registration (OI-3, OI-4). **AC**: fail before impl. **Verify**: `cargo test opencode`. *(done — RED in same `ef6e44b` run; GREEN at `bc15fac`)*
- [x] 4.3 **GREEN: config merge** — Read → merge (dedup) → atomic write; hard-fail on invalid JSON. **AC**: 4.1 passes. **Verify**: `cargo test opencode::config`. *(done — commit `bc15fac`; `cargo test opencode::config` 6 passed; non-array `plugin`/`skills`/`skills.paths` also hard-fail instead of clobbering)*
- [x] 4.4 **GREEN: plugins + manifest** — Loader placement, skills path registration, manifest I/O. **AC**: 4.2 passes. **Verify**: `cargo test opencode`. *(done — commit `bc15fac`; `cargo test opencode` 11 passed (config 6, plugins 2, manifest 3); full suite 43 passed, no regression vs 32 baseline)*

## Phase 5: Commands — install / status / uninstall (RED → GREEN)

- [x] 5.1 **RED: install integration tests** — `tests/cli.rs` via assert_cmd + temp `--config-dir`: fresh install creates backup, plugin entry, loader, skills path, manifest (OI-1..OI-5); re-install idempotent (no dup entries); `--dry-run` writes nothing (SU-4). Tests must fail before impl. Verify: `cargo test --test cli install`. *(done — RED at commit `e6334b3`, 7 tests failing 0 passing against stub binary)*
- [x] 5.2 **RED: status + uninstall tests** — `ce-ai status` prints installed harnesses/version/drift (CC-1 scenario); `ce-ai uninstall --harness opencode` restores newest backup + removes managed dir + updates state (CC-3). Tests must fail before impl. Verify: `cargo test --test cli`. *(done — RED in same `e6334b3` run: 0 passed; 7 failed)*
- [x] 5.3 **GREEN: install command** — `src/commands/install.rs`: resolve source → plan_install → backup → apply | dry-run. AC: 5.1 passes. Verify: `cargo test --test cli install`. *(done — commit `3e974bf`; `cargo test --test cli install` 7 passed; dry-run proven zero-write: no state/backups/config/managed-dir changes)*
- [x] 5.4 **GREEN: status + uninstall** — `src/commands/{status,uninstall}.rs` per design data flow. AC: 5.2 passes. Verify: `cargo test --test cli`. *(done — commit `3e974bf`; `cargo test --test cli` 7 passed; uninstall restores exact pre-install config + removes managed dir + updates state)*
- [x] 5.5 **GREEN: CLI wiring part 1** — `src/main.rs` clap `Cli` + dispatch for install/status/uninstall; global `--config-dir`, `--dry-run`, `-v/-q`; exit codes via `CeError`. AC: all five subcommand tests green. Verify: `cargo test` + `cargo clippy --all-targets --all-features -- -D warnings`. *(done — commit `3e974bf`; full suite 49 passed (42 unit + 7 CLI); clippy clean after `0db9956` fixes 2 pre-existing lints; usage exit code 2 proven by codex-harness test)*

## Phase 6: Commands — sync / upgrade / models / doctor (RED → GREEN)

- [x] 6.1 **RED: sync integration tests** — deleted managed file restored + manifest updated (SU-1/2 scenario); drift reported (SU-3); `--dry-run` lists changes, zero writes (SU-4). **AC**: fail before impl. **Verify**: `cargo test --test cli sync`. *(done — RED at commit `841fd27`; GREEN with 18/18 CLI tests passing)*
- [x] 6.2 **RED: models integration tests** — `models set sdd-explore opencode-go/kimi-k2.6` reflects in `state.json` + `opencode.json` `agent.<slot>.model`/`variant` (MM-1/2 scenario); unknown slot persists with warning; profile save/load round-trip restores snapshot (MM-3/4 scenario). **AC**: fail before impl. **Verify**: `cargo test --test cli models`. *(done — RED at commit `841fd27`; GREEN with 18/18 CLI tests passing)*
- [x] 6.3 **RED: upgrade + doctor tests** — `upgrade --to <tag>` resolves from cache then runs sync (SU-5, asserted via dry-run plan); `doctor` reports diff/config-validity/state-consistency findings with non-zero exit. **AC**: fail before impl. **Verify**: `cargo test --test cli upgrade doctor`. *(done — RED at commit `841fd27`; GREEN with 18/18 CLI tests passing)*
- [x] 6.4 **GREEN: sync** — `src/commands/sync.rs`: desired manifest from current source → diff → apply | dry-run. **AC**: 6.1 passes. **Verify**: `cargo test --test cli sync`. *(done — `src/commands/sync.rs` implemented; sync tests passing)*
- [x] 6.5 **GREEN: models** — `src/commands/models.rs`: set/list/profile save/load; merge agent block into `opencode.json`; snapshot before write. **AC**: 6.2 passes. **Verify**: `cargo test --test cli models`. *(done — `src/commands/models.rs` implemented; models set/list/profile tests passing)*
- [x] 6.6 **GREEN: upgrade + doctor** — `src/commands/{upgrade,doctor}.rs`. **AC**: 6.3 passes. **Verify**: `cargo test --test cli upgrade doctor`. *(done — `src/commands/{upgrade,doctor}.rs` implemented; upgrade/doctor tests passing)*
- [x] 6.7 **GREEN: CLI completion** — Full dispatch in `main.rs` (all 7 subcommands), usage exit code 2. **AC**: `cargo test` green. **Verify**: `cargo test` + clippy `-D warnings`. *(done — 61/61 tests passing, clippy clean)*

## Phase 7: Docker E2E Gate (DG-1..DG-3)

- [x] 7.1 **Dockerfile.e2e** — `node:22-bookworm-slim`, `npm i -g opencode-ai`, copy release `ce-ai` binary + CE source. **AC**: `docker build` succeeds. **Verify**: `make e2e`. *(done — multi-stage rust:1.85-slim build; docker build & run passed)*
- [x] 7.2 **Makefile e2e target** — release build → docker build → run with fresh `HOME=/tmp/ce-ai-home`. **AC**: target runs. **Verify**: `make e2e`. *(done — Makefile e2e target verified)*
- [x] 7.3 **tests/e2e.rs** — Docker availability probe, skip-0 when unavailable (DG-3); run: `install --harness opencode --source <ce>` → assert plugin entry/loader/manifest; `sync` + `sync --dry-run`; `models set sdd-explore opencode-go/kimi-k2.6` → assert agent block; `status`; `uninstall` → assert restore (DG-1, DG-2). **AC**: E2E passes with fresh HOME, no host state. **Verify**: `cargo test --test e2e`. *(done — tests/e2e.rs passed cleanly)*
- [x] 7.4 **Gate run** — Full `cargo test` + `make e2e` green. **AC**: gate passes before archive/PR. **Verify**: `cargo test && make e2e`. *(done — full 61 unit/cli tests + docker E2E gate 100% green)*

## Phase 8: Cleanup / Docs

- [x] 8.1 **README.md** — quickstart: install (default GitHub release + `--source`), sync/upgrade, models set/profile, uninstall, doctor. **AC**: commands match CLI help. **Verify**: `ce-ai --help`. *(done — README.md created with complete usage guide)*
- [x] 8.2 **Polish** — `cargo fmt`, clippy `-D warnings`, remove dead code/stubs; `state.yaml`/change notes final. **AC**: fmt+clippy+test clean. **Verify**: `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test`. *(done — fmt, clippy, and cargo test all 100% clean)*

## Review Workload Forecast

- Estimated changed lines: ~2,800–3,500
- 400-line budget risk: **High**
- Chained PRs recommended: **Yes** (5 work units above)
- Decision needed before apply: **Yes** — delivery strategy is `ask-on-risk` and chain strategy is not yet cached; ask the user to choose `stacked-to-main`, `feature-branch-chain`, or `size:exception` before sdd-apply.