# Proposal: Close Determinism Leaks 1 & 2

## Problem

The determinism audit (2026-08-24) identified two leaks inside the asset/state
layer that `ce-ai` claims to control:

1. **Silent moving-target fallback.** `install` and `upgrade` resolve the
   latest GitHub release at runtime, and on any network, HTTP, or payload
   failure they silently fall back to the `main` branch source tarball — a
   target that changes between minutes. Two runs of the same command can load
   materially different skills into every harness, distinguished only by a
   stderr notice.
2. **Wall-clock bytes in harness-facing output.** `SkillRegistry::resolve`
   embeds `Utc::now()` in the generated markdown header, so identical inputs
   produce different output bytes on every invocation.

## In Scope

- Convert the implicit `main` fallback into an explicit error with actionable
  guidance (`--to <tag>` / `--source <path>`).
- Remove the timestamp from the skill-resolution markdown so identical inputs
  yield byte-identical output.
- Unit tests proving both contracts.
- Update `docs/user-guide/sync-and-upgrade-mechanisms.md` to match the new
  failure contract.
- New beginner-level explanation doc:
  `docs/user-guide/determinism-explained.md`.

## Out of Scope

- Making LLM-driven workflow execution deterministic (impossible; documented
  instead).
- The mtime-based feature inference in `workflow resume` (leak 3), the
  resolution-time hashing TOCTOU window (leak 4), and wall-clock fields in
  `state.json` (metadata only). These are environment-relative by design and
  are covered by the new explanation doc.

## Risks

- Scripts that relied on offline installs succeeding via the `main` fallback
  will now fail loudly. Mitigation: the error message states the exact pinned
  alternatives (`--to`, `--source`) and the changelog records the behavior
  change.
- Consumers parsing the markdown header for `timestamp=` will no longer find
  it. Mitigation: the `status=` tag is unchanged and remains the machine field.

## Success Criteria

- No code path downloads from `main_tarball_url`; network/API failures map to
  `CeError::Network`, zero-release results to `CeError::Usage`.
- Two `skills resolve` invocations over an unchanged tree emit byte-identical
  markdown.
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo test`, and `make e2e` all pass.
