# Product-effectiveness audit — `ce-ai` v1.8.0

**Intent:** Reference.  
**Audit date:** 2026-08-23  
**Revision audited:** `5047df8f6e284ff8c7a0b8cb0ce4d1c11f6d7250` (`feat(audit)`)  
**Scope:** public CLI promises, implementation, automated tests, destructive paths, and local quality gates. This is a source and black-box audit; it is not a penetration test or an independent release-binary verification.

## Executive conclusion

`ce-ai` has useful OpenCode-focused primitives: atomic single-file writes, backup creation, manifest SHA-256 tracking, safe archive-path validation, and passing local unit/integration tests. It does **not** yet reliably deliver several material product promises. The most important mismatch is that the product markets installation and management across 12 harnesses, while the tested and materially implemented path is OpenCode.

Treat the current release as an **experimental OpenCode manager plus advisory utilities**, not as a verified multi-harness orchestrator, workflow FSM, persistent-context recovery system, or companion-tool installer. Do not use the current success messages as proof that those capabilities completed.

| Priority | Finding | User impact | Recommended release decision |
| --- | --- | --- | --- |
| P0 | `ce-ai tools install` reports completion without installing or registering anything. | False readiness and wasted remediation time. | Remove the success claim or implement it before advertising it. |
| P0 | Non-OpenCode installation writes a synthetic config under the OpenCode directory; non-OpenCode uninstall only removes state. | The advertised harness is not configured; misleading “cleanly” success can leave files behind. | Restrict the product claim to OpenCode until harness-specific adapters exist. |
| P1 | `sync --watch` exits after one pass. | Drift is never continuously detected or repaired. | Implement a real watcher or remove the flag. |
| P1 | Global `--dry-run` is violated by workflow checkpoint; remote-source install/upgrade can cache and mutate state. | A preview can change user data. | Make dry-run a command-wide side-effect contract and test it. |
| P1 | Upgrade, sync, and workflow success messages overstate verification and recovery. | Incorrect version provenance and false operational confidence. | Make claims evidence-based and add end-to-end acceptance tests. |

## Method and evidence

The audit used CodeGraph for call-flow and impact analysis, direct source review, and isolated temporary-home executions. No production configuration was modified.

| Evidence | Result | Interpretation |
| --- | --- | --- |
| `cargo fmt --check` | Pass | Formatting gate passes. |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass | No Clippy diagnostics under the configured feature set. |
| `cargo test` | 243 passed, 1 ignored | Unit and integration suite passes, but passing tests do not cover several promised behaviours below. |
| `make e2e` | Pass with Docker test skipped | The test intentionally reports success when Docker is unavailable, so the container scenario was **not executed** on this host. |
| `cargo audit` | Not run locally: subcommand is absent | CI installs and runs it; local dependency-vulnerability status was not independently verified. |
| Isolated Cursor install/uninstall | Reproduced | `install --harness cursor` created `~/.config/opencode/.cursorrules` as JSON, not `~/.cursorrules`; uninstall printed success, removed state, and left that synthetic file. |
| Isolated `sync --watch` | Reproduced | Returned in 0 seconds after an initial sync while printing “watching”. |
| Isolated `--dry-run workflow checkpoint` | Reproduced | Created `state.json` and recorded the checkpoint. |

Severity combines exploitability, destructive potential, product-trust impact, and likelihood: **P0** must be fixed or de-scoped before relying on the feature; **P1** should be fixed in the next corrective release; **P2** is important quality debt.

## Findings and mitigations

### P0 — Companion-tool installation is a no-op with a success message

**Claim:** `ce-ai tools install <engram|codegraph|context7|rtk>` provisions a companion tool and completes MCP registration.

**Evidence:** `src/commands/tools.rs:82-96` only validates one of four names and prints two success lines. It neither invokes an installer nor writes a configuration file nor verifies a process/MCP registration. The only test (`src/commands/tools.rs:102-113`) checks that `tools status` does not panic.

**Risk:** Users can act on an explicit completion message although the sidecar is still unavailable. This is especially harmful because `tools status`, `doctor`, and `audit` use the same tools to infer readiness.

**Mitigation:**

1. Immediately change the command to an explicit advisory that returns non-zero, or hide it from the public command surface.
2. Define per-tool idempotent installers or documented hand-off commands; preserve existing user MCP entries through structured merges.
3. Verify the postcondition after installation (binary/version or a health probe) and return a precise `CeError` on failure.
4. Add hermetic tests for successful registration, pre-existing user configuration, installer failure, and idempotence.

**Acceptance evidence:** executing each supported installation in a temporary home produces the expected managed configuration and a capability probe passes; a failed installer cannot produce a success message.

### P0 — “12 harnesses” is a registry, not 12 working harness integrations

**Claim:** README line 3 says the CLI orchestrates the plugin across 12 harnesses. `install`, `sync`, `models`, `status`, and `uninstall` expose harness names beyond OpenCode.

**Evidence:**

- `src/commands/install.rs:99-174` bases every target path on `ctx.opencode_config_dir`, copies the OpenCode plugin layout, and applies the OpenCode JSON schema (`plugin`, `skills.paths`) to every target.
- `src/harness/mod.rs:247-261` calculates names such as `.cursorrules`, but they are appended to the OpenCode configuration directory rather than resolved from each adapter’s actual host location.
- `src/commands/uninstall.rs:56-74` performs filesystem restore/removal only when the selected target equals `opencode`; for every other harness it removes the state entry and prints a clean-uninstall success message.
- In the isolated Cursor execution, no native `~/.cursorrules` existed; a JSON file was created at `~/.config/opencode/.cursorrules`. A later `uninstall --harness cursor --yes` left that file behind.
- Integration tests exercise OpenCode installation. There is no install/sync/uninstall round-trip for every claimed harness.

**Risk:** Incorrect configuration placement, stale artifacts, and false completion. This also makes `--harness all` materially unsafe as a product promise: it can create multiple synthetic files in one unrelated directory.

**Mitigation:**

1. Short term: advertise and permit only `opencode`; reject unsupported harnesses with an actionable usage error. Do not list them as operationally supported.
2. For each additional harness, implement a dedicated adapter with its own native location, format-aware mutation (JSON vs Markdown/rules), managed-asset ownership model, backup/restore, and probe.
3. Change state/manifest schemas to store per-harness configuration and managed asset locations. A single OpenCode manifest cannot verify all harnesses.
4. Require a parameterized integration matrix for every enabled harness: install, reinstall, dry-run, drift/sync, backup, uninstall, and user-content preservation.

**Acceptance evidence:** a test creates each harness’s native configuration path, verifies that only its documented syntax is written, and proves uninstall restores byte-for-byte user content and removes only ce-ai-owned assets.

### P1 — `sync --watch` does not watch

**Claim:** `sync --watch` “continuously re-sync[s] upon drift” (`src/commands/sync.rs:24-27`).

**Evidence:** `src/commands/sync.rs:39-49` runs one `sync_with`, prints a “watching” message, and returns `Ok(())`; no watcher, polling loop, channel, or blocking wait exists. The integration test at `tests/cli.rs:1007-1018` only asserts the startup text. The isolated run returned in 0 seconds.

**Risk:** Users believe drift is protected when no background work occurs; a later modification is undetected.

**Mitigation:** Either remove `--watch` now or implement a long-running watcher with a documented debounce strategy, cancellation on Ctrl-C, recovery after a failed sync, and a test that changes a managed file after startup and observes restoration.

### P1 — The dry-run guarantee is not global

**Claim:** global `--dry-run` previews planned changes without writing; README presents it as a no-mutation operation.

**Evidence:**

- `src/commands/workflow.rs:89-93` saves state without inspecting `ctx.dry_run`. The isolated command `--dry-run workflow checkpoint` created `state.json` and stored its checkpoint.
- `src/commands/install.rs:239-267` and `src/commands/upgrade.rs:65-73` cache a downloaded tarball and update `state.json` before the command reaches its dry-run branch. `Cache::cache_tarball` writes both paths (`src/source/cache.rs:26-35`). Existing dry-run integration coverage uses `--source`, so it cannot observe the remote/cached-source path.

**Risk:** A supposedly safe preview creates state or cache files, invalidates reproducibility checks, and can overwrite `managed_asset_digest`.

**Mitigation:** Make `dry_run` an explicit policy at every mutating boundary. Resolve/download in memory or a temporary directory without caching, calculate the plan, and only persist cache/state after a non-dry run. Add a reusable test helper that snapshots the entire temporary home and config directory before every dry-run command.

### P1 — Workflow is a checkpoint string, not the claimed FSM/context recovery system

**Claim:** README and CLI describe a deterministic 7-stage FSM with zero-context-loss checkpointing; `workflow resume` says it uses Engram and OpenSpec state.

**Evidence:** `src/commands/workflow.rs:47-120` stores an arbitrary `--phase` and `--task` as a single formatted string in `State.last_update_check`. It validates neither the seven stages nor transitions. `resume_lines` only reprints `status_lines`; it has no Engram, OpenSpec, filesystem, or MCP call. No separate workflow state, transition history, context reference, or recovery protocol exists.

**Risk:** A user may infer process enforcement and recoverability where the command only displays text. Arbitrary input can label a checkpoint as any phase.

**Mitigation:** Choose one truthful contract:

- **Checkpoint utility:** rename/describe it as a lightweight local note, remove FSM/Engram/OpenSpec claims, and retain the current simple storage; or
- **Actual FSM:** define typed stages and legal transitions, record checkpoints separately from update metadata, attach durable OpenSpec and memory identifiers, validate recovery inputs, and expose a machine-readable state.

In either path, add tests for invalid transitions, dry-run, corruption, resumption evidence, and TUI/CLI behavioural parity.

### P1 — Upgrade and sync can report unverifiable or incorrect provenance

**Evidence:**

- `upgrade --to <tag>` does not fetch or select that tag. `src/commands/upgrade.rs:53-56` calls `cached_tarball`, which returns the one digest in state without a tag association, then records the caller-provided tag in the manifest/state.
- `Args.harness` and `Args.force` are defined (`src/commands/upgrade.rs:25-30`) but not used by `run`.
- `sync_with` only compares files in the shared OpenCode managed directory, then prints every detected harness as “synced & verified” and “100% Verified” (`src/commands/sync.rs:153-225`). It does not verify individual harness assets.

**Risk:** A cached release can be labelled as a different requested version, and users may trust a cross-harness verification that did not occur. This weakens supply-chain provenance and rollback decisions.

**Mitigation:** Persist `{tag, URL, archive SHA-256, extraction path}` together; bind `--to` to that exact cache entry or fetch and verify the requested release. Reject unused flags until implemented. Generate verification output only from checks that actually ran, per harness, with failures reflected in exit status. Add mismatch, cache-corruption, requested-tag, and multi-harness test cases.

### P1 — Destructive and cleanup errors are suppressed

**Evidence:** `src/commands/uninstall.rs:62-82` discards restore, removal, managed-directory, and registry-cleanup errors but still saves state and prints success. `src/commands/deinit_prj.rs:91-134` similarly discards several removal/read/atomic-write errors. `src/commands/init_prj.rs:269-282` silently ignores `.gitignore` and registry synchronization failures.

**Risk:** State may claim an operation completed while configuration remains partially modified. The user loses the ability to distinguish a successful cleanup from a failed one.

**Mitigation:** Classify operations as required vs best-effort. Propagate failures for required user-visible mutations; for explicitly optional cleanup, print a warning and preserve state for retry. Order changes so state is committed only after required filesystem work succeeds, or implement a recoverable journal/rollback. Add injected-permission/error tests for every destructive command.

### P1 — Exit-code contract does not match the documented project policy

**Evidence:** repository directives define `CeError` codes 0 through 6 for success, runtime, usage, state, I/O, network, and verification. `src/error.rs:10-70` implements only `Usage` (2), `Runtime`/I/O/JSON (1), and success (0). Consequently cache, network, verification, and state errors are not distinguishable to automation.

**Risk:** CI and scripts cannot reliably choose a remediation path. The policy’s assurance language is not enforceable by the binary.

**Mitigation:** Make the public error taxonomy the single source of truth: add explicit variants, map all boundary errors at their origin, document the table in CLI reference, and add CLI-level exit-code tests for every category.

### P2 — The new `audit` score measures configuration hints, not effectiveness

**Evidence:** `src/commands/audit.rs:135-485` treats file presence, a substring search, PATH-file existence, JSON object counts, and exact repeated paragraphs as successful capability evidence. Examples: an Engram database file means “server active”; any occurrence of `context7` in `opencode.json` means configured; `.codegraph/` means usable; and the RTK executable need not be run. The score equally weights all applicable checks, excludes `INFO`, has no confidence/provenance model, and has only two unit tests plus one output-shape integration test.

**Risk:** A high percentage (91% in the audited environment) can be read as effective token savings or context quality although no token measurement, live health check, freshness check, or task-quality benchmark ran. False positives and false negatives are likely, especially for prompt duplication and MCP schema variants.

**Mitigation:** Rename the score to “configuration coverage” until it has empirical meaning. Add detector confidence, raw evidence, schema-aware parsing, capability probes with timeouts, and an explicit “unknown” state. For an effectiveness score, measure before/after token use and task outcomes on a reproducible corpus; keep advisory configuration checks separate from performance claims.

### P2 — Single-file atomicity does not make multi-file commands transactional

**Evidence:** `state::write_atomic` protects individual writes, and archive path traversal has good reject-before-write coverage. Install and sync nevertheless update backups, managed files, configurations, manifests, registries, and `state.json` sequentially (`src/commands/install.rs`, `src/commands/sync.rs`) without a journal or rollback. No fault-injection tests cover a write failure between those steps.

**Risk:** A disk-full, permission, or process failure can leave a partial installation that the next command misinterprets.

**Mitigation:** Stage complete generations in a sibling temporary directory, validate them, then atomically switch a small pointer/manifest; or add an operation journal with deterministic recovery. At minimum, write the manifest/state last and make status/doctor diagnose incomplete operations explicitly.

## Test effectiveness assessment

| Area | What the suite proves | Important gap |
| --- | --- | --- |
| OpenCode install/sync/models/backup | Core local-source happy paths, JSON preservation, manifest hashes, and selected dry-runs. | Remote source, write failures, and restart/recovery paths. |
| Archive and skill registry | Parent/absolute archive path rejection; registry traversal/symlink tests. | Local source-tree symlink traversal and complete supply-chain provenance. |
| Multi-harness | Identifier/path parsing and selected status probes. | Native-format install, sync, backup, models, and uninstall for each harness. |
| Workflow/TUI | String rendering, checkpoint persistence, selected modal behaviour. | State-machine legality, dry-run, actual resume/context recovery, interactive TUI end-to-end. |
| Tools and audit | No-panic status, scoring arithmetic, and output shape. | Real installation, health probes, detector false-positive/false-negative matrices, JSON schema variants, threshold validation. |
| E2E and security | Docker scenario exists; CI declares a `cargo audit` gate. | This local audit could not run Docker or `cargo audit`; the e2e wrapper treats unavailable Docker as a pass. |

The suite is therefore useful as a regression suite for implemented OpenCode paths, but it is insufficient evidence for the broad capability claims.

## Remediation sequence

1. **Contain misleading behaviour (P0):** disable/de-scope `tools install` and non-OpenCode mutations; adjust help/README/status success language to match actual support.
2. **Restore operational safety (P1):** make dry-run pure; propagate required cleanup errors; remove or implement `sync --watch`; reject unused upgrade flags and unbound requested tags.
3. **Establish honest contracts:** decide whether workflow is a note/checkpoint utility or build the typed FSM and durable context integration it claims.
4. **Build harness adapters incrementally:** enable one harness only after its native-format contract and full lifecycle test matrix pass.
5. **Strengthen verification:** make Docker absence fail or explicitly mark the E2E gate skipped; install `cargo-audit` locally/CI as appropriate; add failure-injection and contract tests before feature work.
6. **Measure effectiveness separately:** evolve `ce-ai audit` from configuration coverage to evidence-backed metrics only after a benchmark design and corpus are agreed.

## Positive controls worth preserving

- `src/source/archive.rs` validates every archive entry before extraction and rejects absolute, parent, and drive-prefixed paths.
- `crate::state::write_atomic` is used for core JSON state/config/manifest writes.
- OpenCode merge logic preserves user plugin and skills entries and rejects malformed/config-shape conflicts instead of silently replacing them.
- Profile snapshots, local-source install/sync, and manifest SHA-256 drift detection have meaningful automated coverage.
- The binary forbids unsafe Rust (`src/main.rs:3`) and passes formatting, Clippy, and the available test suite.

## Suggested release gate for corrective work

Do not mark a corrective change complete until it includes an OpenSpec, a claim-to-test matrix, negative/failure tests, and all of:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
make e2e              # must execute Docker, not only skip
cargo audit
```

For every user-visible success message, the test should demonstrate the exact postcondition that message claims.
