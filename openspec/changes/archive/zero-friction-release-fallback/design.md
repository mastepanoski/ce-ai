# Design: Multi-Tier Zero-Friction Release Resolution

## 1. System Architecture

```
[resolve_latest_release]
       │
       ▼
[Check Token Presence]
  ├── (Token Present) ───────────► [Query GitHub REST API] ──► Success? ──► Return tag
  │                                       │ (403/429/Error)
  │                                       ▼
  └── (No Token / API Error) ──► [Query Web Redirect (/releases/latest)] ──► Success & valid CE tag? ──► Return tag
                                          │ (Redirect Error / Non-CE tag)
                                          ▼
                                 [Query Atom Feed (/releases.atom)] ──► Success & valid CE tag? ──► Return tag
                                          │ (All Failed)
                                          ▼
                                 [Return CeError::Network with Pinning Guidance]
```

## 2. Component Specifications

### `src/source/release.rs`

1. **`resolve_latest_release_from_web_redirect(client: &reqwest::blocking::Client) -> Result<Option<String>, CeError>`**:
   - Queries `https://github.com/{PLUGIN_REPO}/releases/latest` with redirect follow enabled.
   - Extracts the final URL path: if URL contains `/releases/tag/` and the trailing segment matches `compound-engineering-v*`, returns `Ok(Some(tag))`.

2. **`resolve_latest_release_from_atom_feed(client: &reqwest::blocking::Client) -> Result<Option<String>, CeError>`**:
   - Queries `https://github.com/{PLUGIN_REPO}/releases.atom`.
   - Parses XML/HTML tags in atom feed to extract release tags matching `compound-engineering-v*`.
   - Sorts using `compare_versions` and returns the newest semver tag.

3. **`resolve_latest_release(client: &reqwest::blocking::Client, token: Option<&str>) -> Result<Option<String>, CeError>`**:
   - If `token` is present:
     - Tries REST API first.
     - On 403, 429, or connection error, falls back to web redirect then atom feed.
   - If `token` is absent:
     - Tries REST API first; if 403, 429, or connection error, falls back to web redirect then atom feed.
     - If REST API returns 200 OK, uses payload.

4. **`doctor.rs`**:
   - Update `doctor` output:
     - If token is present: `doctor-info: github-token present (authenticated API quota)`
     - If token is absent: `doctor-info: unauthenticated mode (using resilient web release resolver)`
