---
title: "Native Per-Harness Directory Resolution & Artifact Leakage Containment"
category: "bugfixes"
module: "src/harness/mod.rs"
tags: ["harness", "isolation", "bugfix", "multi-harness"]
problem_type: "bug"
severity: "P0"
applies_when: "When installing, updating, or syncing non-OpenCode harnesses where assets might leak into the OpenCode directory."
---

# Native Per-Harness Directory Resolution & Artifact Leakage Containment

## Problem
When installing, updating, or syncing non-OpenCode harnesses (e.g. `ce-ai install --harness cursor`), `ce-ai` previously hardcoded `target_base_dir` to `~/.config/opencode/`. This caused synthetic configuration files (`.cursorrules`, `claude.json`, `config.json`) and managed assets (`compound-engineering/`) to be created inside the **OpenCode** directory instead of the native host harness directory. Subsequent `uninstall` operations removed state entries while leaving stray synthetic files abandoned in `~/.config/opencode/`.

## Solution
1. **`HarnessKind::harness_dir(home_dir)`**: Introduced explicit native directory mapping for all 12 supported `HarnessKind` variants (`opencode`, `claude`, `cursor`, `pi`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`).
2. **Context Home Resolver (`home_dir_from_ctx`)**: Added robust context-aware `$HOME` resolution that gracefully respects both production configuration paths and hermetic `TempDir` test environments.
3. **Lifecycle Parity (`install`, `uninstall`, `sync`, `models set`)**: Updated all lifecycle commands to resolve `h_kind.harness_dir(&home_dir)` per target harness, eliminating cross-directory contamination.
4. **Integration Testing**: Added end-to-end CLI integration tests in `tests/cli.rs` (`install_cursor_harness_writes_to_native_dir_and_leaves_opencode_pristine`, `uninstall_cursor_harness_cleans_native_dir_artifacts`) confirming zero artifact leakage into `~/.config/opencode/`.

## Key Code Snippet

Principle: resolve via `HarnessKind::harness_dir`, never hardcode — every
lifecycle command (`install`, `uninstall`, `sync`, `models set`) asks the
kind for its native root, and most native roots honor vendor env-var
overrides (v1.48.0):

```rust
/// Returns the native configuration directory root for this harness relative to `home_dir`.
pub fn harness_dir(&self, home_dir: &Path) -> PathBuf {
    match self {
        HarnessKind::Opencode => home_dir.join(".config").join("opencode"),
        HarnessKind::Claude => std::env::var_os("CLAUDE_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join(".claude")),
        HarnessKind::Pi => std::env::var_os("PI_CODING_AGENT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join(".pi").join("agent")),
        HarnessKind::Cursor => home_dir.join(".cursor"),
        HarnessKind::Copilot => std::env::var_os("COPILOT_CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join(".copilot")),
        HarnessKind::Codex => std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join(".codex")),
        HarnessKind::Grok => std::env::var_os("GROK_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join(".grok")),
        HarnessKind::Kimi => std::env::var_os("KIMI_CODE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join(".kimi-code")),
        HarnessKind::Agy => crate::harness::agy::AgyAdapter.harness_dir(home_dir),
        HarnessKind::Deepseek => home_dir.join(".config").join("deepseek"),
        HarnessKind::Fx => std::env::var_os("FX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join(".fx")),
        // Single custom-mode contract: the config file lives directly
        // under ~/.ce-ai (see harness::custom::CONFIG_FILE_NAME).
        HarnessKind::Custom => home_dir.join(".ce-ai"),
    }
}
```
