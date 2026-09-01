# Proposal: Zero-Friction Release Resolution with Web Redirect Fallback

## 1. Problem Statement
When running `ce-ai install` or `ce-ai upgrade` without an explicit GitHub Personal Access Token or authenticated GitHub CLI (`gh`), `ce-ai` queries the GitHub REST API (`https://api.github.com/repos/everyinc/compound-engineering-plugin/releases`). GitHub applies an aggressive rate limit of 60 requests per hour per IP address to unauthenticated REST API queries, resulting in `HTTP 403 Forbidden` (`CeError::Network`) errors.

Requiring end users to generate tokens, configure environment variables (`CE_AI_GITHUB_TOKEN`, `GITHUB_TOKEN`), or install and authenticate `gh` introduces significant user friction for what should be an instantaneous, seamless tool installation and update.

## 2. Scope & Boundaries
- **In-Scope**:
  - Implement a zero-friction fallback resolver in `src/source/release.rs` using public web release redirect `https://github.com/{PLUGIN_REPO}/releases/latest`.
  - Add Atom feed `https://github.com/{PLUGIN_REPO}/releases.atom` parsing as a secondary resilient fallback if web redirect does not resolve a `compound-engineering-v*` tag.
  - Transparent hierarchy: Use authenticated GitHub REST API if a token is present; on unauthenticated setups, network errors, or HTTP 403 / 429 rate limits, automatically fallback to the web redirect/feed resolver.
  - Update `doctor.rs` messages to clarify that release updates work frictionlessly out of the box without mandatory tokens.
  - Add comprehensive unit and integration tests.
  - Bump SemVer to `1.29.2` and update `CHANGELOG.md`.
- **Out-of-Scope**:
  - Changing tarball extraction, SHA256 manifest verification, or state diff calculation.

## 3. ISO / NIST Risk Evaluation
- **NIST AI RMF 1.0 & ISO 42001 (Transparency & Resilience)**: Ensures deterministic, reliable access to AI agent tooling without unnecessary administrative or credential management burdens.
- **ISO 27001 / 27002 (Integrity & Non-repudiation)**: Pinned release tags and cryptographic SHA256 verification of downloaded tarballs remain 100% enforced; mutable `main` branch downloads remain strictly forbidden.

## 4. Success Criteria
1. `resolve_latest_release()` resolves the latest `compound-engineering-v*` tag even when unauthenticated, without tokens, and when the REST API returns HTTP 403/429.
2. `ce-ai upgrade` and `ce-ai install` succeed with exit code 0 out of the box on machines with no GitHub credentials.
3. 100% test pass rate across unit, integration, and CI gates.
