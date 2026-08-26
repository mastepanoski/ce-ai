# Design: Close Determinism Leaks 1 & 2

## Contract Change (Leak 1)

`src/source/release.rs`:

- `resolve_latest_release` maps every transport/HTTP/parse failure to
  `CeError::Network` (exit code 5) with actionable guidance in the message.
  `Ok(None)` is now reserved for "API answered, zero `compound-engineering-v*`
  releases".
- New pure helper `pinned_version_and_url(tag: Option<String>) ->
  Result<(String, String), CeError>`:
  - `Some(tag)` → `(tag.clone(), tag_tarball_url(&tag))`
  - `None` → `Err(CeError::Usage(...))` instructing `--to <tag>` /
    `--source <path>`.
  Pure and unit-testable without network; shared by both callers.
- Delete `main_tarball_url` (no remaining callers) and update module docs that
  describe the SF-2 fallback.

`src/commands/upgrade.rs` / `src/commands/install.rs`: replace the
`None => ("main", main_tarball_url())` match arms with the helper (`?`).

## Byte-Stable Resolution Output (Leak 2)

`src/source/registry.rs::resolve`:

- Header becomes `<!-- ce-ai:skill_resolution status={status_tag} -->\n`.
- Remove the `chrono::Utc::now()` call (and the `chrono` import if it becomes
  unused in this file).
- Add unit test `resolve_markdown_is_byte_stable`: two consecutive `resolve`
  calls over the same registry produce identical markdown strings.

## Data Flow After Change

```
resolve_latest_release ──ok──▶ Some(tag) ─▶ pinned_version_and_url ─▶ tag tarball (immutable)
        │                          None ──▶ Usage error (no download)
        └──transport/http/parse──▶ Network error (no download)
```

Every successful install/upgrade therefore binds an immutable tag archive into
provenance `{tag, url, archive_sha256}` — reproducible afterwards via
`upgrade --to <tag>` + digest verification.

## Error Mapping

| Condition | Error | Exit code |
| --- | --- | --- |
| DNS/connection failure | `CeError::Network` | 5 |
| Non-success HTTP status | `CeError::Network` | 5 |
| Body read failure | `CeError::Network` | 5 |
| Release payload not JSON | `CeError::Network` | 5 |
| Zero matching CE releases | `CeError::Usage` | 2 |

## Documentation

- `docs/user-guide/sync-and-upgrade-mechanisms.md` Step 1 documents the new
  fail-loudly contract.
- New `docs/user-guide/determinism-explained.md` (Diátaxis: Explanation,
  Beginner) teaches what determinism means, what ce-ai guarantees on the asset
  layer, why LLM-driven workflow execution cannot be made deterministic by any
  technical means today, and the compensating controls. README map gains one
  row (stays ≤ 100 lines).
