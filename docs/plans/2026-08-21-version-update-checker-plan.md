# Technical Implementation Plan: Version Update Checker & Recommendation (Issue #14)

**Date**: 2026-08-21  
**Origin**: `docs/brainstorms/2026-08-21-version-update-checker-requirements.md`  
**OpenSpec Specifications**: `openspec/changes/version_update_checker/`  

---

## 1. Problem Statement & Scope Boundary

When harnesses are installed from a local dev tree (`source: local`), `ce-ai status` and the TUI report `version: local`. Users cannot easily tell if their local installation is up-to-date or lagging behind the latest GitHub release of `compound-engineering-plugin`.

### In Scope
- **VC-1 (Upstream Release Tag Resolution)**: Probe GitHub releases API (or read cached tag in `state.json`) in `status` and TUI.
- **VC-2 (Local Source Version Transparency)**: Display `(source: local)` alongside the latest available GitHub release tag.
- **VC-3 (Upgrade Recommendation Prompt)**: Display `💡 Recommendation: Run 'ce-ai upgrade' to update to latest release vX.Y.Z.`

---

## 2. Technical Architecture & File Layout

```
src/
├── source/
│   └── release.rs     # Implement check_latest_release_tag with caching
├── state/
│   └── state.rs       # Add latest_release_tag field
├── commands/
│   └── status.rs      # Report latest upstream release & recommendation
├── tui.rs             # Render release badge & recommendation in TUI Status tab
└── tests/
    └── cli.rs         # Integration test for status recommendations
```

---

## 3. Implementation Units

### Unit 1: Upstream Release Resolver & State Caching (`src/source/release.rs` & `src/state/state.rs`)
- Add `pub latest_release_tag: Option<String>` to `State` in `src/state/state.rs`.
- Add `pub fn get_latest_release_summary` in `src/source/release.rs`.

### Unit 2: Status Command & TUI Integration (`src/commands/status.rs` & `src/tui.rs`)
- In `status::run`: Print `upstream: latest release tag is vX.Y.Z`. If `source: local`, print `recommendation: Run 'ce-ai upgrade' to update from local source to latest release.`
- In `tui.rs`: Render release summary badge in `Status & Harnesses` tab.

### Unit 3: Integration Tests (`tests/cli.rs`)
- Add integration test verifying status output includes upstream release tag information and recommendation prompts.
