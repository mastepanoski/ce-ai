# Exploration: Zero-Friction Release Resolution

## 1. Technical Investigation
GitHub provides multiple public endpoints for discovering releases:

1. **REST API (`https://api.github.com/repos/{repo}/releases`)**:
   - Pros: Structured JSON, lists all releases.
   - Cons: Rate limited to 60 req/h per IP when unauthenticated. Fails with 403 Forbidden.

2. **Web Release Redirect (`https://github.com/{repo}/releases/latest`)**:
   - Request: `GET` / `HEAD` with redirect handling.
   - GitHub returns a `302 Found` with `Location: https://github.com/{repo}/releases/tag/{tag_name}`.
   - Pros: Standard web page route, not subject to REST API rate limits, returns the latest published release tag immediately.
   - Extraction: Extract tag from redirect URL path `/releases/tag/(.*)`.

3. **Releases Atom Feed (`https://github.com/{repo}/releases.atom`)**:
   - Request: `GET https://github.com/{repo}/releases.atom`.
   - Returns standard XML/Atom feed of all releases.
   - Pros: Public XML feed, not rate-limited by REST API, contains full history of releases.
   - Extraction: Regex/string matching for `<link rel="alternate" type="text/html" href=".../releases/tag/(compound-engineering-v[^"]+)"/>` or `<title>compound-engineering: (v[^<]+)</title>`.

## 2. Tradeoff Analysis
| Approach | Reliability | Friction | Complexity | Rate Limit Resilience |
|---|---|---|---|---|
| REST API Only | Medium (needs token) | High (requires PAT/gh) | Low | Low (60 req/h) |
| Web Redirect + Atom Fallback | High | Zero (no auth needed) | Low | Very High (standard web/CDN) |
| Multi-Tier Strategy (Token API -> Web Redirect -> Atom Feed) | Highest | Zero | Medium | Highest |

## 3. Decision
Adopt the **Multi-Tier Strategy**:
1. If a token is provided (`CE_AI_GITHUB_TOKEN`, `GITHUB_TOKEN`, `GH_TOKEN`, or `gh auth token`), try GitHub REST API first (fast, structured).
2. If unauthenticated, or if the REST API returns `403 Forbidden`, `429 Too Many Requests`, or connection error, immediately fallback to Web Redirect (`/releases/latest`).
3. If the latest tag is not `compound-engineering-v*` (or if redirect fails), fallback to Atom feed (`/releases.atom`) to extract the latest matching `compound-engineering-v*` release.
4. If all methods fail, return clear error with manual pinning guidance (`--to <tag>`).
