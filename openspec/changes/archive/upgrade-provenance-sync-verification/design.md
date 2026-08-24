# Design: Upgrade Provenance Binding & Honest Sync Verification

## Data Schemas

### `ReleaseProvenance` (`src/state/state.rs`)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseProvenance {
    /// GitHub release tag (e.g. "v1.2.3") or "main" for the SF-2 fallback.
    pub tag: String,
    /// Archive download URL that produced the cached artifact.
    pub url: String,
    /// Lowercase hex SHA256 of the cached tarball bytes.
    pub archive_sha256: String,
    /// Extracted source tree root used for the sync
    /// (`<config>/cache/trees/<safe_tag>`; temp dir under --dry-run).
    pub extraction_path: PathBuf,
}
```

`State` gains:

```rust
#[serde(default, skip_serializing_if = "Option::is_none")]
pub release_provenance: Option<ReleaseProvenance>,
```

Old `state.json` files load unchanged (serde default).

## Interfaces

### `src/source/cache.rs`

- `Cache::cache_tarball(&self, bytes, state_path)` → returns
  `(PathBuf, String)` (path, lowercase-hex digest). **No state write anymore**;
  callers record digest + provenance together.

### New helper `src/source/cache.rs`

```rust
pub(crate) fn record_tarball_provenance(
    state_path: &Path, provenance: ReleaseProvenance,
) -> Result<(), CeError>
```
Single atomic write: sets `managed_asset_digest["tarball"] = sha256:<hex>`
(from `provenance.archive_sha256`) and `release_provenance = Some(prov)`, then
`State::save` (temp+rename). Shared by `upgrade` and `install`.

### `upgrade::run` flows

- **Default (latest release)**: resolve tag/url → download → cache → extract →
  `record_tarball_provenance(tag, url, sha, root)` → sync.
- **`--source <path>`**: untouched (local tree, no release provenance).
- **`--to <tag>`**: replaced `cached_tarball()` with
  `cached_tarball_for(ctx, requested_tag)`:
  1. Missing provenance → `Usage`: run plain upgrade first to fetch.
  2. `prov.tag != requested` → `Usage` naming both tags and remediation;
     nothing recorded.
  3. Re-read cached file bytes; `sha256_hex(actual) != prov.archive_sha256` →
     `Verification` error naming expected vs actual digest (fail closed).
  4. Cross-check `managed_asset_digest["tarball"]` consistency; mismatch →
     `Verification`.
  5. Only then extract + `sync_from_extracted`.

### `src/error.rs`

```rust
Verification(String),   // Display: "verification error: {msg}"
// exit_code(): 6
```

### Honest verification matrix (`src/commands/sync.rs`)

```rust
pub(crate) struct SurfaceCheck {
    pub harness: String,
    pub status: CheckStatus,
}
pub(crate) enum CheckStatus {
    Verified { matched: usize, total: usize },
    Failed { mismatches: Vec<String>, missing: Vec<String> },
    NotVerified { reason: &'static str },
}
pub(crate) fn verify_tree_against(
    root: &Path, expected: &BTreeMap<String, String>,
) -> Vec<String> // returns offending relative paths (missing or hash-mismatched)
```

Flow in `sync_with` after apply:
1. Build `desired` map (existing code).
2. OpenCode surface: `verify_tree_against(managed_dir, &desired)` → Verified /
   Failed.
3. Harness loop: when skills were copied for claude/codex/copilot/grok,
   verify copied tree against the desired `skills/*` subset → Verified/Failed;
   registration-only adapters → `NotVerified { reason }`; opencode itself
   inherits the managed-surface result.
4. Print matrix only from these results; final line computed from data, never
   hardcoded (e.g. `reconciliation status: 2 verified, 3 unverified, 0 failed`).
5. Any `Failed` → return `CeError::Verification("sync verification failed for
   '<harness>': ...")`.

## CLI Contract

- `ce-ai upgrade --to <tag>` — resolves only from a matching, integrity-checked
  cached release; otherwise exits non-zero without mutating state/manifest.
- `ce-ai upgrade --harness|-t`, `ce-ai upgrade --force|-f` — removed; clap
  unknown-argument error (exit 2).
- Exit codes: success 0; tag/cache misuse 2 (`Usage`); integrity/drift 6
  (`Verification`); other failures per existing mapping.

## Testing Strategy

Unit tests colocated (`#[cfg(test)]`):

| Test | Asserts |
| --- | --- |
| `state.rs`: provenance round-trip | field survives save/load atomically |
| `state.rs`: legacy state loads | file without the field loads (serde default) |
| `upgrade.rs`: tag mismatch fails closed | `--to vX` vs cached vY → `Usage`; state still records vY |
| `upgrade.rs`: tampered cache fails closed | mutated bytes → `Verification`, state untouched |
| `upgrade.rs`: matching tag resolves | happy path returns cached path |
| `upgrade.rs`: clap rejects dead flags | `try_parse_from` errors for `--harness`/`--force` |
| `sync.rs`: verify_tree_against | match / hash-mismatch / missing detection |
| `sync.rs`: matrix honesty | statuses derive from real checks; failure yields exit-6 error |

Manual/E2E: `make e2e` covers install→sync→upgrade in Docker.
