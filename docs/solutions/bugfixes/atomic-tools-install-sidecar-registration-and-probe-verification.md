---
module: src/commands/tools.rs
tags: [tools, sidecar-registration, mcpServers, atomic-write, health-probe]
problem_type: bugfix
---

# Atomic Tools Install Sidecar Registration and Post-Probe Verification

## Problem
`ce-ai tools install <tool>` previously printed fake success lines without performing any filesystem mutation, config merge, tool binary execution, or health verification. This caused `status`, `doctor`, and `audit` to report false readiness or missing dependencies.

## Solution
1. **Atomic MCP Registration**: Implemented `register_mcp_server` in `src/opencode/config.rs` using `crate::state::write_atomic`, injecting `mcpServers.<tool>` into `opencode.json` while preserving all pre-existing user MCP servers and custom skills.
2. **Post-Install Capability Verification**: Executed post-install probe (`extract_tool_version` / `is_mcp_configured`). If probe fails, returns `CeError::Runtime` (exit code non-zero) and NEVER outputs a success message.
