# Specification: Zero-Friction Release Resolution

## Formal Requirements

### Requirement 1: Web Redirect Fallback
- **WHEN** `resolve_latest_release` is invoked without a token or when GitHub REST API returns `403 Forbidden` / `429 Too Many Requests`
- **THEN** it MUST query `https://github.com/everyinc/compound-engineering-plugin/releases/latest` and extract the target tag from the resolved redirect URL.

### Requirement 2: Atom Feed Fallback
- **WHEN** Web Redirect fails or resolves to a non-CE tag
- **THEN** it MUST query `https://github.com/everyinc/compound-engineering-plugin/releases.atom` and parse the latest `compound-engineering-v*` release entry.

### Requirement 3: SemVer Validation & Invariance
- **WHEN** any release resolver finds a tag
- **THEN** the tag MUST match `compound-engineering-v*` and resolve to an immutable pinned release tarball URL.

### Requirement 4: User-Facing Friction Elimination
- **WHEN** a user without GitHub tokens or `gh` CLI runs `ce-ai upgrade` or `ce-ai install`
- **THEN** the command MUST resolve the latest version and succeed without prompting for credentials.
