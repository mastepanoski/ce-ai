# OpenSpec Design: Version Update Checker

## Data Schema & Struct Updates
- `State`: `last_update_check: Option<String>`, `latest_release_tag: Option<String>`.
- `release.rs`: `check_latest_release(client: &Client, token: Option<&str>) -> Option<String>`.
- `status.rs`: Append upstream release status and upgrade recommendation.
- `tui.rs`: Render upstream release status and recommendation badge in `Status & Harnesses` tab.
