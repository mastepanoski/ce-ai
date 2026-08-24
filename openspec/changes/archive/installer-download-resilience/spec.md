# Spec: Installer Download Resilience

## ADDED Requirements

### Scenario 1: Transient 404 during release publication
WHEN the resolved latest-release asset returns 404 on the first download attempt
THEN the installer retries up to 3 total attempts with linear backoff before failing.

### Scenario 2: latest lacks the platform asset
WHEN `/releases/latest` has no asset matching the installer's platform filename after retries
THEN the installer scans the 5 most recent releases and installs from the most recent one carrying the exact asset name.

### Scenario 3: Stable release unaffected
WHEN the latest release carries the expected asset
THEN the installer behavior is byte-for-byte equivalent to the previous single-attempt flow (same install directory, same success output).

### Scenario 4: Exhausted fallbacks fail loudly
WHEN all attempts and the recent-releases fallback fail
THEN the installer exits non-zero with an error naming the attempted URLs.

## Acceptance Criteria

- Windows PowerShell Installer Gate passes when run during an active release publication window or immediately after one.
- bash installer logic mirrors the ps1 retry/fallback semantics.
