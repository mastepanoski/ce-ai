---
module: src/harness/copilot.rs
tags: copilot, json, native-adapter, harness, mcp-servers
problem_type: architectural_expansion
---

# Solution: GitHub Copilot Native Harness Adapter

## Problem
Prior to Issue #177, `ce-ai` lacked native support for GitHub Copilot CLI / extension (`~/.copilot/mcp-config.json`). Generic config path assumptions were used (`~/.config/github-copilot/config.json`), causing failure since Copilot natively uses JSON configuration with `mcpServers` object, skills stored in `<harness_dir>/skills/`, and project instructions in `.github/copilot-instructions.md`.

## Solution
1. Implemented `CopilotAdapter` in `src/harness/copilot.rs` using `serde_json` (`CopilotMcpConfig` and `CopilotMcpServer`).
2. Native `mcpServers` JSON object registration and unregistration with attribute merging (`command`, `args`, `env`), preserving unmanaged top-level JSON keys and custom MCP server fields.
3. Native skills placement in `<harness_dir>/skills/<name>/SKILL.md` with automatic uninstall cleanup.
4. Support for `$COPILOT_CONFIG_DIR` environment variable override.
5. Project rule adoption in `.github/copilot-instructions.md` with demarcated `CE-AI MANAGED BLOCK`.
6. Backup tagging with `copilot-` prefix in `src/state/backups.rs`.
7. Zero OpenCode key leakage (`plugin`, `skills.paths`).
