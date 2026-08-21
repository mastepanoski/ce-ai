# Implementation Plan: Multi-Harness Operations & TUI Target Scope

**Date**: 2026-08-21  
**Origin**: `docs/brainstorms/2026-08-21-multi-harness-operations-and-tui-ux-requirements.md`  
**OpenSpec Specifications**: `openspec/changes/multi_harness_operations/`  
**Target File**: `docs/plans/2026-08-21-multi-harness-operations-plan.md`  

---

## 1. Problem Statement & Scope Boundary

`ce-ai` detects up to 12 AI coding agent harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`). However, subcommands such as `sync` and `upgrade` currently operate on single or hardcoded paths, making it ambiguous whether updates apply to all installed harnesses. Furthermore, upgrading a harness installed from local source (`source: local`) could inadvertently overwrite local development trees.

### In Scope
- **MH-1 (Multi-Harness Sync & Upgrade)**: Support `--harness all` (or defaulting to all installed host harnesses) in `ce-ai sync` and `ce-ai upgrade` with itemized progress reporting.
- **MH-2 (Local Source Upgrade Protection)**: Skip or require `--force` when upgrading harnesses installed from `source: local` to prevent overwriting local dev trees.
- **MH-3 (TUI Global Target Scope Selector)**: Add a global `Target Harness: [ All Installed / <harness> ]` selector in the TUI header and action panels.
- **Plugin Non-Interference Guarantee**: Ensure harness-specific plugin and skills registration (e.g. in `.claude.json`, `.pi/config.json`, `.cursorrules`, `opencode.json`, `antigravity.json`) does not conflict with existing user plugin declarations, custom skills, or MCP server configurations.

### Out of Scope
- Modifying third-party harness binary installers.

---

## 2. Requirements Traceability

- **MH-1**: Bulk multi-harness dispatch in `sync::run` and `upgrade::run` (see `openspec/changes/multi_harness_operations/spec.md`).
- **MH-2**: Protection guard for `source: local` installations in `upgrade::run` (see `openspec/changes/multi_harness_operations/spec.md`).
- **MH-3**: Global TUI target harness selector in `src/tui.rs` (see `openspec/changes/multi_harness_operations/spec.md`).

---

## 3. Technical Architecture & File Layout

```
src/
├── commands/
│   ├── sync.rs        # Support --harness all and bulk multi-harness sync
│   ├── upgrade.rs     # Add --force flag and local source upgrade protection
│   └── install.rs     # Target path resolution per harness
├── harness/
│   ├── mod.rs         # Multi-harness config pathing & CE probing
│   └── claude.rs      # Claude Code plugin/skills non-clobbering merger
├── tui.rs             # TUI global target harness selector & itemized output
└── tests/
    └── cli.rs         # Integration tests for multi-harness sync & upgrade
```

---

## 4. Implementation Units

### Unit 1: Local Source Protection & Upgrade Refinement (`src/commands/upgrade.rs`)
- **Goal**: Add `pub force: bool` (`-f, --force`) flag to `upgrade::Args` and inspect installation source per target harness.
- **Files**:
  - `src/commands/upgrade.rs`
- **Behavior**:
  - If `source == "local"`, skip release upgrade with protective warning: `"Skipping upgrade for harness 'X' (source: local). Pass --force to override."` unless `--force` is true.
  - Iterate over target harnesses (`all` vs specific harness) and perform upgrade for each active target.
- **Test Scenarios**:
  - `upgrade_local_source_without_force_is_skipped`: Verify local source harness is skipped.
  - `upgrade_local_source_with_force_upgrades`: Verify `--force` overrides local source protection.

### Unit 2: Bulk Multi-Harness Sync (`src/commands/sync.rs`)
- **Goal**: Support `--harness <name>` or `all` (default) in `sync::run`.
- **Files**:
  - `src/commands/sync.rs`
- **Behavior**:
  - Resolve target harnesses (all active installed harnesses if `all` or default).
  - Perform manifest drift calculation and file reconciliation for each active installed harness target.
  - Preserve user plugins, custom skills, and MCP configurations in JSON harness configs without clobbering.
- **Test Scenarios**:
  - `sync_all_harnesses_reconciles_drift_across_targets`: Verify sync reconciles drift on opencode, claude, pi, etc.

### Unit 3: TUI Global Target Scope Selector (`src/tui.rs`)
- **Goal**: Add global `selected_harness_target_idx` allowing selection of `All Installed` or any individual host harness.
- **Files**:
  - `src/tui.rs`
- **Behavior**:
  - Render `Target Harness: < [ All Installed / harness_name ] >` in TUI header and action panels.
  - Route `Install`, `Sync`, `Upgrade`, `Models`, `Uninstall`, `Backups` key events to execute over selected target scope and display itemized results per harness.
- **Test Scenarios**:
  - `tui_target_harness_navigation`: Verify `◄`/`►` and `h`/`l` switch target harness selection smoothly.

### Unit 4: CLI Integration Tests & Quality Gates (`tests/cli.rs`)
- **Goal**: End-to-end integration tests for multi-harness sync, upgrade, and local-source protection.
- **Files**:
  - `tests/cli.rs`
- **Verification**:
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `make e2e`
