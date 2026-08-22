# Exploration: Installer Download Resilience

## Options Evaluated

| Option | Assessment |
| :--- | :--- |
| Retry with backoff around resolve+download (**chosen**) | Minimal, fixes the observed failure window directly; no new dependencies. |
| Pin gate to a fixed previous tag | Hides real regressions; gate would stop testing latest. |
| Publish all platforms' assets before flipping `latest` (workflow change) | Correct long-term but touches release orchestration owned by concurrent sessions; higher blast radius. |

## Design Notes

- Backoff: 3 attempts, sleep 10s then 20s between attempts (~30s worst-case coverage, matching observed upload windows).
- After retries against `latest`, scan `/releases?per_page=5` for the most recent release carrying the exact asset name — covers `latest` pointing at a tag whose Windows asset uploads last.
- bash mirror keeps curl/wget duality; ps1 keeps curl.exe/HttpWebRequest duality.
