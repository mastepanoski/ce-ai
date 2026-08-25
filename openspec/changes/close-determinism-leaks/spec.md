# Spec: Close Determinism Leaks 1 & 2

## Requirement: Deterministic source resolution

The system SHALL resolve plugin sources only from immutable release tags or
user-supplied local paths, and SHALL NOT download from a mutable branch as an
implicit fallback.

### Scenario: Network failure during release resolution

- **WHEN** the GitHub API query fails at the transport layer
- **THEN** `install`/`upgrade` exit with code 5 (`Network`)
- **AND** no tarball is downloaded
- **AND** the error message names the pinned alternatives (`--to <tag>`,
  `--source <path>`).

### Scenario: HTTP failure during release resolution

- **WHEN** the GitHub API responds with a non-success status
- **THEN** the command exits with code 5 (`Network`)
- **AND** no fallback URL is fetched.

### Scenario: Unparseable release payload

- **WHEN** the API response body cannot be parsed as JSON
- **THEN** the command exits with code 5 (`Network`).

### Scenario: No matching releases exist

- **WHEN** the API answers successfully but contains zero
  `compound-engineering-v*` tags
- **THEN** the command exits with code 2 (`Usage`)
- **AND** the message directs the user to `--to <tag>` or `--source <path>`.

### Scenario: Pinned resolution helper is total

- **WHEN** `pinned_version_and_url` receives `Some(tag)`
- **THEN** it returns `(tag, tag_tarball_url(tag))`
- **WHEN** it receives `None`
- **THEN** it returns `CeError::Usage`.

## Requirement: Byte-stable skill resolution output

`SkillRegistry::resolve` SHALL produce byte-identical markdown for identical
registry state and query, independent of wall-clock time.

### Scenario: Repeated resolution

- **WHEN** `resolve` is invoked twice with the same registry and query
- **THEN** both invocations return identical markdown strings.

### Scenario: Status tag preserved

- **WHEN** resolution succeeds without degradation
- **THEN** the markdown header contains `status=paths-injected`
- **AND** the header contains no timestamp field.

## Acceptance Criteria

- [ ] No references to `main_tarball_url` remain in `src/`.
- [ ] Unit tests cover both `pinned_version_and_url` branches and markdown
      byte-stability.
- [ ] `docs/user-guide/sync-and-upgrade-mechanisms.md` matches the new
      failure contract.
- [ ] `docs/user-guide/determinism-explained.md` exists with exactly one
      Diátaxis intent (Explanation) and Beginner audience labeling.
- [ ] All quality gates pass (fmt, clippy `-D warnings`, test, e2e).
