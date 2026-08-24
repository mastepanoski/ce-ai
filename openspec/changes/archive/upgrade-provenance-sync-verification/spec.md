# OpenSpec Specification: Upgrade Provenance & Honest Sync Verification

## Specifications

### PB-1: Atomic Release Provenance Persistence
- **WHEN** `ce-ai upgrade` fetches a release tarball (default path),
- **THEN** it MUST persist `{tag, url, archive_sha256, extraction_path}` together with the `managed_asset_digest["tarball"]` entry in one atomic `state.json` write (temp file + rename).

### PB-2: Requested-Tag Binding
- **WHEN** `ce-ai upgrade --to <tag>` runs,
- **THEN** the cached artifact MUST be used only if recorded provenance `tag` equals `<tag>`;
- **AND** otherwise MUST fail with a usage error naming both tags and the remediation command, without mutating `state.json` or `install-manifest.json`.

### PB-3: Cache Integrity Gate
- **WHEN** a cached release tarball is resolved via `--to <tag>`,
- **THEN** its on-disk bytes MUST be re-hashed and compared to the recorded `archive_sha256`;
- **AND** any mismatch MUST abort with a verification error (exit code 6) reporting expected and actual digests, before extraction or sync.

### PB-4: No Silent Flags
- **WHEN** `ce-ai upgrade --harness|-t` or `ce-ai upgrade --force|-f` is invoked,
- **THEN** the CLI MUST reject the flag with a usage error (exit code 2) instead of accepting and ignoring it.

### HV-1: Verification Output Reflects Executed Checks
- **WHEN** `ce-ai sync` (or upgrade-triggered sync) completes without `--dry-run`,
- **THEN** each reported harness line MUST state only checks that actually ran: hash-verified surfaces report matched/total counts, surfaces not verified are explicitly labelled as such.

### HV-2: Failure Propagation
- **WHEN** any verified surface contains missing or hash-mismatched files after sync applies,
- **THEN** the command MUST return exit code 6 (`Verification`) listing the offending paths, instead of printing a success matrix.
