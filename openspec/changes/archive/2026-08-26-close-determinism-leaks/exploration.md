# Exploration: Close Determinism Leaks 1 & 2

## Investigation

Audit evidence (file:line at time of audit):

| Leak | Location | Behavior |
| --- | --- | --- |
| Silent `main` fallback | `src/source/release.rs:94-140` | Every failure mode (`send()`, HTTP status, body read, JSON parse) prints a stderr notice and returns `Ok(None)` |
| Fallback consumption | `src/commands/upgrade.rs:58-62`, `src/commands/install.rs:423-427` | `None => ("main".to_string(), main_tarball_url())` — unversioned, mutable source |
| Timestamp in output | `src/source/registry.rs:247-253` | `chrono::Utc::now().to_rfc3339()` interpolated into the generated markdown header |

Callers of `resolve_latest_release`: exactly two (`upgrade.rs`, `install.rs`),
both following the same `None → main` pattern. `main_tarball_url` has no other
callers, so removing the fallback orphans the function (clippy `-D warnings`
would reject it as dead code).

Consumers of the markdown: `src/commands/skills.rs:126` prints it to stdout;
the integration test `tests/cli.rs:1787` asserts only the stable prefix
`<!-- ce-ai:skill_resolution status=`, not the timestamp.

## Evaluated Options

### Leak 1

1. **Pin by default** — require `--to <tag>` always. Rejected: breaks the
   one-command quick path and duplicates what provenance already records after
   the first fetch.
2. **Fail loudly, never fall back** — keep latest-tag resolution as a
   convenience, but any resolution failure is an error and a zero-release API
   answer is a usage error. Chosen: removes the only nondeterministic download
   target while preserving UX; every successful run still pins its tag +
   SHA256 into `state.json` provenance.
3. **Opt-in flag `--allow-main`** — rejected: keeps a moving-target code path
   alive for no reproducibility benefit.

### Leak 2

1. **Drop the whole header comment** — rejected: `status=` is consumed by
   callers/tests as the machine-readable degradation signal.
2. **Remove only `timestamp=`** — chosen: keeps the functional status tag,
   makes output byte-stable. Wall-clock metadata belongs in logs, not in
   content that agents ingest.
