---
module: src/harness/codex.rs
tags: codex, toml, native-adapter, harness, mcp-servers
problem_type: architectural_expansion
---

# Solution: Codex Native Harness Adapter

## Problem
Prior to Issue #175, `ce-ai` lacked native support for OpenAI Codex CLI (`~/.codex/config.toml`). Synthetic JSON fallback was used, causing failure since Codex natively uses TOML configuration with `[mcp_servers.<name>]` tables, skills stored in `~/.codex/skills/`, and project instructions in `AGENTS.md` / `.codex/AGENTS.md`.

## Solution
1. Implemented `CodexAdapter` in `src/harness/codex.rs` using `toml` crate (`toml::Table` / `toml::Value`).
2. Native `[mcp_servers.<name>]` TOML table registration and unregistration with attribute merging (`command`, `args`, `env`), preserving unmanaged user TOML sections and user-defined MCP server fields.
3. Native skills placement in `~/.codex/skills/<name>/SKILL.md` with automatic uninstall cleanup.
4. Support for `$CODEX_HOME` environment variable override (aligning with official Codex CLI standards).
5. Clean environment table replacement during MCP server TOML re-registration.
6. Elimination of legacy generic JSON adapter references (`.codex/config.json`).
7. Explicit OpenSpec contract clarification: `.codex/AGENTS.md` is adopted when `.codex/` exists, while root `AGENTS.md` is preserved without self-referential managed blocks.
5. Project rule adoption in `.codex/AGENTS.md` with demarcated `CE-AI MANAGED BLOCK`.
6. Zero OpenCode key leakage (`plugin`, `skills.paths`).
