# OpenSpec Proposal: Multi-Harness Operations & TUI Target Scope

## Problem Statement
Users running `ce-ai` across multiple AI agent harnesses (`opencode`, `claude`, `pi`, `cursor`, `copilot`, `kimi`, `agy`, etc.) experience ambiguity during `sync`, `upgrade`, and TUI interactions. Operations should default to updating all installed host harnesses while protecting local source installations (`source: local`) and providing clear per-harness target controls in the TUI.

## Proposed Changes
1. **Multi-Harness Target Execution**: Enable `--harness all` support in `ce-ai sync` and `ce-ai upgrade`.
2. **Local Source Protection**: Prompt or require `--force` when upgrading harnesses installed from `source: local`.
3. **TUI Target Scope Selector**: Add explicit `Target Harness: [ All Installed / <name> ]` controls in TUI navigation.

## Success Criteria
- 100% test coverage for multi-harness sync, upgrade, and local-source protection.
- Clean execution across Linux, macOS, Windows, and Docker E2E.
