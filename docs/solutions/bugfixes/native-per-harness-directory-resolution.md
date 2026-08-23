---
title: "Native Per-Harness Directory Resolution & Artifact Leakage Containment"
category: "bugfixes"
module: "src/harness/mod.rs"
tags: ["harness", "isolation", "bugfix", "multi-harness"]
problem_type: "bug"
severity: "P0"
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
```rust
/// Returns the native configuration directory root for this harness relative to `home_dir`.
pub fn harness_dir(&self, home_dir: &Path) -> PathBuf {
    match self {
        HarnessKind::Opencode => home_dir.join(".config").join("opencode"),
        HarnessKind::Claude => home_dir.join(".config").join("claude"),
        HarnessKind::Pi => home_dir.join(".pi"),
        HarnessKind::Cursor => home_dir.join(".cursor"),
        HarnessKind::Copilot => home_dir.join(".config").join("github-copilot"),
        HarnessKind::Codex => home_dir.join(".config").join("codex"),
        HarnessKind::Grok => home_dir.join(".config").join("grok"),
        HarnessKind::Kimi => home_dir.join(".config").join("kimi"),
        HarnessKind::Agy => home_dir.join(".gemini").join("antigravity-cli"),
        HarnessKind::Deepseek => home_dir.join(".config").join("deepseek"),
        HarnessKind::Fx => home_dir.join(".config").join("fx"),
        HarnessKind::Custom => home_dir.join(".config").join("custom"),
    }
}
```
