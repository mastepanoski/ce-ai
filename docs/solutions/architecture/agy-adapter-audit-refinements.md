---
module: harness
tags: [agy, gemini, antigravity, harness, adapter, audit, design]
problem_type: documentation_clarification
---

# Solution: Antigravity (AGY) Adapter Audit Refinements

## Problem
Audit of Google Antigravity (`agy`) Native Harness Adapter identified three specification and design clarifications:
1. `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` are `ce-ai` extension environment variables for custom directory relocation (default is `~/.gemini`). They were missing explicit documentation as `ce-ai` extensions in `design.md`.
2. Project rule adoption targets `.agents/rules/compound-engineering.md` and `GEMINI.md`. Their architectural relationship needed explicit documentation in `design.md`.
3. Name collision policy in `register_agy_mcp_server` (converting a pre-existing remote `serverUrl` entry to `stdio` command when registered with the same name) needed formal documentation in `design.md`.

## Solution Details
1. **Environment Extensions**: Documented `$ANTIGRAVITY_CONFIG_DIR` and `$GEMINI_HOME` in `openspec/changes/agy-native-harness-adapter/design.md`.
2. **Project Rules Architecture**: Documented `GEMINI.md` as canonical instruction file and `.agents/rules/compound-engineering.md` as derived stub file in `design.md`.
3. **Collision Policy Specification**: Documented `register_agy_mcp_server` behavior where updating an entry to stdio command resets `server_url` to `None` while preserving distinct remote server entries.

## Verification
- Verified OpenSpec consistency across `openspec/changes/agy-adapter-audit-refinements/` (`proposal.md`, `exploration.md`, `design.md`, `spec.md`, `tasks.md`).
- 100% green test suite (137 unit tests, 73 integration tests).
