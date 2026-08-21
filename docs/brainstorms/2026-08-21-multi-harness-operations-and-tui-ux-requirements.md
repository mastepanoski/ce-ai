# Multi-Harness Operations & TUI UX Requirements

## 1. Problem Statement
`ce-ai` detects and supports up to 12 AI coding harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `codex`, `grok`, `kimi`, `agy`, `deepseek`, `fx`, `custom`). However, subcommands such as `sync` and `upgrade` currently operate on single or hardcoded paths, making it ambiguous whether updates apply to all installed harnesses or only `opencode`. Additionally, upgrading a harness installed from local source (`source: local`) could inadvertently overwrite local development trees.

## 2. Target User & Use Cases
- **Multi-Harness Engineers**: Developers running `compound-engineering` across OpenCode, Claude Code, Antigravity (AGY), Kimi, Pi, and Cursor simultaneously.
- **TUI Dashboard Users**: Users expecting clear target selection (`[ Target: All Installed ]`) and per-harness execution feedback.

## 3. Key Requirements
1. **Multi-Harness Bulk Upgrade & Sync (`MH-1`)**:
   - `ce-ai upgrade` and `ce-ai sync` MUST support `--harness all` (defaulting to all detected/installed host harnesses).
   - Per-harness execution feedback MUST be output cleanly for each active harness target.
2. **Local Source Protection (`MH-2`)**:
   - Upgrading a harness marked with `source: local` MUST require explicit confirmation (`--force` or prompt confirmation) or be skipped with a protective warning to prevent overwriting local development trees.
3. **TUI Global Target Selector (`MH-3`)**:
   - The TUI dashboard MUST feature an explicit global `Target Harness: [ All Installed / <harness> ]` selector accessible via `◄`/`►` or `h`/`l` across action tabs (Install, Sync, Upgrade, Models, Uninstall).
   - Execution feedback in the TUI status panel MUST clearly display itemized results for every target harness.

## 4. Success Criteria
- Running `ce-ai sync` or `ce-ai upgrade` updates all active host-installed harnesses.
- Local source installations are protected from unintended release overwrites.
- TUI navigation unambiguously shows the active target harness scope.
