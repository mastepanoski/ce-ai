> STATUS (v1.20.1): Update check live in the upgrade flow and state. Residual open boxes below were not re-audited item-by-item.

# OpenSpec Tasks: Version Update Checker

- [ ] **Unit 1: Upstream Release Resolver & State Caching (`src/source/release.rs` & `src/state/state.rs`)**
  - [ ] Add `latest_release_tag` to `State`.
  - [ ] Implement `check_latest_release_tag` with caching in `release.rs`.
- [ ] **Unit 2: Status Command & TUI Integration (`src/commands/status.rs` & `src/tui.rs`)**
  - [ ] Display latest upstream release tag in `ce-ai status`.
  - [ ] Display recommendation prompt when `source: local` or older release detected.
  - [ ] Render update status badge in TUI `Status & Harnesses` tab.
- [ ] **Unit 3: Integration Tests (`tests/cli.rs`)**
  - [ ] Test status command release tag reporting and local source upgrade recommendation.
